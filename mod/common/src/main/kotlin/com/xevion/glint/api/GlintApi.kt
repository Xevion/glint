package com.xevion.glint.api

import com.xevion.glint.capture.Camera
import com.xevion.glint.capture.Position
import com.xevion.glint.scene.Scene
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonNamingStrategy
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.net.HttpURLConnection
import java.net.URI
import java.nio.charset.StandardCharsets

/**
 * HTTP client for communicating with the Glint backend API.
 * Handles scene CRUD operations, synchronization, and device authentication.
 */
object GlintApi {
    private val JSON =
        Json {
            namingStrategy = JsonNamingStrategy.SnakeCase
            ignoreUnknownKeys = true
            encodeDefaults = true
        }

    /**
     * Sets the Authorization header on an HTTP connection if a token is provided.
     */
    private fun HttpURLConnection.setAuthHeader(token: String?) {
        if (!token.isNullOrBlank()) {
            setRequestProperty("Authorization", "Bearer $token")
        }
    }

    /**
     * Validates URL format for API connection.
     * Accepts: http://host:port, https://host:port, host:port (assumes http)
     * Rejects: URLs with paths, invalid ports, invalid hostnames
     */
    fun validateApiUrl(url: String): UrlValidationResult {
        val trimmed = url.trim()
        if (trimmed.isEmpty()) {
            return UrlValidationResult.Empty
        }

        // Check for invalid characters
        if (trimmed.contains(" ") || trimmed.contains("\t") || trimmed.contains("\n")) {
            return UrlValidationResult.Invalid("URL contains whitespace")
        }

        // Try to parse as URI
        val normalized =
            if (!trimmed.startsWith("http://") && !trimmed.startsWith("https://")) {
                "http://$trimmed"
            } else {
                trimmed
            }

        try {
            val uri = URI(normalized)

            // Must have http or https scheme
            if (uri.scheme != "http" && uri.scheme != "https") {
                return UrlValidationResult.Invalid("Only HTTP/HTTPS protocols supported")
            }

            // Must have host
            if (uri.host == null || uri.host.isEmpty()) {
                return UrlValidationResult.Invalid("No hostname specified")
            }

            // Must not have path (except root)
            if (uri.path != null && uri.path.isNotEmpty() && uri.path != "/") {
                return UrlValidationResult.Invalid("URL must not include path (only protocol://host:port)")
            }

            // Must not have query or fragment
            if (uri.query != null || uri.fragment != null) {
                return UrlValidationResult.Invalid("URL must not include query or fragment")
            }

            // Port validation (if specified)
            if (uri.port != -1 && (uri.port < 1 || uri.port > 65535)) {
                return UrlValidationResult.Invalid("Invalid port number (must be 1-65535)")
            }

            // Construct clean URL
            val cleanUrl =
                if (uri.port != -1) {
                    "${uri.scheme}://${uri.host}:${uri.port}"
                } else {
                    "${uri.scheme}://${uri.host}"
                }

            return UrlValidationResult.Valid(cleanUrl)
        } catch (e: Exception) {
            return UrlValidationResult.Invalid(e.message ?: "Invalid URL format")
        }
    }

