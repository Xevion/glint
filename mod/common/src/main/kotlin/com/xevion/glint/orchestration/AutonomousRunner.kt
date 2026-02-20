package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.AgentClient
import com.xevion.glint.api.CaptureRun
import com.xevion.glint.api.CaptureRunItem
import com.xevion.glint.api.CreateRunItemRequest
import com.xevion.glint.api.CreateRunRequest
import com.xevion.glint.api.HttpClient
import com.xevion.glint.api.WorkItem
import com.xevion.glint.api.captureKey
import com.xevion.glint.api.retryOnRateLimit
import com.xevion.glint.capture.HighResCapture
import com.xevion.glint.session.SessionRegistry
import net.minecraft.client.Minecraft
import java.util.concurrent.CancellationException
import java.util.concurrent.CompletableFuture
import java.util.concurrent.CompletionException
import kotlin.time.Duration.Companion.seconds

/**
 * Drives the autonomous capture loop: fetch work → create run → prepare → capture → upload → repeat.
 *
 * Asset preparation is delegated to [AssetPreparer] (runs off the tick thread).
 * The [Orchestrator] handles world/shader/scene transitions via [SessionRegistry].
 * Capture uploads are managed by [RunUploader] (thread-safe, per-run lifecycle).
 */
class AutonomousRunner(
    private val apiUrl: String,
    private val apiToken: String,
    private val forceScenes: String? = null,
    private val forceShaders: String? = null,
    private val shaderLimit: Int = 10,
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
    private var currentWorkItems: List<WorkItem> = emptyList()

    private enum class State {
        FetchingWork,
        CreatingRun,
        PreparingAssets,
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

        class PrepareAssets(
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

            State.PreparingAssets -> {
                tickPreparingAssets()
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
                            shaderLimit = shaderLimit,
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
                                        shaderVersionId = item.shader.versionId,
                                        sceneId = item.scene.id,
                                        profileId = item.shader.profileId,
                                        presetId = item.preset?.id,
                                    )
                                },
                            resolutionWidth = HighResCapture.CAPTURE_WIDTH,
                            resolutionHeight = HighResCapture.CAPTURE_HEIGHT,
                            imageFormat = "webp",
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
        currentWorkItems = workItems
        log.info("Created capture run") {
            "run_id" to runId
            "items" to runItems.size
        }

        // Build lookup: (shaderVersionId, sceneId, profileId, presetId) → RunItemInfo
        val workItemsByKey =
            workItems.associateBy { it.captureKey() }
        val itemLookup =
            runItems.associate { item ->
                val key = CaptureItemKey(item.shaderVersionId, item.sceneId, item.profileId, item.presetId)
                val workItem = workItemsByKey[key]
                val presetId = workItem?.preset?.id
                val sceneVersionId =
                    workItem?.scene?.versionId
                        ?: error("Work item missing scene_version_id for ${item.shaderVersionId}/${item.sceneId}")
                key to RunItemInfo(item.id, presetId, sceneVersionId)
            }

        runUploader = RunUploader(apiUrl, apiToken, runId, itemLookup)

        // Start asset preparation
        state = State.PreparingAssets
        pending = PendingOp.PrepareAssets(CompletableFuture.supplyAsync { assetPreparer.prepareAll(workItems) })
    }

    // -- PreparingAssets --

    private fun tickPreparingAssets() {
        val op = pending as? PendingOp.PrepareAssets ?: return
        if (!op.future.isDone) return

        pending = null
        val result = op.future.join()

        when (result) {
            is PrepResult.Ready -> {
                log.info("Assets prepared, starting orchestration") {
                    "ready_items" to result.items.size
                }

                val runId = currentRunId ?: error("No run ID for orchestration")
                val uploader = runUploader!!

                val started =
                    SessionRegistry.startLinearOrchestration(
                        items = result.items,
                        runId = runId,
                        scenePackages = result.scenePackages,
                        outputDir = "glint/runs/$runId",
                    ) { orchestrator ->
                        orchestrator.onCaptureTaken = { event ->
                            // Find the shaderVersionId for this capture from the work items.
                            // The event has sceneId + profileId + presetId; we need
                            // shaderVersionId to key into the RunUploader's lookup.
                            val profileId = event.entry.shader?.profileId
                            val presetId = event.presetId
                            val matchingItem =
                                result.items.find { item ->
                                    item.scene.id == event.sceneId &&
                                        item.shader.profileId == profileId &&
                                        item.preset?.id == presetId
                                }
                            if (matchingItem != null) {
                                uploader.handleCapture(matchingItem.shader.versionId, presetId, event)
                            } else {
                                log.warn("No matching work item for capture event") {
                                    "scene_id" to event.sceneId
                                    "profile_id" to (profileId ?: "null")
                                    "preset_id" to (presetId ?: "null")
                                }
                            }
                        }
                    }

                if (started) {
                    state = State.Capturing
                } else {
                    log.error("Failed to start orchestration")
                    uploader.failAllItems(currentWorkItems)
                    startFinalizingRun()
                }
            }

            is PrepResult.Failed -> {
                log.error("Asset preparation failed") { "reason" to result.reason }
                val uploader = runUploader!!
                if (result.isShaderFailure && result.failedShaderVersionId != null) {
                    uploader.reportShaderFailure(result.failedShaderVersionId, result.reason)
                }
                uploader.failAllItems(currentWorkItems)
                startFinalizingRun()
            }
        }
    }

    // -- Capturing --

    private fun tickCapturing() {
        if (!SessionRegistry.isOrchestrationActive()) {
            log.info("Orchestration complete")
            // Fail any items that didn't produce a screenshot
            runUploader!!.failAllItems(currentWorkItems)
            startFinalizingRun()
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
        currentWorkItems = emptyList()
        runUploader = null

        fetchWork()
    }

    // -- Helpers --

    private fun shutdown() {
        runUploader?.drainAndShutdown(60.seconds)

        state = State.Done
        log.info("Autonomous runner shutting down")
        Minecraft.getInstance().stop()
    }
}
