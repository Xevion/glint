package com.xevion.glint.api

import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.json.Json
import java.net.HttpURLConnection
import java.net.URI
import java.nio.charset.StandardCharsets

/**
 * HTTP client for the Glint backend API endpoints.
 * Used in autonomous mode to fetch work, manage capture runs, upload screenshots, and report results.
 */
object AgentApi {
    private val JSON =
        Json {
            ignoreUnknownKeys = true
            encodeDefaults = true
        }

    private fun HttpURLConnection.setBearerAuth(token: String) {
        setRequestProperty("Authorization", "Bearer $token")
    }

    /** Fetches the list of needed captures from the backend. */
    fun fetchWork(
        apiUrl: String,
        token: String,
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
        val url = "$apiUrl/api/work$params"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "GET"
            connection.setBearerAuth(token)
            connection.connectTimeout = 10000
            connection.readTimeout = 30000

            when (connection.responseCode) {
                200 -> {
                    val body = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    val items = JSON.decodeFromString(ListSerializer(WorkItem.serializer()), body)
                    Result.success(items)
                }

                else -> {
                    val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                    Result.failure(ApiError.HttpError(connection.responseCode, errorBody))
                }
            }
        } catch (e: Exception) {
            Result.failure(ApiError.fromException(e))
        }
    }

    /** Creates a capture run with planned items. */
    fun createRun(
        apiUrl: String,
        token: String,
        request: CreateRunRequest,
    ): Result<CaptureRun> {
        val url = "$apiUrl/api/runs"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setBearerAuth(token)
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 10000
            connection.readTimeout = 30000

            val requestBody = JSON.encodeToString(CreateRunRequest.serializer(), request)
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                in 200..299 -> {
                    val body = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    val run = JSON.decodeFromString<CaptureRun>(body)
                    Result.success(run)
                }

                else -> {
                    val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                    Result.failure(ApiError.HttpError(connection.responseCode, errorBody))
                }
            }
        } catch (e: Exception) {
            Result.failure(ApiError.fromException(e))
        }
    }

    /** Lists items for a capture run (to get server-assigned item IDs). */
    fun listRunItems(
        apiUrl: String,
        token: String,
        runId: String,
    ): Result<List<CaptureRunItem>> {
        val url = "$apiUrl/api/runs/$runId/items"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "GET"
            connection.setBearerAuth(token)
            connection.connectTimeout = 5000
            connection.readTimeout = 30000

            when (connection.responseCode) {
                200 -> {
                    val body = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    val items = JSON.decodeFromString(ListSerializer(CaptureRunItem.serializer()), body)
                    Result.success(items)
                }

                else -> {
                    val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                    Result.failure(ApiError.HttpError(connection.responseCode, errorBody))
                }
            }
        } catch (e: Exception) {
            Result.failure(ApiError.fromException(e))
        }
    }

    /** Gets a presigned upload URL and capture ID for a single file. */
    fun getUploadUrl(
        apiUrl: String,
        token: String,
        request: UploadUrlRequest,
    ): Result<UploadUrlResponse> {
        val url = "$apiUrl/api/upload-url"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setBearerAuth(token)
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 30000

            val requestBody = JSON.encodeToString(UploadUrlRequest.serializer(), request)
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                200 -> {
                    val body = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    val response = JSON.decodeFromString<UploadUrlResponse>(body)
                    Result.success(response)
                }

                else -> {
                    val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                    Result.failure(ApiError.HttpError(connection.responseCode, errorBody))
                }
            }
        } catch (e: Exception) {
            Result.failure(ApiError.fromException(e))
        }
    }

    /** Uploads a file to a presigned URL. */
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

    /** Reports a run item as successfully completed. */
    fun completeItem(
        apiUrl: String,
        token: String,
        runId: String,
        itemId: String,
        request: CompleteItemRequest,
    ): Result<Unit> {
        val url = "$apiUrl/api/runs/$runId/items/$itemId/complete"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setBearerAuth(token)
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 30000

            val requestBody = JSON.encodeToString(CompleteItemRequest.serializer(), request)
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

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
    }

    /** Reports a run item as failed. */
    fun failItem(
        apiUrl: String,
        token: String,
        runId: String,
        itemId: String,
        request: FailItemRequest,
    ): Result<Unit> {
        val url = "$apiUrl/api/runs/$runId/items/$itemId/fail"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setBearerAuth(token)
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            val requestBody = JSON.encodeToString(FailItemRequest.serializer(), request)
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

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
    }

    /** Claims a run item, returning a presigned URL for upload. */
    fun claimItem(
        apiUrl: String,
        token: String,
        runId: String,
        itemId: String,
        request: ClaimItemRequest,
    ): Result<ClaimItemResponse> {
        val url = "$apiUrl/api/runs/$runId/items/$itemId/claim"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setBearerAuth(token)
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 10000
            connection.readTimeout = 30000

            val requestBody = JSON.encodeToString(ClaimItemRequest.serializer(), request)
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                in 200..299 -> {
                    val body = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    val response = JSON.decodeFromString<ClaimItemResponse>(body)
                    Result.success(response)
                }

                else -> {
                    val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                    Result.failure(ApiError.HttpError(connection.responseCode, errorBody))
                }
            }
        } catch (e: Exception) {
            Result.failure(ApiError.fromException(e))
        }
    }

    /** Confirms that an upload has completed successfully. */
    fun confirmUpload(
        apiUrl: String,
        token: String,
        runId: String,
        itemId: String,
        request: ConfirmUploadRequest,
    ): Result<Unit> {
        val url = "$apiUrl/api/runs/$runId/items/$itemId/confirm-upload"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setBearerAuth(token)
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 30000

            val requestBody = JSON.encodeToString(ConfirmUploadRequest.serializer(), request)
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

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
    }

    /** Finalizes a capture run. */
    fun completeRun(
        apiUrl: String,
        token: String,
        runId: String,
    ): Result<CaptureRun> {
        val url = "$apiUrl/api/runs/$runId/complete"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setBearerAuth(token)
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 30000
            connection.outputStream.use { it.write("{}".toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                in 200..299 -> {
                    val body = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    val run = JSON.decodeFromString<CaptureRun>(body)
                    Result.success(run)
                }

                else -> {
                    val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                    Result.failure(ApiError.HttpError(connection.responseCode, errorBody))
                }
            }
        } catch (e: Exception) {
            Result.failure(ApiError.fromException(e))
        }
    }

    /** Reports a persistent shader failure (increments failure count). */
    fun reportFailure(
        apiUrl: String,
        token: String,
        request: ReportFailureRequest,
    ): Result<Unit> {
        val url = "$apiUrl/api/report-failure"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setBearerAuth(token)
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            val requestBody = JSON.encodeToString(ReportFailureRequest.serializer(), request)
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

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
    }
}
