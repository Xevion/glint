package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.AgentApi
import com.xevion.glint.api.CreateRunItemRequest
import com.xevion.glint.api.CreateRunRequest
import com.xevion.glint.api.WorkItem
import com.xevion.glint.session.SessionRegistry
import net.minecraft.client.Minecraft
import java.util.concurrent.CompletableFuture
import kotlin.time.Duration.Companion.seconds

/**
 * Drives the autonomous capture loop: fetch work → create run → capture → upload → repeat.
 *
 * Asset preparation is delegated to [AssetPreparer] (runs off the tick thread).
 * Capture uploads are managed by [RunUploader] (thread-safe, per-run lifecycle).
 */
class AutonomousRunner(
    private val apiUrl: String,
    private val apiToken: String,
    private val forceScenes: String? = null,
    private val forceShaders: String? = null,
    private val workLimit: Int = 50,
) {
    private val log = Loggers.Orchestration.get()
    private val assetPreparer = AssetPreparer(Minecraft.getInstance().gameDirectory)

    private val isForceMode: Boolean = forceScenes != null || forceShaders != null

    private var state: State = State.FetchingWork
    private var pendingFuture: CompletableFuture<*>? = null

    // Work batch (per-run)
    private var currentRunId: String? = null
    private var runUploader: RunUploader? = null

    // Shader group processing
    private var shaderGroups: List<ShaderGroup> = emptyList()
    private var currentGroupIndex: Int = 0

    private enum class State {
        FetchingWork,
        CreatingRun,
        PreparingCapture,
        Capturing,
        FinalizingRun,
        Done,
    }

    /** Call once to kick off the first work fetch. */
    fun start() {
        log.info("Autonomous runner started")
        fetchWork()
    }

    /** Call every client tick from SessionRegistry or Glint.onClientTick(). */
    fun tick() {
        when (state) {
            State.FetchingWork -> tickFetchingWork()
            State.CreatingRun -> tickCreatingRun()
            State.PreparingCapture -> tickPreparingCapture()
            State.Capturing -> tickCapturing()
            State.FinalizingRun -> tickFinalizingRun()
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
            val itemLookup =
                runItems.associate { item ->
                    Triple(item.shaderVersionId, item.sceneId, item.profile) to item.id
                }

            runUploader = RunUploader(apiUrl, apiToken, runId, itemLookup)

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

    // -- PreparingCapture (async) --

    private fun tickPreparingCapture() {
        val group =
            shaderGroups.getOrNull(currentGroupIndex) ?: run {
                startFinalizingRun()
                return
            }

        // First tick: launch async preparation
        if (pendingFuture == null) {
            log.info("Preparing shader group") {
                "progress" to "${currentGroupIndex + 1}/${shaderGroups.size}"
                "shader" to group.shaderName
                "items" to group.items.size
            }
            val runId = currentRunId
            pendingFuture = CompletableFuture.supplyAsync { assetPreparer.prepare(group, runId) }
            return
        }

        // Poll for completion
        val future = pendingFuture as? CompletableFuture<*> ?: return
        if (!future.isDone) return

        pendingFuture = null

        @Suppress("UNCHECKED_CAST")
        val result = (future as CompletableFuture<PrepResult>).join()

        when (result) {
            is PrepResult.Ready -> {
                log.info("Starting capture") {
                    "scene_count" to result.spec.sceneIds.size
                    "shader_count" to result.spec.shaders.size
                }
                val uploader = runUploader!!
                val shaderVersionId = group.shaderVersionId
                val started =
                    SessionRegistry.startOrchestration(result.spec) { orchestrator ->
                        orchestrator.onCaptureTaken = { event ->
                            uploader.handleCapture(shaderVersionId, event)
                        }
                    }
                if (started) {
                    state = State.Capturing
                } else {
                    log.error("Failed to start orchestration")
                    uploader.failUnsubmittedItems(group.shaderVersionId, group.items)
                    advanceToNextGroup()
                }
            }

            is PrepResult.Failed -> {
                log.error("Preparation failed") {
                    "shader" to group.shaderName
                    "reason" to result.reason
                }
                runUploader!!.failUnsubmittedItems(group.shaderVersionId, group.items)
                if (result.isShaderFailure) {
                    runUploader!!.reportShaderFailure(group.shaderVersionId, result.reason)
                }
                advanceToNextGroup()
            }
        }
    }

    // -- Capturing --

    private fun tickCapturing() {
        if (!SessionRegistry.isOrchestrationActive()) {
            val group = shaderGroups[currentGroupIndex]
            log.info("Orchestration complete") {
                "shader" to group.shaderName
            }

            // Fail any items that didn't produce a screenshot
            runUploader!!.failUnsubmittedItems(group.shaderVersionId, group.items)
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

        val uploader = runUploader
        pendingFuture =
            CompletableFuture.supplyAsync {
                uploader?.drainAndShutdown()
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

        // Clear state
        currentRunId = null
        shaderGroups = emptyList()
        currentGroupIndex = 0
        runUploader = null

        fetchWork()
    }

    // -- Helpers --

    private fun advanceToNextGroup() {
        currentGroupIndex++
        if (currentGroupIndex >= shaderGroups.size) {
            startFinalizingRun()
        } else {
            pendingFuture = null
            state = State.PreparingCapture
        }
    }

    private fun shutdown() {
        runUploader?.drainAndShutdown(60.seconds)

        state = State.Done
        log.info("Autonomous runner shutting down")
        Minecraft.getInstance().stop()
    }
}
