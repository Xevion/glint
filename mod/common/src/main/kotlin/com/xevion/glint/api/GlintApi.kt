package com.xevion.glint.api

import com.xevion.glint.scene.Scene
import com.xevion.glint.screenshot.Camera
import com.xevion.glint.screenshot.Position
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import java.net.HttpURLConnection
import java.net.URI
import java.nio.charset.StandardCharsets

/**
 * HTTP client for communicating with the Glint backend API.
 * Handles scene CRUD operations and synchronization.
 */
object GlintApi {
    private val JSON =
        Json {
            ignoreUnknownKeys = true
            encodeDefaults = true
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
     * Tests connection to the API server by fetching world list.
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

        val url = "${validationResult.normalizedUrl}/api/admin/worlds"

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

    /**
     * Fetches list of available worlds from the API server.
     */
    fun listWorlds(apiUrl: String): Result<List<WorldInfo>> {
        val url = "$apiUrl/api/admin/worlds"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "GET"
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
     */
    fun createScene(
        apiUrl: String,
        worldId: String,
        scene: Scene,
    ): Result<ApiScene> {
        val url = "$apiUrl/api/admin/scenes"
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
     */
    fun updateScene(
        apiUrl: String,
        worldId: String,
        scene: Scene,
    ): Result<ApiScene> {
        val url = "$apiUrl/api/admin/scenes/${scene.id}"
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
     * Disables a scene on the backend (soft delete).
     */
    fun disableScene(
        apiUrl: String,
        worldId: String,
        sceneSlug: String,
    ): Result<Unit> {
        val url = "$apiUrl/api/admin/scenes/$sceneSlug?world_id=$worldId"

        return try {
            val connection = URI(url).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = "DELETE"
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
}

/**
 * Request payload for creating a scene (from mod).
 */
@Serializable
data class CreateSceneRequest(
    @SerialName("world_id") val worldId: String,
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
 * Request payload for updating a scene (position/camera/environment only).
 */
@Serializable
data class UpdateSceneRequest(
    @SerialName("world_id") val worldId: String,
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
 * Scene object returned by the backend API.
 */
@Serializable
data class ApiScene(
    val id: String,
    val name: String,
    val slug: String,
    val description: String?,
    @SerialName("world_id") val worldId: String,
    val x: Double,
    val y: Double,
    val z: Double,
    val pitch: Double,
    val yaw: Double,
    val dimension: String,
    @SerialName("time_of_day_ticks") val timeOfDayTicks: Int,
    val weather: String,
    @SerialName("weather_intensity") val weatherIntensity: Double,
    @SerialName("moon_phase") val moonPhase: Int?,
    val biome: String?,
    val active: Boolean,
    @SerialName("created_at") val createdAt: String,
)

/**
 * World information from backend API.
 */
@Serializable
data class WorldInfo(
    val id: String,
    val slug: String,
    val name: String,
    val description: String?,
    @SerialName("minecraft_version") val minecraftVersion: String,
    @SerialName("file_url") val fileUrl: String?,
    @SerialName("size_bytes") val sizeBytes: Long?,
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
