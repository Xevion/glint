package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.AgentApi
import com.xevion.glint.api.CreateRunItemRequest
import com.xevion.glint.api.CreateRunRequest
import com.xevion.glint.api.FailItemRequest
import com.xevion.glint.api.ReportFailureRequest
import com.xevion.glint.api.WorkItem
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.session.SessionRegistry
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.put
import net.minecraft.client.Minecraft
import java.io.File
import java.util.concurrent.CompletableFuture
import java.util.concurrent.ConcurrentHashMap

/**
 * Drives the autonomous capture loop: fetch work → create run → capture → upload → repeat.
 *
 * Screenshots are uploaded concurrently as they are captured via [UploadManager],
 * rather than waiting for the entire capture group to finish.
 */
class AutonomousRunner(
    private val apiUrl: String,
    private val apiToken: String,
    private val forceScenes: String? = null,
    private val forceShaders: String? = null,
    private val workLimit: Int = 50,
) {
    private val log = Loggers.Orchestration.get()
    private val json = Json { ignoreUnknownKeys = true }

    private val isForceMode: Boolean = forceScenes != null || forceShaders != null

    private var state: State = State.FetchingWork
    private var pendingFuture: CompletableFuture<*>? = null

    // Work batch
    private var currentRunId: String? = null
    private var itemLookup: Map<Triple<String, String, String?>, String> = emptyMap()

    // Shader group processing
    private var shaderGroups: List<ShaderGroup> = emptyList()
    private var currentGroupIndex: Int = 0
    private var groupSuccessCount: Int = 0
    private var consecutiveEmptyRuns: Int = 0

    // Concurrent upload state
    private var uploadManager: UploadManager? = null
    private val submittedItemIds: MutableSet<String> = ConcurrentHashMap.newKeySet()
    private var waitStartTime: Long = 0

    private enum class State {
        FetchingWork,
        CreatingRun,
        PreparingCapture,
        Capturing,
        WaitingForUploads,
        FinalizingRun,
        Done,
    }

    private data class ShaderGroup(
        val shaderVersionId: String,
        val shaderId: String,
        val shaderSlug: String,
        val shaderName: String,
        val version: String,
        val downloadUrl: String?,
        val fileHash: String?,
        val items: List<WorkItem>,
    )

    /** Call once to kick off the first work fetch. */
    fun start() {
        log.info("Autonomous runner started")
        fetchWork()
    }

    /** Call every client tick from SessionRegistry or Glint.onClientTick(). */
    fun tick() {
        when (state) {
            State.FetchingWork -> {
                tickFetchingWork()
            }

            State.CreatingRun -> {
                tickCreatingRun()
            }

            State.PreparingCapture -> {
                tickPreparingCapture()
            }

            State.Capturing -> {
                tickCapturing()
            }

            State.WaitingForUploads -> {
                tickWaitingForUploads()
            }

            State.FinalizingRun -> {
                tickFinalizingRun()
            }

            State.Done -> {}
        }
    }

    val isRunning: Boolean get() = state != State.Done

    // -- FetchingWork --

    private fun fetchWork() {
        state = State.FetchingWork
        pendingFuture =
            CompletableFuture.supplyAsync {
                AgentApi.fetchWork(
                    apiUrl,
                    apiToken,
                    limit = workLimit,
                    force = isForceMode,
                    shaders = forceShaders,
                    scenes = forceScenes,
                )
            }
    }

    private fun tickFetchingWork() {
        val future = pendingFuture as? CompletableFuture<*> ?: return
        if (!future.isDone) return

        pendingFuture = null

        @Suppress("UNCHECKED_CAST")
        val result = (future as CompletableFuture<Result<List<WorkItem>>>).join()

        result
            .onSuccess { items ->
                if (items.isEmpty()) {
                    log.info("No work available, shutting down")
                    shutdown()
                    return
                }
                log.info("Fetched work") { "items" to items.size }
                startCreatingRun(items)
            }.onFailure { error ->
                log.error("Failed to fetch work") { "error" to error.message }
                shutdown()
            }
    }

    // -- CreatingRun --

    private fun startCreatingRun(workItems: List<WorkItem>) {
        state = State.CreatingRun
        pendingFuture =
            CompletableFuture.supplyAsync {
                val createRequest =
                    CreateRunRequest(
                        items =
                            workItems.map { item ->
                                CreateRunItemRequest(
                                    shaderVersionId = item.shaderVersionId,
                                    sceneId = item.sceneId,
                                    profile = item.profile,
                                )
                            },
                    )

                val runResult = AgentApi.createRun(apiUrl, apiToken, createRequest)
                val run = runResult.getOrThrow()

                val itemsResult = AgentApi.listRunItems(apiUrl, apiToken, run.id)
                val runItems = itemsResult.getOrThrow()

                Triple(run.id, runItems, workItems)
            }
    }

    private fun tickCreatingRun() {
        val future = pendingFuture as? CompletableFuture<*> ?: return
        if (!future.isDone) return

        pendingFuture = null

        try {
            @Suppress("UNCHECKED_CAST")
            val triple =
                (future as CompletableFuture<Triple<String, List<com.xevion.glint.api.CaptureRunItem>, List<WorkItem>>>).join()
            val (runId, runItems, workItems) = triple

            currentRunId = runId
            log.info("Created capture run") {
                "run_id" to runId
                "items" to runItems.size
            }

            // Build lookup: (shaderVersionId, sceneId, profile) → runItemId
            itemLookup =
                runItems.associate { item ->
                    Triple(item.shaderVersionId, item.sceneId, item.profile) to item.id
                }

            // Group work items by shader version
            shaderGroups =
                workItems
                    .groupBy { it.shaderVersionId }
                    .map { (_, items) ->
                        val first = items.first()
                        ShaderGroup(
                            shaderVersionId = first.shaderVersionId,
                            shaderId = first.shaderId,
                            shaderSlug = first.shaderSlug,
                            shaderName = first.shaderName,
                            version = first.version,
                            downloadUrl = first.downloadUrl,
                            fileHash = first.fileHash,
                            items = items,
                        )
                    }

            currentGroupIndex = 0
            state = State.PreparingCapture
        } catch (e: Exception) {
            log.error("Failed to create capture run") { "error" to e.message }
            shutdown()
        }
    }

    // -- PreparingCapture --

    private fun tickPreparingCapture() {
        val group =
            shaderGroups.getOrNull(currentGroupIndex) ?: run {
                state = State.FinalizingRun
                startFinalizingRun()
                return
            }

        try {
            log.info("Preparing shader group") {
                "progress" to "${currentGroupIndex + 1}/${shaderGroups.size}"
                "shader" to group.shaderName
                "items" to group.items.size
            }

            // Download worlds (deduplicated across items in this group)
            val worldFolders = downloadWorlds(group)
            if (worldFolders.isEmpty()) {
                failGroupItems(group, "Failed to download any worlds")
                advanceToNextGroup()
                return
            }

            // Download shader if needed
            val shaderFilename =
                if (group.shaderSlug != "vanilla") {
                    val filename = downloadShader(group)
                    if (filename == null) {
                        failGroupItems(group, "Failed to download shader: ${group.shaderName}")
                        reportShaderFailure(group, "Failed to download shader")
                        advanceToNextGroup()
                        return
                    }
                    filename
                } else {
                    null
                }

            writeSceneDefinitions(group)
            val spec = buildCaptureSpec(group, shaderFilename)

            if (spec == null) {
                log.error("Failed to build capture spec") { "shader" to group.shaderName }
                failGroupItems(group, "No valid scenes found")
                advanceToNextGroup()
                return
            }

            log.info("Starting capture") {
                "scene_count" to spec.sceneIds.size
                "shader_count" to spec.shaders.size
            }
            val started =
                SessionRegistry.startOrchestration(spec) { orchestrator ->
                    orchestrator.onScreenshotCaptured = { event -> handleScreenshotCaptured(event) }
                }
            if (started) {
                state = State.Capturing
            } else {
                log.error("Failed to start orchestration")
                failGroupItems(group, "Orchestrator failed to start")
                advanceToNextGroup()
            }
        } catch (e: Exception) {
            log.error(e, "Error preparing capture")
            failGroupItems(group, "Preparation failed: ${e.message}")
            advanceToNextGroup()
        }
    }

    // -- Screenshot callback (called on main thread) --

    private fun handleScreenshotCaptured(event: ScreenshotCapturedEvent) {
        val group = shaderGroups.getOrNull(currentGroupIndex) ?: return
        val runId = currentRunId ?: return

        val key = Triple(group.shaderVersionId, event.sceneId, event.entry.shader?.profile)
        val itemId = itemLookup[key]
        if (itemId == null) {
            log.warn("No run item found for captured screenshot") {
                "scene_id" to event.sceneId
                "profile" to (event.entry.shader?.profile ?: "null")
            }
            return
        }

        // Lazily create the upload manager for this run
        if (uploadManager == null) {
            uploadManager = UploadManager(apiUrl, apiToken, runId)
        }

        submittedItemIds.add(itemId)

        uploadManager!!.submit(
            UploadManager.UploadTask(
                itemId = itemId,
                fileBytes = event.fileBytes,
                resolutionWidth = event.entry.resolution.width,
                resolutionHeight = event.entry.resolution.height,
                capturedAt = event.entry.timestamp,
                sceneId = event.sceneId,
                profile = event.entry.shader?.profile,
            ),
        )
    }

    // -- Capturing --

    private fun tickCapturing() {
        if (!SessionRegistry.isOrchestrationActive()) {
            val group = shaderGroups[currentGroupIndex]
            log.info("Orchestration complete") {
                "shader" to group.shaderName
            }

            // Fail any items that didn't produce a screenshot
            val runId = currentRunId
            if (runId != null) {
                for (item in group.items) {
                    val key = Triple(group.shaderVersionId, item.sceneId, item.profile)
                    val itemId = itemLookup[key] ?: continue
                    if (itemId !in submittedItemIds) {
                        AgentApi.failItem(
                            apiUrl,
                            apiToken,
                            runId,
                            itemId,
                            FailItemRequest(errorMessage = "No screenshot produced"),
                        )
                    }
                }
            }

            // Check if uploads are still pending
            if (uploadManager?.hasPending == true) {
                waitStartTime = System.currentTimeMillis()
                state = State.WaitingForUploads
            } else {
                if (uploadManager != null) {
                    groupSuccessCount++
                }
                advanceToNextGroup()
            }
        }
    }

    // -- WaitingForUploads --

    private fun tickWaitingForUploads() {
        val manager =
            uploadManager ?: run {
                advanceToNextGroup()
                return
            }

        if (!manager.hasPending) {
            log.info("All uploads drained") {
                "completed" to manager.completed
                "failed" to manager.failed
            }
            if (manager.completed > 0) {
                groupSuccessCount++
            }
            advanceToNextGroup()
            return
        }

        val elapsed = System.currentTimeMillis() - waitStartTime
        if (elapsed > UPLOAD_DRAIN_TIMEOUT_MS) {
            log.warn("Upload drain timed out after ${elapsed}ms, proceeding") {
                "completed" to manager.completed
                "failed" to manager.failed
            }
            advanceToNextGroup()
        }
    }

    // -- FinalizingRun --

    private fun startFinalizingRun() {
        state = State.FinalizingRun
        val runId =
            currentRunId ?: run {
                shutdown()
                return
            }

        pendingFuture =
            CompletableFuture.supplyAsync {
                AgentApi.completeRun(apiUrl, apiToken, runId)
            }
    }

    private fun tickFinalizingRun() {
        val future = pendingFuture as? CompletableFuture<*> ?: return
        if (!future.isDone) return

        pendingFuture = null

        @Suppress("UNCHECKED_CAST")
        val result = (future as CompletableFuture<Result<com.xevion.glint.api.CaptureRun>>).join()

        result
            .onSuccess { run ->
                log.info("Capture run finalized") {
                    "run_id" to run.id
                    "completed" to run.completedItems
                    "failed" to run.failedItems
                }
            }.onFailure { error ->
                log.error("Failed to finalize run") { "error" to error.message }
            }

        // Check if the run was productive before fetching more work
        val wasProductive = groupSuccessCount > 0
        if (!wasProductive) {
            consecutiveEmptyRuns++
        } else {
            consecutiveEmptyRuns = 0
        }

        // Clear state
        currentRunId = null
        itemLookup = emptyMap()
        shaderGroups = emptyList()
        currentGroupIndex = 0
        groupSuccessCount = 0
        uploadManager?.shutdown()
        uploadManager = null
        submittedItemIds.clear()

        if (consecutiveEmptyRuns >= 2) {
            log.warn("Stopping after $consecutiveEmptyRuns consecutive runs with no successful uploads")
            shutdown()
        } else {
            fetchWork()
        }
    }

    // -- Helpers --

    private fun advanceToNextGroup() {
        // Clean up per-group upload state
        submittedItemIds.clear()

        currentGroupIndex++
        if (currentGroupIndex >= shaderGroups.size) {
            startFinalizingRun()
        } else {
            state = State.PreparingCapture
        }
    }

    private fun failGroupItems(
        group: ShaderGroup,
        message: String,
    ) {
        val runId = currentRunId ?: return
        CompletableFuture.runAsync {
            for (item in group.items) {
                val key = Triple(group.shaderVersionId, item.sceneId, item.profile)
                val itemId = itemLookup[key] ?: continue
                AgentApi
                    .failItem(
                        apiUrl,
                        apiToken,
                        runId,
                        itemId,
                        FailItemRequest(errorMessage = message),
                    ).onFailure { error ->
                        log.error("Failed to report item failure") { "error" to error.message }
                    }
            }
        }
    }

    private fun reportShaderFailure(
        group: ShaderGroup,
        message: String,
    ) {
        CompletableFuture.runAsync {
            AgentApi
                .reportFailure(
                    apiUrl,
                    apiToken,
                    ReportFailureRequest(
                        shaderVersionId = group.shaderVersionId,
                        errorMessage = message,
                    ),
                ).onFailure { error ->
                    log.error("Failed to report shader failure") { "error" to error.message }
                }
        }
    }

    private fun writeSceneDefinitions(group: ShaderGroup) {
        val mc = Minecraft.getInstance()
        val scenesDir = File(mc.gameDirectory, "glint/scenes")
        scenesDir.mkdirs()

        // Group items by world
        val itemsByWorld = group.items.groupBy { it.worldSlug }

        for ((worldSlug, items) in itemsByWorld) {
            val collectionFile = File(scenesDir, "$worldSlug.json")

            val sceneElements =
                items
                    .distinctBy { it.sceneId }
                    .mapNotNull { item ->
                        try {
                            val parsed = json.parseToJsonElement(item.sceneDefinitionJson)
                            buildJsonObject {
                                for ((key, value) in parsed.jsonObject) {
                                    if (key == "id") {
                                        put("id", item.sceneId)
                                    } else {
                                        put(key, value)
                                    }
                                }
                            }
                        } catch (e: Exception) {
                            log.warn("Failed to parse scene definition") {
                                "scene_id" to item.sceneId
                                "error" to e.message
                            }
                            null
                        }
                    }

            val collection =
                buildJsonObject {
                    put("world", worldSlug)
                    put("folder", worldSlug)
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
                "scene_count" to sceneElements.size
            }
        }

        SceneManager.clearCache()
    }

    private fun buildCaptureSpec(
        group: ShaderGroup,
        shaderFilename: String?,
    ): CaptureSpec? {
        val allSceneIds = group.items.map { it.sceneId }.distinct()
        if (allSceneIds.isEmpty()) return null

        // Build shader list — group profiles for this shader version
        val profiles = group.items.mapNotNull { it.profile }.distinct()
        val shaders =
            buildList {
                if (shaderFilename == null) {
                    add(ShaderSpec(filename = null))
                } else if (profiles.isEmpty()) {
                    add(ShaderSpec(filename = shaderFilename))
                } else {
                    for (profile in profiles) {
                        add(ShaderSpec(filename = shaderFilename, profile = profile))
                    }
                }
            }

        return CaptureSpec(
            sceneIds = allSceneIds,
            shaders = shaders,
            outputDir = "glint/runs/$currentRunId",
            shutdownOnComplete = false,
            runId = currentRunId,
        )
    }

    private fun downloadWorlds(group: ShaderGroup): Map<String, String> {
        val mc = Minecraft.getInstance()
        val savesDir = File(mc.gameDirectory, "saves")
        val worldFolders = mutableMapOf<String, String>()

        // Deduplicate worlds across items
        val uniqueWorlds =
            group.items
                .distinctBy { it.worldId }
                .map { item ->
                    object {
                        val id = item.worldId
                        val slug = item.worldSlug
                        val name = item.worldName
                        val fileUrl = item.worldFileUrl
                        val fileHash = item.worldFileHash
                    }
                }

        for (world in uniqueWorlds) {
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

    private fun downloadShader(group: ShaderGroup): String? {
        val mc = Minecraft.getInstance()
        val shaderpacksDir = File(mc.gameDirectory, "shaderpacks")
        shaderpacksDir.mkdirs()

        // Check if already present
        val files = shaderpacksDir.listFiles() ?: emptyArray()
        val existingFile =
            files.firstOrNull { it.name.contains(group.shaderSlug, ignoreCase = true) }
                ?: files.firstOrNull { it.nameWithoutExtension.contains(group.shaderName, ignoreCase = true) }

        if (existingFile != null) {
            log.debug("Shader already present") { "file" to existingFile.name }
            return existingFile.name
        }

        val downloadUrl = group.downloadUrl
        if (downloadUrl == null) {
            log.error("No download URL for shader") { "name" to group.shaderName }
            return null
        }

        val filename = "${group.shaderSlug}-${group.version}.zip"
        val targetFile = File(shaderpacksDir, filename)

        log.info("Downloading shader") {
            "name" to group.shaderName
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

    private fun shutdown() {
        uploadManager?.let { manager ->
            if (manager.hasPending) {
                log.info("Draining pending uploads before shutdown...")
                manager.awaitAll(timeoutMs = 60_000)
            }
            manager.shutdown()
        }

        state = State.Done
        log.info("Autonomous runner shutting down")
        Minecraft.getInstance().stop()
    }

    companion object {
        private const val UPLOAD_DRAIN_TIMEOUT_MS = 5 * 60 * 1000L // 5 minutes
    }
}
