package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.AgentClient
import com.xevion.glint.api.ApiError
import com.xevion.glint.api.ClaimItemRequest
import com.xevion.glint.api.ConfirmUploadRequest
import com.xevion.glint.api.FailItemRequest
import com.xevion.glint.api.HttpClient
import com.xevion.glint.api.ReportFailureRequest
import com.xevion.glint.api.WorkItem
import java.util.concurrent.CancellationException
import java.util.concurrent.CompletableFuture
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.ExecutionException
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.TimeoutException
import java.util.concurrent.atomic.AtomicInteger
import kotlin.time.Duration
import kotlin.time.Duration.Companion.minutes

/** Resolved item metadata from the work→run item mapping. */
data class RunItemInfo(
    val itemId: String,
    val worldVersionId: String,
    val sceneVersionId: String,
)

/**
 * Manages the capture-to-upload pipeline for a single capture run.
 *
 * Thread-safe for cross-thread access from both IO pool (capture events) and game thread (failure reporting).
 */
class RunUploader(
    private val apiUrl: String,
    private val apiToken: String,
    private val runId: String,
    private val itemLookup: Map<Triple<String, String, String?>, RunItemInfo>,
    maxConcurrent: Int = 4,
) {
    private val log = Loggers.Orchestration.get()
    private val client = HttpClient(apiUrl, token = apiToken)
    private val executor: ExecutorService = Executors.newFixedThreadPool(maxConcurrent)
    private val submittedItemIds: MutableSet<String> = ConcurrentHashMap.newKeySet()
    private val pendingCount = AtomicInteger(0)
    private val completedCount = AtomicInteger(0)
    private val failedCount = AtomicInteger(0)
    private val pendingFutures: ConcurrentLinkedQueue<CompletableFuture<*>> = ConcurrentLinkedQueue()

    val hasPending: Boolean get() = pendingCount.get() > 0
    val completed: Int get() = completedCount.get()
    val failed: Int get() = failedCount.get()

    /**
     * Handles a capture event by looking up the item ID and submitting an upload task.
     *
     * Called from IO pool threads.
     */
    fun handleCapture(
        shaderVersionId: String,
        event: CaptureTakenEvent,
    ) {
        val key = Triple(shaderVersionId, event.sceneId, event.entry.shader?.profile)
        val info = itemLookup[key]
        if (info == null) {
            log.warn("No run item found for capture") {
                "scene_id" to event.sceneId
                "profile" to (event.entry.shader?.profile ?: "null")
            }
            return
        }

        submittedItemIds.add(info.itemId)
        val pending = pendingCount.incrementAndGet()
        log.debug("Upload queued") {
            "item_id" to info.itemId
            "scene_id" to event.sceneId
            "bytes" to event.fileBytes.size
            "pending" to pending
        }

        executor.submit {
            try {
                executeUpload(info.itemId, info.worldVersionId, info.sceneVersionId, event)
                val completed = completedCount.incrementAndGet()
                log.debug("Upload succeeded") {
                    "item_id" to info.itemId
                    "completed" to completed
                    "failed" to failedCount.get()
                }
            } catch (e: ApiError) {
                val failed = failedCount.incrementAndGet()
                log.error("Upload failed") {
                    "item_id" to info.itemId
                    "error" to e.message
                    "completed" to completedCount.get()
                    "failed" to failed
                }
                AgentClient
                    .failItem(
                        client,
                        runId,
                        info.itemId,
                        FailItemRequest(errorMessage = "Upload failed: ${e.message}"),
                    ).onFailure { failError ->
                        log.error("Failed to report item failure") {
                            "item_id" to info.itemId
                            "error" to failError.message
                        }
                    }
            } finally {
                pendingCount.decrementAndGet()
            }
        }
    }

    /**
     * Asynchronously fails items that never received a screenshot.
     *
     * Called from game thread (tick).
     */
    fun failUnsubmittedItems(
        shaderVersionId: String,
        items: List<WorkItem>,
    ) {
        val unsubmittedEntries =
            items.mapNotNull { item ->
                val key = Triple(shaderVersionId, item.sceneId, item.profile)
                val info = itemLookup[key] ?: return@mapNotNull null
                if (info.itemId in submittedItemIds) return@mapNotNull null
                item to info.itemId
            }

        for ((item, itemId) in unsubmittedEntries) {
            val future =
                CompletableFuture.runAsync({
                    AgentClient
                        .failItem(
                            client,
                            runId,
                            itemId,
                            FailItemRequest(errorMessage = "Capture not taken"),
                        ).onSuccess {
                            log.debug("Failed unsubmitted item") {
                                "item_id" to itemId
                                "scene_id" to item.sceneId
                            }
                        }.onFailure { e ->
                            log.error("Failed to report unsubmitted item failure") {
                                "item_id" to itemId
                                "error" to e.message
                            }
                        }
                }, executor)
            pendingFutures.add(future)
        }
    }

    /**
     * Reports a shader-level failure asynchronously.
     *
     * Called from game thread (tick).
     */
    fun reportShaderFailure(
        shaderVersionId: String,
        message: String,
    ) {
        val future =
            CompletableFuture.runAsync({
                AgentClient
                    .reportFailure(
                        client,
                        ReportFailureRequest(
                            shaderVersionId = shaderVersionId,
                            errorMessage = message,
                        ),
                    ).onSuccess {
                        log.debug("Reported shader failure") {
                            "shader_version_id" to shaderVersionId
                            "message" to message
                        }
                    }.onFailure { e ->
                        log.error("Failed to report shader failure") {
                            "shader_version_id" to shaderVersionId
                            "error" to e.message
                        }
                    }
            }, executor)
        pendingFutures.add(future)
    }

    /**
     * Drains pending uploads and failure-report futures, then shuts down the executor.
     *
     * Called from game thread (tick) at end of run.
     */
    fun drainAndShutdown(timeout: Duration = 5.minutes) {
        executor.shutdown()
        if (!executor.awaitTermination(timeout.inWholeMilliseconds, TimeUnit.MILLISECONDS)) {
            log.warn("Upload executor timed out, forcing shutdown")
            executor.shutdownNow()
        }

        val futures = mutableListOf<CompletableFuture<*>>()
        while (true) {
            val future = pendingFutures.poll() ?: break
            futures.add(future)
        }

        for (future in futures) {
            try {
                future.get(10, TimeUnit.SECONDS)
            } catch (e: TimeoutException) {
                log.error("Failure report timed out") { "error" to e.message }
            } catch (e: ExecutionException) {
                log.error("Failure report did not complete") { "error" to (e.cause?.message ?: e.message) }
            } catch (e: CancellationException) {
                log.error("Failure report was cancelled") { "error" to e.message }
            } catch (e: InterruptedException) {
                Thread.currentThread().interrupt()
                log.error("Interrupted while draining failure reports")
                break
            }
        }

        log.info("RunUploader drained") {
            "completed" to completedCount.get()
            "failed" to failedCount.get()
        }
    }

    /**
     * Executes the three-phase upload pipeline: claim → upload → confirm.
     */
    private fun executeUpload(
        itemId: String,
        worldVersionId: String,
        sceneVersionId: String,
        event: CaptureTakenEvent,
    ) {
        log.debug("Claiming item") { "item_id" to itemId }
        val claimResponse =
            AgentClient
                .claimItem(
                    client,
                    runId,
                    itemId,
                    ClaimItemRequest(
                        resolutionWidth = event.entry.resolution.width,
                        resolutionHeight = event.entry.resolution.height,
                        capturedAt = event.entry.timestamp,
                        worldVersionId = worldVersionId,
                        sceneVersionId = sceneVersionId,
                    ),
                ).getOrThrow()

        log.debug("Uploading to R2") {
            "item_id" to itemId
            "capture_id" to claimResponse.captureId
        }
        AgentClient.uploadFile(claimResponse.presignedUrl, event.fileBytes).getOrThrow()

        log.debug("Confirming upload") { "item_id" to itemId }
        AgentClient.confirmUpload(client, runId, itemId, ConfirmUploadRequest()).getOrThrow()
    }
}
