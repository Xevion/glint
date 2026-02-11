package com.xevion.glint.api

import java.net.HttpURLConnection
import java.net.URI
import java.nio.charset.StandardCharsets

/**
 * URL validation and connection testing utilities for the Glint API.
 */
object UrlValidation {
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
}

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
