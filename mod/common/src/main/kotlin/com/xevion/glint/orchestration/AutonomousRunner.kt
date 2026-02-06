package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.AgentApi
import com.xevion.glint.api.CaptureRecord
import com.xevion.glint.api.CompleteJobRequest
import com.xevion.glint.api.JobPayload
import com.xevion.glint.api.PrepareUploadFile
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.session.SessionRegistry
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.put
import net.minecraft.client.Minecraft
import java.io.File
import java.util.concurrent.CompletableFuture

/**
 * Drives the autonomous capture loop: claim job → capture → upload → repeat.
 *
 * Started once from TitleScreenMixin, advances each tick.
 * HTTP calls run on background threads; game-state operations run on the main thread.
 */
class AutonomousRunner(
    private val apiUrl: String,
    private val apiToken: String,
    private val forceScenes: String? = null,
    private val forceShaders: String? = null,
) {
    private val log = Loggers.Orchestration.get()
    private val json = Json { ignoreUnknownKeys = true }

    private val isForceMode: Boolean = forceScenes != null || forceShaders != null

    private var state: State = State.ClaimingJob
    private var currentJob: JobPayload? = null
    private var pendingFuture: CompletableFuture<*>? = null
    private var lastHeartbeat: Long = 0
    private var forceJobCreated: Boolean = false

    private enum class State {
        ClaimingJob,
        PreparingCapture,
        Capturing,
        UploadingResults,
        Done,
    }

    /** Call once to kick off the first job claim. */
    fun start() {
        log.info("Autonomous runner started")
        claimNextJob()
    }

    /** Call every client tick from SessionRegistry or Glint.onClientTick(). */
    fun tick() {
        when (state) {
            State.ClaimingJob -> {
                tickClaimingJob()
            }

            State.PreparingCapture -> {
                tickPreparingCapture()
            }

            State.Capturing -> {
                tickCapturing()
            }

            State.UploadingResults -> {
                tickUploadingResults()
            }

            State.Done -> {}
        }
    }

    val isRunning: Boolean get() = state != State.Done

    // -- ClaimingJob: waiting for background HTTP to return --

    private fun claimNextJob() {
        state = State.ClaimingJob
        pendingFuture =
            if (isForceMode && !forceJobCreated) {
                forceJobCreated = true
                CompletableFuture.supplyAsync {
                    AgentApi
                        .forceJob(apiUrl, apiToken, forceScenes, forceShaders)
                        .map { it as JobPayload? }
                }
            } else {
                CompletableFuture.supplyAsync {
                    AgentApi.claimJob(apiUrl, apiToken)
                }
            }
    }

    private fun tickClaimingJob() {
        val future = pendingFuture as? CompletableFuture<*> ?: return
        if (!future.isDone) return

        pendingFuture = null

        @Suppress("UNCHECKED_CAST")
        val result = (future as CompletableFuture<Result<JobPayload?>>).join()

        result
            .onSuccess { payload ->
                if (payload == null) {
                    log.info("No jobs available, shutting down")
                    shutdown()
                    return
                }
                log.info("Claimed job") {
                    "job_id" to payload.id
                    "shader" to payload.shader.name
                }
                currentJob = payload
                lastHeartbeat = System.currentTimeMillis()
                state = State.PreparingCapture
            }.onFailure { error ->
                log.error("Failed to claim job") { "error" to error.message }
                shutdown()
            }
    }

    // -- PreparingCapture: write scenes, build CaptureSpec, start orchestration --

    private fun tickPreparingCapture() {
        val job = currentJob ?: return

        try {
            // Download worlds
            val worldFolders = downloadWorlds(job)
            if (worldFolders.isEmpty()) {
                failJobAsync(job.id, "Failed to download any worlds")
                return
            }

            // Download shader if needed
            val shaderFilename =
                if (job.shader.slug != "vanilla") {
                    val filename = downloadShader(job.shader)
                    if (filename == null) {
                        failJobAsync(job.id, "Failed to download shader: ${job.shader.name}")
                        return
                    }
                    filename
                } else {
                    null
                }

            writeSceneDefinitions(job)
            val spec = buildCaptureSpec(job, shaderFilename)

            if (spec == null) {
                log.error("Failed to build capture spec") { "job_id" to job.id }
                failJobAsync(job.id, "No valid scenes found for job")
                return
            }

            log.info("Starting capture") {
                "scene_count" to spec.sceneIds.size
                "shader_count" to spec.shaders.size
            }
            if (SessionRegistry.startOrchestration(spec)) {
                state = State.Capturing
            } else {
                log.error("Failed to start orchestration")
                failJobAsync(job.id, "Orchestrator failed to start")
            }
        } catch (e: Exception) {
            log.error(e, "Error preparing capture")
            failJobAsync(job.id, "Preparation failed: ${e.message}")
        }
    }

    // -- Capturing: orchestrator is running, send heartbeats --

    private fun tickCapturing() {
        // Send periodic heartbeats (every 30 seconds)
        val now = System.currentTimeMillis()
        if (now - lastHeartbeat > 30_000) {
            lastHeartbeat = now
            CompletableFuture.runAsync {
                AgentApi.heartbeat(apiUrl, apiToken, currentJob!!.id).onFailure { error ->
                    log.warn("Heartbeat failed") { "error" to error.message }
                }
            }
        }

        // Check if orchestration is complete
        if (!SessionRegistry.isOrchestrationActive()) {
            log.info("Orchestration complete") { "job_id" to currentJob!!.id }
            startUpload()
        }
    }

    // -- UploadingResults: background upload + completion report --

    private fun startUpload() {
        state = State.UploadingResults
        val job = currentJob!!

        pendingFuture =
            CompletableFuture.supplyAsync {
                uploadAndComplete(job)
            }
    }

    private fun tickUploadingResults() {
        val future = pendingFuture as? CompletableFuture<*> ?: return
        if (!future.isDone) return

        pendingFuture = null

        @Suppress("UNCHECKED_CAST")
        val result = (future as CompletableFuture<Result<Unit>>).join()

        result
            .onSuccess {
                log.info("Job completed") { "job_id" to currentJob!!.id }
            }.onFailure { error ->
                log.error("Upload/completion failed") { "error" to error.message }
            }

        currentJob = null
        claimNextJob()
    }

    private fun uploadAndComplete(job: JobPayload): Result<Unit> {
        val mc = Minecraft.getInstance()
        val outputDir = File(mc.gameDirectory, "glint/jobs/${job.id}")
        val manifestFile = File(outputDir, "manifest.json")

        if (!manifestFile.exists()) {
            val partialFile = File(outputDir, "manifest_partial.json")
            val message =
                if (partialFile.exists()) {
                    log.warn("Only partial manifest found") { "job_id" to job.id }
                    "Capture only partially completed"
                } else {
                    "No manifest produced"
                }
            AgentApi.failJob(apiUrl, apiToken, job.id, message)
            return Result.failure(RuntimeException(message))
        }

        val manifest =
            try {
                json.decodeFromString<OrchestrationManifest>(manifestFile.readText())
            } catch (e: Exception) {
                val message = "Failed to parse manifest: ${e.message}"
                AgentApi.failJob(apiUrl, apiToken, job.id, message)
                return Result.failure(RuntimeException(message))
            }

        // Collect screenshot files with structured metadata for prepare request
        data class ScreenshotInfo(
            val file: File,
            val sceneId: String,
            val profile: String?,
            val resolution: com.xevion.glint.screenshot.Resolution,
            val timestamp: String,
        )

        val screenshotsByPath = mutableMapOf<String, ScreenshotInfo>()

        for (session in manifest.sessions) {
            for (screenshot in session.screenshots) {
                val sessionBase = File(mc.gameDirectory, session.sessionDir)
                val file = File(sessionBase, "${session.sceneId}/${screenshot.file}")
                if (file.exists()) {
                    val relativePath = file.relativeTo(mc.gameDirectory).path
                    screenshotsByPath[relativePath] =
                        ScreenshotInfo(
                            file = file,
                            sceneId = session.sceneId,
                            profile = screenshot.shader?.profile,
                            resolution = screenshot.resolution,
                            timestamp = screenshot.timestamp,
                        )
                } else {
                    log.warn("Screenshot file not found") { "path" to file.absolutePath }
                }
            }
        }

        if (screenshotsByPath.isEmpty()) {
            AgentApi.failJob(apiUrl, apiToken, job.id, "No screenshot files found")
            return Result.failure(RuntimeException("No screenshot files found"))
        }

        // Build structured prepare request
        val prepareFiles =
            screenshotsByPath.map { (path, info) ->
                PrepareUploadFile(
                    localPath = path,
                    sceneId = info.sceneId,
                    profile = info.profile,
                )
            }

        val prepareResult = AgentApi.prepareUpload(apiUrl, apiToken, job.id, prepareFiles)
        val uploads = prepareResult.getOrElse { return Result.failure(it) }.uploads

        // Upload each file using presigned URLs
        for ((path, info) in screenshotsByPath) {
            val target = uploads[path] ?: continue
            val uploadResult = AgentApi.uploadFile(target.presignedUrl, info.file.readBytes())
            uploadResult.onFailure { error ->
                log.error("Failed to upload file") {
                    "path" to path
                    "error" to error.message
                }
            }
        }

        // Build capture records referencing pre-assigned capture IDs
        val captures =
            screenshotsByPath.mapNotNull { (path, info) ->
                val target = uploads[path] ?: return@mapNotNull null
                CaptureRecord(
                    captureId = target.captureId,
                    sceneId = info.sceneId,
                    profile = info.profile,
                    screenshotPath = path,
                    resolutionWidth = info.resolution.width,
                    resolutionHeight = info.resolution.height,
                    capturedAt = info.timestamp,
                )
            }

        return AgentApi.completeJob(
            apiUrl,
            apiToken,
            job.id,
            CompleteJobRequest(captures = captures),
        )
    }

    // -- Helpers --

    private fun writeSceneDefinitions(job: JobPayload) {
        val mc = Minecraft.getInstance()
        val scenesDir = File(mc.gameDirectory, "glint/scenes")
        scenesDir.mkdirs()

        val worldMap = job.worlds.associateBy { it.id }
        val scenesByWorld = job.scenes.groupBy { it.worldId }

        for ((worldId, scenes) in scenesByWorld) {
            val world = worldMap[worldId] ?: continue
            val collectionFile = File(scenesDir, "${world.slug}.json")

            val sceneElements =
                scenes.mapNotNull { scene ->
                    try {
                        val parsed = json.parseToJsonElement(scene.definitionJson)
                        // Override id with backend UUID to avoid collisions with pre-existing collections
                        buildJsonObject {
                            for ((key, value) in parsed.jsonObject) {
                                if (key == "id") {
                                    put("id", scene.id)
                                } else {
                                    put(key, value)
                                }
                            }
                        }
                    } catch (e: Exception) {
                        log.warn("Failed to parse scene definition") {
                            "scene_id" to scene.id
                            "error" to e.message
                        }
                        null
                    }
                }

            val collection =
                buildJsonObject {
                    put("world", world.slug)
                    put("folder", world.slug)
                    put("version", "1.21.4")
                    put("scenes", JsonArray(sceneElements))
                }

            collectionFile.writeText(
                json.encodeToString(
                    kotlinx.serialization.json.JsonElement
                        .serializer(),
                    collection,
                ),
            )
            log.info("Wrote scene collection") {
                "file" to collectionFile.name
                "scene_count" to scenes.size
            }
        }

        SceneManager.clearCache()
    }

    private fun buildCaptureSpec(
        job: JobPayload,
        shaderFilename: String?,
    ): CaptureSpec? {
        // Use scene UUIDs from the job payload directly (written to collections in writeSceneDefinitions)
        val allSceneIds = job.scenes.map { it.id }
        if (allSceneIds.isEmpty()) return null

        // Build shader list
        val shaders =
            buildList {
                if (shaderFilename == null) {
                    add(ShaderSpec(filename = null))
                } else if (job.profiles.isEmpty()) {
                    add(ShaderSpec(filename = shaderFilename))
                } else {
                    for (profile in job.profiles) {
                        add(ShaderSpec(filename = shaderFilename, profile = profile))
                    }
                }
            }

        return CaptureSpec(
            sceneIds = allSceneIds,
            shaders = shaders,
            outputDir = "glint/jobs/${job.id}",
            shutdownOnComplete = false, // We handle shutdown ourselves after upload
            jobId = job.id,
        )
    }

    private fun downloadWorlds(job: JobPayload): Map<String, String> {
        val mc = Minecraft.getInstance()
        val savesDir = File(mc.gameDirectory, "saves")
        val worldFolders = mutableMapOf<String, String>()

        for (world in job.worlds) {
            val existingDir = File(savesDir, world.slug)
            if (existingDir.exists() && existingDir.isDirectory) {
                log.debug("World already present") { "slug" to world.slug }
                worldFolders[world.id] = world.slug
                continue
            }

            val fileUrl = world.fileUrl
            if (fileUrl == null) {
                log.error("No download URL for world") { "name" to world.name }
                continue
            }

            log.info("Downloading world") {
                "name" to world.name
                "slug" to world.slug
            }
            try {
                val folderPath =
                    com.xevion.glint.download.WorldDownloader
                        .downloadWorld(
                            worldSlug = world.slug,
                            worldId = world.id,
                            fileUrl = fileUrl,
                            expectedHash = world.fileHash,
                            progressCallback = { progress ->
                                log.debug("World download progress") {
                                    "slug" to world.slug
                                    "progress" to progress
                                }
                            },
                        ).join()

                val downloadedDir = File(mc.gameDirectory, folderPath)
                val levelDat = File(downloadedDir, "level.dat")

                if (levelDat.exists()) {
                    val targetDir = File(savesDir, world.slug)
                    downloadedDir.copyRecursively(targetDir, overwrite = true)
                    worldFolders[world.id] = world.slug
                    log.info("World installed") { "slug" to world.slug }
                } else {
                    val subfolders = downloadedDir.listFiles()?.filter { it.isDirectory } ?: emptyList()
                    val worldSubfolder = subfolders.find { File(it, "level.dat").exists() }
                    if (worldSubfolder != null) {
                        val targetDir = File(savesDir, world.slug)
                        worldSubfolder.copyRecursively(targetDir, overwrite = true)
                        worldFolders[world.id] = world.slug
                        log.info("World installed") {
                            "slug" to world.slug
                            "subfolder" to worldSubfolder.name
                        }
                    } else {
                        log.error("Downloaded world has no level.dat") { "path" to downloadedDir.absolutePath }
                    }
                }
            } catch (e: Exception) {
                log.error(e, "Failed to download world") { "slug" to world.slug }
            }
        }

        return worldFolders
    }

    private fun downloadShader(shader: com.xevion.glint.api.JobShaderInfo): String? {
        val mc = Minecraft.getInstance()
        val shaderpacksDir = File(mc.gameDirectory, "shaderpacks")
        shaderpacksDir.mkdirs()

        // Check if already present
        val files = shaderpacksDir.listFiles() ?: emptyArray()
        val existingFile =
            files.firstOrNull { it.name.contains(shader.slug, ignoreCase = true) }
                ?: files.firstOrNull { it.nameWithoutExtension.contains(shader.name, ignoreCase = true) }

        if (existingFile != null) {
            log.debug("Shader already present") { "file" to existingFile.name }
            return existingFile.name
        }

        val downloadUrl = shader.downloadUrl
        if (downloadUrl == null) {
            log.error("No download URL for shader") { "name" to shader.name }
            return null
        }

        val filename = "${shader.slug}-${shader.version}.zip"
        val targetFile = File(shaderpacksDir, filename)

        log.info("Downloading shader") {
            "name" to shader.name
            "file" to filename
        }
        try {
            val connection =
                java.net
                    .URI(downloadUrl)
                    .toURL()
                    .openConnection() as java.net.HttpURLConnection
            connection.requestMethod = "GET"
            connection.connectTimeout = 10000
            connection.readTimeout = 120000

            if (connection.responseCode !in 200..299) {
                log.error("Shader download failed") { "status" to connection.responseCode }
                return null
            }

            connection.inputStream.use { input ->
                targetFile.outputStream().use { output ->
                    input.copyTo(output)
                }
            }

            log.info("Shader downloaded") {
                "file" to filename
                "bytes" to targetFile.length()
            }
            return filename
        } catch (e: Exception) {
            log.error(e, "Failed to download shader")
            targetFile.delete()
            return null
        }
    }

    private fun failJobAsync(
        jobId: String,
        message: String,
    ) {
        CompletableFuture.runAsync {
            AgentApi.failJob(apiUrl, apiToken, jobId, message).onFailure { error ->
                log.error("Failed to report job failure") { "error" to error.message }
            }
        }
        currentJob = null
        claimNextJob()
    }

    private fun shutdown() {
        state = State.Done
        log.info("Autonomous runner shutting down")
        Minecraft.getInstance().stop()
    }
}
