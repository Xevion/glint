package com.xevion.glint.api

import com.xevion.glint.api.HttpClient.Method
import kotlinx.serialization.Serializable
import java.io.File
import java.net.HttpURLConnection
import java.net.URI
import java.nio.charset.StandardCharsets

/**
 * Handles the 3-step presigned URL upload flow for scene packages:
 * 1. Initiate upload (backend returns presigned URL)
 * 2. Upload package ZIP to R2 via presigned URL
 * 3. Complete upload (backend creates scene_version)
 */
object SceneUploadClient {
    private const val UPLOAD_BUFFER_SIZE = 8192

    /** Initiate upload for a NEW scene (scene doesn't exist on backend yet). */
    fun initiateNewScene(
        client: HttpClient,
        request: InitiateNewSceneUploadRequest,
    ): Result<UploadInitiatedResponse> =
        client.request {
            method = Method.POST
            path = "/api/scenes/upload"
            jsonBody(InitiateNewSceneUploadRequest.serializer(), request)
            expectedStatus = setOf(200, 201)
            onStatus(409) { ApiError.HttpError(409, "Scene already exists") }
        }

    /** Initiate upload for a new VERSION of an existing scene. */
    fun initiateVersion(
        client: HttpClient,
        sceneSlug: String,
        request: InitiateVersionUploadRequest,
    ): Result<UploadInitiatedResponse> =
        client.request {
            method = Method.POST
            path = "/api/scenes/$sceneSlug/versions"
            jsonBody(InitiateVersionUploadRequest.serializer(), request)
            expectedStatus = setOf(200, 201)
            onStatus(404) { ApiError.HttpError(404, "Scene not found") }
        }

    /** Upload the package ZIP to the presigned R2 URL. */
    fun uploadPackage(
        uploadUrl: String,
        packageFile: File,
        onProgress: (bytesUploaded: Long, totalBytes: Long) -> Unit,
    ): Result<Unit> =
        try {
            val totalBytes = packageFile.length()
            val connection = URI(uploadUrl).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "PUT"
            connection.setRequestProperty("Content-Type", "application/zip")
            connection.setFixedLengthStreamingMode(totalBytes)
            connection.doOutput = true
            connection.connectTimeout = 10000
            connection.readTimeout = 120000

            try {
                packageFile.inputStream().use { input ->
                    connection.outputStream.use { output ->
                        val buffer = ByteArray(UPLOAD_BUFFER_SIZE)
                        var bytesUploaded = 0L
                        var bytesRead: Int
                        var lastReportedPercent = -1
                        while (input.read(buffer).also { bytesRead = it } != -1) {
                            output.write(buffer, 0, bytesRead)
                            bytesUploaded += bytesRead
                            val percent = if (totalBytes > 0) ((bytesUploaded * 100) / totalBytes).toInt() else 0
                            if (percent != lastReportedPercent) {
                                lastReportedPercent = percent
                                onProgress(bytesUploaded, totalBytes)
                            }
                        }
                        // Final progress report
                        if (lastReportedPercent < 100) onProgress(bytesUploaded, totalBytes)
                    }
                }

                when (connection.responseCode) {
                    in 200..299 -> {
                        Result.success(Unit)
                    }

                    else -> {
                        val errorBody =
                            connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                        Result.failure(ApiError.HttpError(connection.responseCode, errorBody))
                    }
                }
            } finally {
                connection.disconnect()
            }
        } catch (e: Exception) {
            Result.failure(ApiError.fromException(e))
        }

    /** Complete the upload, confirming hash and creating the scene_version. */
    fun completeUpload(
        client: HttpClient,
        uploadId: String,
        request: CompleteUploadRequest,
    ): Result<CompleteUploadResponse> =
        client.request {
            method = Method.POST
            path = "/api/scenes/uploads/$uploadId/complete"
            jsonBody(CompleteUploadRequest.serializer(), request)
            onStatus(404) { ApiError.HttpError(404, "Upload not found") }
            onStatus(409) { ApiError.HttpError(409, "Upload already completed") }
        }
}

@Serializable
data class InitiateNewSceneUploadRequest(
    val name: String,
    val slug: String,
    val description: String? = null,
    val dimension: String,
    val minecraftVersion: String,
    val fileHash: String,
    val sizeBytes: Long,
)

@Serializable
data class InitiateVersionUploadRequest(
    val fileHash: String,
    val sizeBytes: Long,
    val minecraftVersion: String,
)

@Serializable
data class UploadInitiatedResponse(
    val uploadId: String,
    val uploadUrl: String,
)

@Serializable
data class UploadPosition(
    val x: Double,
    val y: Double,
    val z: Double,
)

@Serializable
data class UploadCamera(
    val yaw: Double,
    val pitch: Double,
)

@Serializable
data class UploadEnvironment(
    val timeOfDayTicks: Int,
    val weather: String,
    val weatherIntensity: Double = 0.0,
    val moonPhase: Int? = null,
)

@Serializable
data class CompleteUploadRequest(
    val fileHash: String,
    val camera: UploadCamera,
    val environment: UploadEnvironment,
    val fov: Int,
    val renderDistance: Int,
    val biome: String? = null,
    val position: UploadPosition,
)

@Serializable
data class CompleteUploadResponse(
    val sceneId: String,
    val sceneVersionId: String,
    val presetId: String? = null,
)
