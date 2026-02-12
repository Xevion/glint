package com.xevion.glint.api

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.net.HttpURLConnection
import java.net.URI
import java.nio.charset.StandardCharsets

/**
 * Device authorization flow client.
 * Handles the device code grant flow for authenticating the mod with the backend.
 */
object AuthClient {
    private val json get() = GlintJson

    /**
     * Starts device authorization flow.
     * Uses raw HTTP because this is pre-auth (no token yet, URL may not be stored).
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
                        val response = json.decodeFromString<DeviceAuthResponse>(responseBody)
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
     * Polls for device token.
     * Handles authorization_pending, expired_token, and invalid_grant error types.
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

            val requestBody = json.encodeToString(DeviceTokenRequest.serializer(), DeviceTokenRequest(deviceCode))
            connection.outputStream.use { it.write(requestBody.toByteArray(StandardCharsets.UTF_8)) }

            when (connection.responseCode) {
                200 -> {
                    val responseBody = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                    try {
                        val response = json.decodeFromString<DeviceTokenResponse>(responseBody)
                        Result.success(response)
                    } catch (e: Exception) {
                        Result.failure(ApiError.ParseError("Failed to parse token response", e))
                    }
                }

                400 -> {
                    // Parse error response to determine specific error type
                    val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8) ?: ""
                    try {
                        val errorJson = json.decodeFromString<JsonObject>(errorBody)
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

                429 -> {
                    val retryAfter = connection.getHeaderField("Retry-After")?.toLongOrNull() ?: 5L
                    Result.failure(ApiError.RateLimited(retryAfter))
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

@Serializable
private data class DeviceTokenRequest(
    val deviceCode: String,
)

@Serializable
data class DeviceAuthResponse(
    val deviceCode: String,
    val userCode: String,
    val verificationUri: String,
    val verificationUriComplete: String,
    val expiresIn: Long,
    val interval: Long,
)

@Serializable
data class DeviceTokenResponse(
    val accessToken: String,
    val tokenType: String,
    val expiresIn: Long,
)