    /**
     * Tests connection to the API server using the device status endpoint (no auth required).
     * Returns success if server responds with 200, error otherwise.
     */
    fun testConnection(apiUrl: String): Result<String> {
        val validationResult = validateApiUrl(apiUrl)
        if (validationResult !is UrlValidationResult.Valid) {
            return Result.failure(
                ApiError.ConfigError(
                    when (validationResult) {
                        is UrlValidationResult.Invalid -> validationResult.reason
                        is UrlValidationResult.Empty -> "URL is empty"
                        else -> "Invalid URL"
                    },
                ),
            )
        }

        val url = "${validationResult.normalizedUrl}/api/device/status"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "GET"
            connection.connectTimeout = 5000
            connection.readTimeout = 5000

            when (connection.responseCode) {
                200 -> {
                    Result.success("Connection successful")
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

    // ============== Device Authorization Flow ==============

    /**
     * Starts device authorization flow. Returns device_code and user_code for the mod to display.
     */
    fun startDeviceAuth(apiUrl: String): Result<DeviceAuthResponse> {
        val url = "$apiUrl/api/device/authorize"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            // Empty body for POST
            connection.outputStream.use { it.write("{}".toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                200 -> {
                    val responseBody = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    try {
                        val response = JSON.decodeFromString<DeviceAuthResponse>(responseBody)
                        Result.success(response)
                    } catch (e: Exception) {
                        Result.failure(ApiError.ParseError("Failed to parse device auth response", e))
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
     * Polls for device token. Returns the access token if user has authorized,
     * or specific error types for pending/expired/invalid states.
     */
    fun pollDeviceToken(
        apiUrl: String,
        deviceCode: String,
    ): Result<DeviceTokenResponse> {
        val url = "$apiUrl/api/device/token"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            val requestBody = """{"device_code":"$deviceCode"}"""
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                200 -> {
                    val responseBody = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    try {
                        val response = JSON.decodeFromString<DeviceTokenResponse>(responseBody)
                        Result.success(response)
                    } catch (e: Exception) {
                        Result.failure(ApiError.ParseError("Failed to parse token response", e))
                    }
                }

                400 -> {
                    // Parse error response to determine specific error type
                    val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8) ?: ""
                    try {
                        val errorJson = JSON.decodeFromString<JsonObject>(errorBody)
                        val errorType = errorJson["error"]?.jsonPrimitive?.content
                        when (errorType) {
                            "authorization_pending" -> Result.failure(ApiError.AuthorizationPending())
                            "expired_token" -> Result.failure(ApiError.TokenExpired())
                            "invalid_grant" -> Result.failure(ApiError.InvalidGrant())
                            else -> Result.failure(ApiError.HttpError(400, errorBody))
                        }
                    } catch (e: Exception) {
                        Result.failure(ApiError.HttpError(400, errorBody))
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

    // ============== Authenticated API Endpoints ==============

    /**
     * Fetches list of available worlds from the API server.
     * Requires authentication token.
     */
    fun listWorlds(
        apiUrl: String,
        token: String,
    ): Result<List<WorldInfo>> {
        val url = "$apiUrl/api/worlds"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "GET"
            connection.setAuthHeader(token)
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            when (connection.responseCode) {
                200 -> {
                    val responseBody = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    try {
                        val worlds = JSON.decodeFromString<List<WorldInfo>>(responseBody)
                        Result.success(worlds)
                    } catch (e: Exception) {
                        Result.failure(ApiError.ParseError("Failed to parse worlds list", e))
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
     * Creates a new scene on the backend.
     * Requires authentication token.
     */
    fun createScene(
        apiUrl: String,
        worldId: String,
        scene: Scene,
        token: String,
    ): Result<ApiScene> {
        val url = "$apiUrl/api/scenes"
        val request =
            CreateSceneRequest(
                worldId = worldId,
                slug = scene.id,
                name = scene.name,
                position = scene.position,
                camera = scene.camera,
                dimension = scene.dimension,
                timeOfDay = scene.timeOfDay,
                weather = scene.weather.toMinecraftString(),
                weatherIntensity = scene.weatherIntensity.toDouble(),
                moonPhase = scene.moonPhase,
                biome = scene.biome,
            )

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setRequestProperty("Content-Type", "application/json")
            connection.setAuthHeader(token)
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            val requestBody = JSON.encodeToString(CreateSceneRequest.serializer(), request)
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                201 -> {
                    val responseBody = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    try {
                        val apiScene = JSON.decodeFromString(ApiScene.serializer(), responseBody)
                        Result.success(apiScene)
                    } catch (e: Exception) {
                        Result.failure(ApiError.ParseError("Failed to parse response", e))
                    }
                }

                409 -> {
                    Result.failure(ApiError.HttpError(409, "Scene already exists"))
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
     * Updates an existing scene on the backend.
     * Requires authentication token.
     */
    fun updateScene(
        apiUrl: String,
        worldId: String,
        scene: Scene,
        token: String,
    ): Result<ApiScene> {
        val url = "$apiUrl/api/scenes/by-slug/${scene.id}"
        val request =
            UpdateSceneRequest(
                worldId = worldId,
                position = scene.position,
                camera = scene.camera,
                dimension = scene.dimension,
                timeOfDay = scene.timeOfDay,
                weather = scene.weather.toMinecraftString(),
                weatherIntensity = scene.weatherIntensity.toDouble(),
                moonPhase = scene.moonPhase,
                biome = scene.biome,
            )

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "PUT"
            connection.setRequestProperty("Content-Type", "application/json")
            connection.setAuthHeader(token)
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            val requestBody = JSON.encodeToString(UpdateSceneRequest.serializer(), request)
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                200 -> {
                    val responseBody = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    try {
                        val apiScene = JSON.decodeFromString(ApiScene.serializer(), responseBody)
                        Result.success(apiScene)
                    } catch (e: Exception) {
                        Result.failure(ApiError.ParseError("Failed to parse response", e))
                    }
                }

                404 -> {
                    Result.failure(ApiError.HttpError(404, "Scene not found"))
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
     * Fetches active scenes for a specific world from the backend.
     * Requires authentication token.
     */
    fun fetchScenes(
        apiUrl: String,
        worldId: String,
        token: String,
    ): Result<List<ApiScene>> {
        val url = "$apiUrl/api/scenes?worldId=$worldId"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "GET"
            connection.setAuthHeader(token)
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            when (connection.responseCode) {
                200 -> {
                    val responseBody = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    try {
                        val scenes = JSON.decodeFromString<List<ApiScene>>(responseBody)
                        Result.success(scenes)
                    } catch (e: Exception) {
                        Result.failure(ApiError.ParseError("Failed to parse scenes list", e))
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
     * Batch disables scenes on the backend (soft delete by slugs within a world).
     * Requires authentication token.
     */
    fun batchDisableScenes(
        apiUrl: String,
        worldId: String,
        slugs: List<String>,
        token: String,
    ): Result<Unit> {
        val url = "$apiUrl/api/scenes/batch?worldId=$worldId"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "DELETE"
            connection.setRequestProperty("Content-Type", "application/json")
            connection.setAuthHeader(token)
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            val requestBody = JSON.encodeToString(BatchDisableRequest.serializer(), BatchDisableRequest(slugs))
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                204 -> {
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

    /**
     * Disables a scene on the backend (soft delete).
     * Requires authentication token.
     */
    fun disableScene(
        apiUrl: String,
        worldId: String,
        sceneSlug: String,
        token: String,
    ): Result<Unit> {
        val url = "$apiUrl/api/scenes/by-slug/$sceneSlug?worldId=$worldId"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "DELETE"
            connection.setAuthHeader(token)
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            when (connection.responseCode) {
                204 -> {
                    Result.success(Unit)
                }

                404 -> {
                    Result.failure(ApiError.HttpError(404, "Scene not found"))
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
     * Initiates a world upload by creating the world record and obtaining a presigned upload URL.
     * Requires authentication token.
     */
    fun createWorldUpload(
        apiUrl: String,
        request: CreateWorldUploadRequest,
        token: String,
    ): Result<CreateWorldUploadResponse> {
        val url = "$apiUrl/api/worlds"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setRequestProperty("Content-Type", "application/json")
            connection.setAuthHeader(token)
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            val requestBody = JSON.encodeToString(CreateWorldUploadRequest.serializer(), request)
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                201 -> {
                    val responseBody = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    try {
                        val response = JSON.decodeFromString(CreateWorldUploadResponse.serializer(), responseBody)
                        Result.success(response)
                    } catch (e: Exception) {
                        Result.failure(ApiError.ParseError("Failed to parse world upload response", e))
                    }
                }

                409 -> {
                    Result.failure(ApiError.HttpError(409, "World slug already exists"))
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
     * Completes a world upload after the file has been uploaded to the presigned URL.
     * Requires authentication token.
     */
    fun completeWorldUpload(
        apiUrl: String,
        worldSlug: String,
        request: CompleteWorldUploadRequest,
        token: String,
    ): Result<WorldInfo> {
        val url = "$apiUrl/api/worlds/$worldSlug/complete"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setRequestProperty("Content-Type", "application/json")
            connection.setAuthHeader(token)
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            val requestBody = JSON.encodeToString(CompleteWorldUploadRequest.serializer(), request)
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                201 -> {
                    val responseBody = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    try {
                        val world = JSON.decodeFromString(WorldInfo.serializer(), responseBody)
                        Result.success(world)
                    } catch (e: Exception) {
                        Result.failure(ApiError.ParseError("Failed to parse world response", e))
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
     * Initiates a new version upload for an existing world.
     * Returns a presigned URL for uploading the world file to a staging path.
     * After uploading, call [completeWorldVersionUpload] to verify and finalize.
     */
    fun createWorldVersionUpload(
        apiUrl: String,
        worldId: String,
        request: CreateWorldVersionUploadRequest,
        token: String,
    ): Result<CreateWorldVersionUploadResponse> {
        val url = "$apiUrl/api/worlds/$worldId/versions"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setRequestProperty("Content-Type", "application/json")
            connection.setAuthHeader(token)
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            val requestBody = JSON.encodeToString(CreateWorldVersionUploadRequest.serializer(), request)
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                201 -> {
                    val responseBody = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    try {
                        val response =
                            JSON.decodeFromString(CreateWorldVersionUploadResponse.serializer(), responseBody)
                        Result.success(response)
                    } catch (e: Exception) {
                        Result.failure(ApiError.ParseError("Failed to parse world version upload response", e))
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
     * Phase 2 of world version upload: verifies the uploaded file and creates the version record.
     */
    fun completeWorldVersionUpload(
        apiUrl: String,
        worldId: String,
        request: CompleteWorldVersionUploadRequest,
        token: String,
    ): Result<Unit> {
        val url = "$apiUrl/api/worlds/$worldId/versions/complete"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "POST"
            connection.setRequestProperty("Content-Type", "application/json")
            connection.setAuthHeader(token)
            connection.doOutput = true
            connection.connectTimeout = 5000
            connection.readTimeout = 10000

            val requestBody =
                JSON.encodeToString(CompleteWorldVersionUploadRequest.serializer(), request)
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                201 -> {
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

/**
 * Request payload for creating a scene (from mod).
 */
@Serializable
data class CreateSceneRequest(
    val worldId: String,
    val slug: String,
    val name: String,
    val position: Position,
    val camera: Camera,
    val dimension: String,
    val timeOfDay: Int,
    val weather: String,
    val weatherIntensity: Double,
    val moonPhase: Int?,
    val biome: String?,
)

/**
 * Request payload for batch disabling scenes.
 */
@Serializable
data class BatchDisableRequest(
    val slugs: List<String>,
)

/**
 * Request payload for updating a scene (position/camera/environment only).
 */
@Serializable
data class UpdateSceneRequest(
    val worldId: String,
    val position: Position,
    val camera: Camera,
    val dimension: String,
    val timeOfDay: Int,
    val weather: String,
    val weatherIntensity: Double,
    val moonPhase: Int?,
    val biome: String?,
)

/**
 * Version data nested inside API scene responses.
 */
@Serializable
data class ApiSceneVersion(
    val id: String,
    val sceneId: String,
    val x: Double,
    val y: Double,
    val z: Double,
    val pitch: Double,
    val yaw: Double,
    val timeOfDayTicks: Int,
    val weather: String,
    val weatherIntensity: Double,
    val moonPhase: Int?,
    val biome: String?,
    val createdAt: String,
)

/**
 * Scene object returned by the backend API.
 * Matches the backend's SceneWithVersion / SceneListItem shape:
 * flattened Scene fields + nested version.
 */
@Serializable
data class ApiScene(
    val id: String,
    val name: String,
    val slug: String,
    val description: String?,
    val worldId: String,
    val dimension: String,
    val parentSceneId: String? = null,
    val active: Boolean,
    val createdAt: String,
    val version: ApiSceneVersion,
)

/**
 * Version information nested in WorldInfo (from backend's WorldWithDetails).
 */
@Serializable
data class WorldVersionInfo(
    val id: String,
    val worldId: String,
    val fileUrl: String?,
    val fileHash: String?,
    val sizeBytes: Long?,
    val createdAt: String,
)

/**
 * World information from backend API (WorldWithDetails shape).
 */
@Serializable
data class WorldInfo(
    val id: String,
    val slug: String,
    val name: String,
    val description: String?,
    val minecraftVersion: String,
    val createdAt: String,
    val updatedAt: String,
    val latestVersion: WorldVersionInfo?,
)

/**
 * Result of URL validation.
 */
sealed class UrlValidationResult {
    object Empty : UrlValidationResult()

    data class Valid(
        val normalizedUrl: String,
    ) : UrlValidationResult()

    data class Invalid(
        val reason: String,
    ) : UrlValidationResult()
}

/**
 * Response from device authorization request.
 */
@Serializable
data class DeviceAuthResponse(
    val deviceCode: String,
    val userCode: String,
    val verificationUri: String,
    val verificationUriComplete: String,
    val expiresIn: Long,
    val interval: Long,
)

/**
 * Response from device token polling request.
 */
@Serializable
data class DeviceTokenResponse(
    val accessToken: String,
    val tokenType: String,
    val expiresIn: Long,
)

/**
 * Request payload for initiating a world upload.
 */
@Serializable
data class CreateWorldUploadRequest(
    val name: String,
    val slug: String,
    val description: String? = null,
    val minecraftVersion: String,
    val fileHash: String,
    val fileSizeBytes: Long,
)

/**
 * Response from initiating a world upload, containing the presigned upload URL.
 */
@Serializable
data class CreateWorldUploadResponse(
    val uploadId: String,
    val presignedUrl: String,
    val expiresAt: String,
)

/**
 * Request payload for completing a world upload.
 */
@Serializable
data class CompleteWorldUploadRequest(
    val uploadId: String,
)

/**
 * Request payload for creating a new world version upload.
 */
@Serializable
data class CreateWorldVersionUploadRequest(
    val fileHash: String,
    val fileSizeBytes: Long,
)

/**
 * Response from creating a world version upload, containing the presigned upload URL.
 */
@Serializable
data class CreateWorldVersionUploadResponse(
    val uploadId: String,
    val presignedUrl: String,
    val expiresAt: String,
)

/**
 * Request payload for completing a world version upload.
 */
@Serializable
data class CompleteWorldVersionUploadRequest(
    val uploadId: String,
)
