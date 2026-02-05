package com.xevion.glint.orchestration

import com.xevion.glint.Glint
import com.xevion.glint.api.AgentApi
import com.xevion.glint.api.CaptureRecord
import com.xevion.glint.api.CompleteJobRequest
import com.xevion.glint.api.JobPayload
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.session.SessionRegistry
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.buildJsonObject
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
) {
    private val logger = Glint.LOGGER
    private val json = Json { ignoreUnknownKeys = true }

    private var state: State = State.ClaimingJob
    private var currentJob: JobPayload? = null
    private var pendingFuture: CompletableFuture<*>? = null
    private var lastHeartbeat: Long = 0

    private enum class State {
        ClaimingJob,
        PreparingCapture,
        Capturing,
        UploadingResults,
        Done,
    }

    /** Call once to kick off the first job claim. */
    fun start() {
        logger.info("Autonomous runner started")
        claimNextJob()
    }

    /** Call every client tick from SessionRegistry or Glint.onClientTick(). */
    fun tick() {
        when (state) {
            State.ClaimingJob -> tickClaimingJob()
            State.PreparingCapture -> tickPreparingCapture()
            State.Capturing -> tickCapturing()
            State.UploadingResults -> tickUploadingResults()
            State.Done -> {}
        }
    }

    val isRunning: Boolean get() = state != State.Done

    // -- ClaimingJob: waiting for background HTTP to return --

    private fun claimNextJob() {
        state = State.ClaimingJob
        pendingFuture =
            CompletableFuture.supplyAsync {
                AgentApi.claimJob(apiUrl, apiToken)
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
                    logger.info("No jobs available, shutting down")
                    shutdown()
                    return
                }
                logger.info("Claimed job: ${payload.id} (shader: ${payload.shader.name})")
                currentJob = payload
                lastHeartbeat = System.currentTimeMillis()
                state = State.PreparingCapture
            }.onFailure { error ->
                logger.error("Failed to claim job: ${error.message}")
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
                logger.error("Failed to build capture spec for job ${job.id}")
                failJobAsync(job.id, "No valid scenes found for job")
                return
            }

            logger.info("Starting capture: ${spec.sceneIds.size} scenes, ${spec.shaders.size} shaders")
            if (SessionRegistry.startOrchestration(spec)) {
                state = State.Capturing
            } else {
                logger.error("Failed to start orchestration")
                failJobAsync(job.id, "Orchestrator failed to start")
            }
        } catch (e: Exception) {
            logger.error("Error preparing capture", e)
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
                    logger.warn("Heartbeat failed: ${error.message}")
                }
            }
        }

        // Check if orchestration is complete
        if (!SessionRegistry.isOrchestrationActive()) {
            logger.info("Orchestration complete for job ${currentJob!!.id}")
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
                logger.info("Job ${currentJob!!.id} completed successfully")
            }.onFailure { error ->
                logger.error("Upload/completion failed: ${error.message}")
            }

        currentJob = null
        claimNextJob()
    }

    private fun uploadAndComplete(job: JobPayload): Result<Unit> {
        val mc = Minecraft.getInstance()
        val outputDir = File(mc.gameDirectory, "glint/jobs/${job.id}")
        val manifestFile = File(outputDir, "manifest.json")

        if (!manifestFile.exists()) {
            // Try partial manifest
            val partialFile = File(outputDir, "manifest_partial.json")
            if (partialFile.exists()) {
                logger.warn("Only partial manifest found for job ${job.id}")
                return AgentApi.failJob(apiUrl, apiToken, job.id, "Capture only partially completed")
            }
            return AgentApi.failJob(apiUrl, apiToken, job.id, "No manifest produced")
        }

        val manifest =
            try {
                json.decodeFromString<OrchestrationManifest>(manifestFile.readText())
            } catch (e: Exception) {
                return AgentApi.failJob(apiUrl, apiToken, job.id, "Failed to parse manifest: ${e.message}")
            }

        // Collect screenshot files and build capture records
        val screenshotFiles = mutableMapOf<String, File>()
        val captures = mutableListOf<CaptureRecord>()

        for (session in manifest.sessions) {
            for (screenshot in session.screenshots) {
                val sessionBase = File(mc.gameDirectory, session.sessionDir)
                val file = File(sessionBase, "${session.sceneId}/${screenshot.file}")
                if (file.exists()) {
                    val relativePath = file.relativeTo(mc.gameDirectory).path
                    screenshotFiles[relativePath] = file
                    captures.add(
                        CaptureRecord(
                            sceneId = session.sceneId,
                            profile = screenshot.shader?.profile,
                            screenshotPath = relativePath,
                            resolutionWidth = screenshot.resolution.width,
                            resolutionHeight = screenshot.resolution.height,
                            capturedAt = screenshot.timestamp,
                        ),
                    )
                } else {
                    logger.warn("Screenshot file not found: ${file.absolutePath}")
                }
            }
        }

        if (screenshotFiles.isEmpty()) {
            return AgentApi.failJob(apiUrl, apiToken, job.id, "No screenshot files found")
        }

        // Request pre-signed URLs
        val prepareResult = AgentApi.prepareUpload(apiUrl, apiToken, job.id, screenshotFiles.keys.toList())
        val urls = prepareResult.getOrElse { return Result.failure(it) }.urls

        // Upload each file
        for ((key, file) in screenshotFiles) {
            val url = urls[key] ?: continue
            val uploadResult = AgentApi.uploadFile(url, file.readBytes())
            uploadResult.onFailure { error ->
                logger.error("Failed to upload $key: ${error.message}")
            }
        }

        // Report completion
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
                        json.parseToJsonElement(scene.definitionJson)
                    } catch (e: Exception) {
                        logger.warn("Failed to parse scene definition for ${scene.id}: ${e.message}")
                        null
                    }
                }

            val collection =
                kotlinx.serialization.json.buildJsonObject {
                    put("world", world.slug)
                    put("version", "1.21.4")
                    put("scenes", kotlinx.serialization.json.JsonArray(sceneElements))
                }

            collectionFile.writeText(
                json.encodeToString(
                    kotlinx.serialization.json.JsonElement
                        .serializer(),
                    collection,
                ),
            )
            logger.info("Wrote scene collection: ${collectionFile.name} (${scenes.size} scenes)")
        }

        SceneManager.clearCache()
    }

    private fun buildCaptureSpec(
        job: JobPayload,
        shaderFilename: String?,
    ): CaptureSpec? {
        // Discover all scene IDs from the scene collections we just wrote
        val allSceneIds = mutableListOf<String>()
        for (world in job.worlds) {
            val collections = SceneManager.discoverAllCollections()
            for ((_, collection) in collections) {
                if (collection.world != world.slug) continue
                for (scene in collection.scenes) {
                    allSceneIds.add(scene.id)
                    for (variant in scene.variants) {
                        allSceneIds.add(variant.id)
                    }
                }
            }
        }

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
                logger.info("World already present: ${world.slug}")
                worldFolders[world.id] = world.slug
                continue
            }

            val fileUrl = world.fileUrl
            if (fileUrl == null) {
                logger.error("No download URL for world: ${world.name}")
                continue
            }

            logger.info("Downloading world: ${world.name} (${world.slug})")
            try {
                val folderPath =
                    com.xevion.glint.download.WorldDownloader
                        .downloadWorld(
                            worldSlug = world.slug,
                            worldId = world.id,
                            fileUrl = fileUrl,
                            expectedHash = world.fileHash,
                            progressCallback = { progress ->
                                logger.debug("World download progress: ${world.slug} - $progress")
                            },
                        ).join()

                val downloadedDir = File(mc.gameDirectory, folderPath)
                val levelDat = File(downloadedDir, "level.dat")

                if (levelDat.exists()) {
                    val targetDir = File(savesDir, world.slug)
                    downloadedDir.copyRecursively(targetDir, overwrite = true)
                    worldFolders[world.id] = world.slug
                    logger.info("World installed to saves/${world.slug}")
                } else {
                    val subfolders = downloadedDir.listFiles()?.filter { it.isDirectory } ?: emptyList()
                    val worldSubfolder = subfolders.find { File(it, "level.dat").exists() }
                    if (worldSubfolder != null) {
                        val targetDir = File(savesDir, world.slug)
                        worldSubfolder.copyRecursively(targetDir, overwrite = true)
                        worldFolders[world.id] = world.slug
                        logger.info("World installed to saves/${world.slug} (from subfolder ${worldSubfolder.name})")
                    } else {
                        logger.error("Downloaded world has no level.dat: ${downloadedDir.absolutePath}")
                    }
                }
            } catch (e: Exception) {
                logger.error("Failed to download world ${world.slug}: ${e.message}", e)
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
            logger.info("Shader already present: ${existingFile.name}")
            return existingFile.name
        }

        val downloadUrl = shader.downloadUrl
        if (downloadUrl == null) {
            logger.error("No download URL for shader: ${shader.name}")
            return null
        }

        val filename = "${shader.slug}-${shader.version}.zip"
        val targetFile = File(shaderpacksDir, filename)

        logger.info("Downloading shader: ${shader.name} -> $filename")
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
                logger.error("Shader download failed: HTTP ${connection.responseCode}")
                return null
            }

            connection.inputStream.use { input ->
                targetFile.outputStream().use { output ->
                    input.copyTo(output)
                }
            }

            logger.info("Shader downloaded: $filename (${targetFile.length()} bytes)")
            return filename
        } catch (e: Exception) {
            logger.error("Failed to download shader: ${e.message}", e)
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
                logger.error("Failed to report job failure: ${error.message}")
            }
        }
        currentJob = null
        claimNextJob()
    }

    private fun shutdown() {
        state = State.Done
        logger.info("Autonomous runner shutting down")
        Minecraft.getInstance().stop()
    }
}
