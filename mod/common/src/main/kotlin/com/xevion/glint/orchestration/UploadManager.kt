package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.AgentApi
import com.xevion.glint.api.ClaimItemRequest
import com.xevion.glint.api.ConfirmUploadRequest
import com.xevion.glint.api.FailItemRequest
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger

/**
 * Manages concurrent screenshot uploads using a fixed thread pool.
 *
 * Each task claims a run item from the backend (getting a presigned URL),
 * uploads the screenshot bytes, then confirms the upload. On failure,
 * the item is reported as failed.
 */
class UploadManager(
    private val apiUrl: String,
    private val apiToken: String,
    private val runId: String,
    maxConcurrent: Int = 4,
) {
    private val log = Loggers.Orchestration.get()
    private val executor = Executors.newFixedThreadPool(maxConcurrent)
    private val pendingCount = AtomicInteger(0)
    private val completedCount = AtomicInteger(0)
    private val failedCount = AtomicInteger(0)

    data class UploadTask(
        val itemId: String,
        val fileBytes: ByteArray,
        val resolutionWidth: Int,
        val resolutionHeight: Int,
        val capturedAt: String,
        val sceneId: String,
        val profile: String?,
    )

    val hasPending: Boolean get() = pendingCount.get() > 0
    val completed: Int get() = completedCount.get()
    val failed: Int get() = failedCount.get()

    fun submit(task: UploadTask) {
        val pending = pendingCount.incrementAndGet()
        log.info("Upload queued") {
            "item_id" to task.itemId
            "scene_id" to task.sceneId
            "bytes" to task.fileBytes.size
            "pending" to pending
        }
        executor.submit {
            try {
                executeUpload(task)
                val completed = completedCount.incrementAndGet()
                log.info("Upload succeeded") {
                    "item_id" to task.itemId
                    "completed" to completed
                    "failed" to failedCount.get()
                }
            } catch (e: Exception) {
                val failed = failedCount.incrementAndGet()
                log.error("Upload failed") {
                    "item_id" to task.itemId
                    "error" to e.message
                    "completed" to completedCount.get()
                    "failed" to failed
                }
                try {
                    AgentApi.failItem(
                        apiUrl,
                        apiToken,
                        runId,
                        task.itemId,
                        FailItemRequest(errorMessage = "Upload failed: ${e.message}"),
                    )
                } catch (failError: Exception) {
                    log.error("Failed to report item failure: ${failError.message}")
                }
            } finally {
                pendingCount.decrementAndGet()
            }
        }
    }

    private fun executeUpload(task: UploadTask) {
        log.debug("Claiming item") { "item_id" to task.itemId }
        val claimResponse =
            AgentApi
                .claimItem(
                    apiUrl,
                    apiToken,
                    runId,
                    task.itemId,
                    ClaimItemRequest(
                        resolutionWidth = task.resolutionWidth,
                        resolutionHeight = task.resolutionHeight,
                        capturedAt = task.capturedAt,
                    ),
                ).getOrThrow()

        log.debug("Uploading to R2") {
            "item_id" to task.itemId
            "capture_id" to claimResponse.captureId
        }
        AgentApi.uploadFile(claimResponse.presignedUrl, task.fileBytes).getOrThrow()

        log.debug("Confirming upload") { "item_id" to task.itemId }
        AgentApi
            .confirmUpload(
                apiUrl,
                apiToken,
                runId,
                task.itemId,
                ConfirmUploadRequest(),
            ).getOrThrow()
    }

    fun awaitAll(timeoutMs: Long = 30_000) {
        executor.shutdown()
        if (!executor.awaitTermination(timeoutMs, TimeUnit.MILLISECONDS)) {
            log.warn("Upload manager timed out waiting for pending uploads")
            executor.shutdownNow()
        }
    }

    fun shutdown() {
        executor.shutdown()
        if (!executor.awaitTermination(5, TimeUnit.SECONDS)) {
            executor.shutdownNow()
        }
    }
}
