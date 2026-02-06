package com.xevion.glint.api

import kotlinx.serialization.json.Json
import java.net.HttpURLConnection
import java.net.URI
import java.nio.charset.StandardCharsets

/**
 * HTTP client for the Glint backend agent API endpoints.
 * Used in autonomous mode to claim jobs, upload screenshots, and report results.
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

    /**
     * Claims the next available job from the backend.
     * Returns null if no jobs are available.
     */
    fun claimJob(
        apiUrl: String,
        token: String,
    ): Result<JobPayload?> {
        val url = "$apiUrl/api/agent/jobs/next"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "GET"
            connection.setBearerAuth(token)
            connection.connectTimeout = 10000
            connection.readTimeout = 30000

            when (connection.responseCode) {
                200 -> {
                    val body = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    if (body.isBlank() || body.trim() == "null") {
                        Result.success(null)
                    } else {
                        try {
                            val payload = JSON.decodeFromString<JobPayload>(body)
                            Result.success(payload)
                        } catch (e: Exception) {
                            Result.failure(ApiError.ParseError("Failed to parse job payload", e))
                        }
                    }
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

    /**
     * Force-creates a job using selector syntax.
     * Returns a fully claimed JobPayload ready for execution.
     */
    fun forceJob(
        apiUrl: String,
        token: String,
        scenes: String?,
        shaders: String?,
    ): Result<JobPayload> {
        val url = "$apiUrl/api/agent/jobs/force"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setBearerAuth(token)
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 10000
            connection.readTimeout = 30000

            val requestBody = JSON.encodeToString(ForceJobRequest.serializer(), ForceJobRequest(scenes, shaders))
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                200 -> {
                    val body = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    val payload = JSON.decodeFromString<JobPayload>(body)
                    Result.success(payload)
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

    /**
     * Sends a heartbeat for an active job.
     */
    fun heartbeat(
        apiUrl: String,
        token: String,
        jobId: String,
    ): Result<Unit> {
        val url = "$apiUrl/api/agent/jobs/$jobId/heartbeat"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setBearerAuth(token)
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 10000
            connection.outputStream.use { it.write("{}".toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                in 200..299 -> Result.success(Unit)
                else -> {
                    val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                    Result.failure(ApiError.HttpError(connection.responseCode, errorBody))
                }
            }
        } catch (e: Exception) {
            Result.failure(ApiError.fromException(e))
        }
    }

    /**
     * Requests pre-signed upload URLs for screenshot files.
     */
    fun prepareUpload(
        apiUrl: String,
        token: String,
        jobId: String,
        files: List<String>,
    ): Result<PrepareUploadResponse> {
        val url = "$apiUrl/api/agent/jobs/$jobId/prepare"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setBearerAuth(token)
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 30000

            val requestBody = JSON.encodeToString(PrepareUploadRequest.serializer(), PrepareUploadRequest(files))
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                200 -> {
                    val body = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    val response = JSON.decodeFromString<PrepareUploadResponse>(body)
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

    /**
     * Uploads a file to a pre-signed URL.
     */
    fun uploadFile(
        presignedUrl: String,
        fileBytes: ByteArray,
        contentType: String = "image/png",
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
                in 200..299 -> Result.success(Unit)
                else -> {
                    val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                    Result.failure(ApiError.HttpError(connection.responseCode, errorBody))
                }
            }
        } catch (e: Exception) {
            Result.failure(ApiError.fromException(e))
        }

    /**
     * Reports job completion with capture records.
     */
    fun completeJob(
        apiUrl: String,
        token: String,
        jobId: String,
        request: CompleteJobRequest,
    ): Result<Unit> {
        val url = "$apiUrl/api/agent/jobs/$jobId/complete"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setBearerAuth(token)
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 30000

            val requestBody = JSON.encodeToString(CompleteJobRequest.serializer(), request)
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                in 200..299 -> Result.success(Unit)
                else -> {
                    val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                    Result.failure(ApiError.HttpError(connection.responseCode, errorBody))
                }
            }
        } catch (e: Exception) {
            Result.failure(ApiError.fromException(e))
        }
    }

    /**
     * Reports job failure.
     */
    fun failJob(
        apiUrl: String,
        token: String,
        jobId: String,
        errorMessage: String,
    ): Result<Unit> {
        val url = "$apiUrl/api/agent/jobs/$jobId/fail"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setBearerAuth(token)
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            val requestBody = JSON.encodeToString(FailJobRequest.serializer(), FailJobRequest(errorMessage))
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                in 200..299 -> Result.success(Unit)
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
