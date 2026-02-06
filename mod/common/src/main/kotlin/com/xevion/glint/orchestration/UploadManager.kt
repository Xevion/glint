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
        pendingCount.incrementAndGet()
        executor.submit {
            try {
                executeUpload(task)
                completedCount.incrementAndGet()
            } catch (e: Exception) {
                failedCount.incrementAndGet()
                log.error("Upload failed for item ${task.itemId}: ${e.message}")
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
        // 1. Claim the item (get presigned URL)
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

        // 2. Upload the file
        AgentApi.uploadFile(claimResponse.presignedUrl, task.fileBytes).getOrThrow()

        // 3. Confirm the upload
        AgentApi
            .confirmUpload(
                apiUrl,
                apiToken,
                runId,
                task.itemId,
                ConfirmUploadRequest(),
            ).getOrThrow()

        log.debug("Upload complete") {
            "item_id" to task.itemId
            "capture_id" to claimResponse.captureId
        }
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
