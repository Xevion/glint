package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.AgentClient
import com.xevion.glint.api.CaptureRun
import com.xevion.glint.api.CaptureRunItem
import com.xevion.glint.api.CreateRunItemRequest
import com.xevion.glint.api.CreateRunRequest
import com.xevion.glint.api.HttpClient
import com.xevion.glint.api.WorkItem
import com.xevion.glint.api.retryOnRateLimit
import com.xevion.glint.session.SessionRegistry
import net.minecraft.client.Minecraft
import java.util.concurrent.CancellationException
import java.util.concurrent.CompletableFuture
import java.util.concurrent.CompletionException
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
    private val client = HttpClient(apiUrl, token = apiToken)
    private val assetPreparer = AssetPreparer(Minecraft.getInstance().gameDirectory)

    private val isForceMode: Boolean = forceScenes != null || forceShaders != null

    private var state: State = State.FetchingWork
    private var pending: PendingOp? = null

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

    private sealed class PendingOp {
        abstract val future: CompletableFuture<*>

        class FetchWork(
            override val future: CompletableFuture<Result<List<WorkItem>>>,
        ) : PendingOp()

        class CreateRun(
            override val future: CompletableFuture<Triple<String, List<CaptureRunItem>, List<WorkItem>>>,
        ) : PendingOp()

        class PrepareCapture(
            override val future: CompletableFuture<PrepResult>,
        ) : PendingOp()

        class FinalizeRun(
            override val future: CompletableFuture<Result<CaptureRun>>,
        ) : PendingOp()
    }

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
        pending =
            PendingOp.FetchWork(
                CompletableFuture.supplyAsync {
                    retryOnRateLimit(operationName = "fetch work") {
                        AgentClient.fetchWork(
                            client,
                            limit = workLimit,
                            force = isForceMode,
                            shaders = forceShaders,
                            scenes = forceScenes,
                        )
                    }
                },
            )
    }

    private fun tickFetchingWork() {
        val op = pending as? PendingOp.FetchWork ?: return
        if (!op.future.isDone) return

        pending = null
        val result = op.future.join()

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
        pending =
            PendingOp.CreateRun(
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

                    val run =
                        retryOnRateLimit(operationName = "create run") {
                            AgentClient.createRun(client, createRequest)
                        }.getOrThrow()

                    val runItems =
                        retryOnRateLimit(operationName = "list run items") {
                            AgentClient.listRunItems(client, run.id)
                        }.getOrThrow()

                    Triple(run.id, runItems, workItems)
                },
            )
    }

    private fun tickCreatingRun() {
        val op = pending as? PendingOp.CreateRun ?: return
        if (!op.future.isDone) return

        pending = null

        val triple =
            try {
                op.future.join()
            } catch (e: CompletionException) {
                log.error("Failed to create capture run") { "error" to (e.cause?.message ?: e.message) }
                shutdown()
                return
            } catch (e: CancellationException) {
                log.error("Capture run creation cancelled") { "error" to e.message }
                shutdown()
                return
            }

        val (runId, runItems, workItems) = triple

        currentRunId = runId
        log.info("Created capture run") {
            "run_id" to runId
            "items" to runItems.size
        }

        // Build lookup: (shaderVersionId, sceneId, profile) → RunItemInfo
        // Merge run items (which have item IDs) with work items (which have world version IDs)
        val workItemsByKey =
            workItems.associateBy { Triple(it.shaderVersionId, it.sceneId, it.profile) }
        val itemLookup =
            runItems.associate { item ->
                val key = Triple(item.shaderVersionId, item.sceneId, item.profile)
                val workItem = workItemsByKey[key]
                val worldVersionId =
                    workItem?.worldVersionId
                        ?: error("Work item missing world_version_id for ${item.shaderVersionId}/${item.sceneId}")
                val sceneVersionId =
                    workItem.sceneVersionId
                        ?: error("Work item missing scene_version_id for ${item.shaderVersionId}/${item.sceneId}")
                key to RunItemInfo(item.id, worldVersionId, sceneVersionId)
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
    }

    // -- PreparingCapture (async) --

    private fun tickPreparingCapture() {
        val group =
            shaderGroups.getOrNull(currentGroupIndex) ?: run {
                startFinalizingRun()
                return
            }

        // First tick: launch async preparation
        if (pending == null) {
            log.info("Preparing shader group") {
                "progress" to "${currentGroupIndex + 1}/${shaderGroups.size}"
                "shader" to group.shaderName
                "items" to group.items.size
            }
            val runId = currentRunId
            pending = PendingOp.PrepareCapture(CompletableFuture.supplyAsync { assetPreparer.prepare(group, runId) })
            return
        }

        // Poll for completion
        val op = pending as? PendingOp.PrepareCapture ?: return
        if (!op.future.isDone) return

        pending = null
        val result = op.future.join()

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
        pending =
            PendingOp.FinalizeRun(
                CompletableFuture.supplyAsync {
                    uploader?.drainAndShutdown()
                    retryOnRateLimit(operationName = "complete run $runId") {
                        AgentClient.completeRun(client, runId)
                    }
                },
            )
    }

    private fun tickFinalizingRun() {
        val op = pending as? PendingOp.FinalizeRun ?: return
        if (!op.future.isDone) return

        pending = null
        val result = op.future.join()

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
            pending = null
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
