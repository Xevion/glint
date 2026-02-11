package com.xevion.glint.api

import com.xevion.glint.api.HttpClient.Method
import java.net.HttpURLConnection
import java.net.URI
import java.nio.charset.StandardCharsets

/**
 * HTTP client for autonomous capture agent endpoints.
 * Used by the orchestration system to fetch work, manage runs, and upload captures.
 */
object AgentClient {
    fun fetchWork(
        client: HttpClient,
        limit: Int = 100,
        force: Boolean = false,
        shaders: String? = null,
        scenes: String? = null,
    ): Result<List<WorkItem>> {
        val params =
            buildString {
                append("?limit=$limit")
                if (force) append("&force=true")
                if (shaders != null) append("&shaders=$shaders")
                if (scenes != null) append("&scenes=$scenes")
            }
        return client.request {
            path = "/api/work$params"
            connectTimeoutOverride = 10000
            readTimeoutOverride = 30000
        }
    }

    fun createRun(
        client: HttpClient,
        request: CreateRunRequest,
    ): Result<CaptureRun> =
        client.request {
            method = Method.POST
            path = "/api/runs"
            jsonBody(CreateRunRequest.serializer(), request)
            expectedStatus = setOf(200, 201)
            connectTimeoutOverride = 10000
            readTimeoutOverride = 30000
        }

    fun listRunItems(
        client: HttpClient,
        runId: String,
    ): Result<List<CaptureRunItem>> =
        client.request {
            path = "/api/runs/$runId/items"
            readTimeoutOverride = 30000
        }

    fun claimItem(
        client: HttpClient,
        runId: String,
        itemId: String,
        request: ClaimItemRequest,
    ): Result<ClaimItemResponse> =
        client.request {
            method = Method.POST
            path = "/api/runs/$runId/items/$itemId/claim"
            jsonBody(ClaimItemRequest.serializer(), request)
            connectTimeoutOverride = 10000
            readTimeoutOverride = 30000
        }

    /**
     * Uploads a file to a presigned URL (external, no auth).
     */
    fun uploadFile(
        presignedUrl: String,
        fileBytes: ByteArray,
        contentType: String = "image/webp",
    ): Result<Unit> =
        try {
            val connection = URI(presignedUrl).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "PUT"
            connection.setRequestProperty("Content-Type", contentType)
            connection.doOutput = true
            connection.connectTimeout = 10000
            connection.readTimeout = 60000
            connection.outputStream.use { it.write(fileBytes) }

            when (connection.responseCode) {
                in 200..299 -> {
                    Result.success(Unit)
                }

                else -> {
                    val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                    Result.failure(ApiError.HttpError(connection.responseCode, errorBody))
                }
            }
        } catch (e: Exception) {
            Result.failure(ApiError.fromException(e))
        }

    fun confirmUpload(
        client: HttpClient,
        runId: String,
        itemId: String,
        request: ConfirmUploadRequest,
    ): Result<Unit> =
        client.requestUnit {
            method = Method.POST
            path = "/api/runs/$runId/items/$itemId/confirm-upload"
            jsonBody(ConfirmUploadRequest.serializer(), request)
            readTimeoutOverride = 30000
        }

    fun completeItem(
        client: HttpClient,
        runId: String,
        itemId: String,
        request: CompleteItemRequest,
    ): Result<Unit> =
        client.requestUnit {
            method = Method.POST
            path = "/api/runs/$runId/items/$itemId/complete"
            jsonBody(CompleteItemRequest.serializer(), request)
            readTimeoutOverride = 30000
        }

    fun failItem(
        client: HttpClient,
        runId: String,
        itemId: String,
        request: FailItemRequest,
    ): Result<Unit> =
        client.requestUnit {
            method = Method.POST
            path = "/api/runs/$runId/items/$itemId/fail"
            jsonBody(FailItemRequest.serializer(), request)
        }

    fun completeRun(
        client: HttpClient,
        runId: String,
    ): Result<CaptureRun> =
        client.request {
            method = Method.POST
            path = "/api/runs/$runId/complete"
            body = "{}"
            readTimeoutOverride = 30000
        }

    fun reportFailure(
        client: HttpClient,
        request: ReportFailureRequest,
    ): Result<Unit> =
        client.requestUnit {
            method = Method.POST
            path = "/api/report-failure"
            jsonBody(ReportFailureRequest.serializer(), request)
        }
}
