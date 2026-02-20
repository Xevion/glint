package com.xevion.glint.api

import java.net.HttpURLConnection
import java.net.URI
import java.nio.charset.StandardCharsets

/** Maximum number of redirects to follow before giving up. */
private const val MAX_REDIRECTS = 5

/** HTTP status codes that indicate a redirect. */
private val REDIRECT_CODES = setOf(301, 302, 307, 308)

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
     * Tests connection to the API server, following redirects (including cross-protocol).
     *
     * First checks server reachability via the unauthenticated device status endpoint.
     * If a [token] is provided, also validates the session against `/api/user/me`.
     *
     * Returns a [ConnectionTestResult] containing the success message and the
     * resolved base URL (which may differ from [apiUrl] if redirects were followed).
     */
    fun testConnection(
        apiUrl: String,
        token: String? = null,
    ): Result<ConnectionTestResult> {
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

        val baseUrl = validationResult.normalizedUrl

        // Step 1: Check server reachability (unauthenticated), following redirects
        val resolvedBaseUrl: String
        try {
            val (finalConnection, finalUrl) = openWithRedirects("$baseUrl/api/device/status", "GET")

            if (finalConnection.responseCode != 200) {
                val errorBody = finalConnection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                return Result.failure(ApiError.HttpError(finalConnection.responseCode, errorBody))
            }

            // Derive the resolved base URL from the final URL after redirects
            resolvedBaseUrl = extractBaseUrl(finalUrl, "/api/device/status")
        } catch (e: Exception) {
            return Result.failure(ApiError.fromException(e))
        }

        // Step 2: Validate session if token is available
        if (!token.isNullOrBlank()) {
            try {
                val (connection, _) = openWithRedirects("$resolvedBaseUrl/api/user/me", "GET")
                connection.setRequestProperty("Authorization", "Bearer $token")

                when (connection.responseCode) {
                    200 -> {
                        return Result.success(ConnectionTestResult("Connection and session valid", resolvedBaseUrl))
                    }

                    401 -> {
                        return Result.failure(ApiError.HttpError(401, "Session expired or invalid"))
                    }

                    else -> {
                        val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                        return Result.failure(ApiError.HttpError(connection.responseCode, errorBody))
                    }
                }
            } catch (e: Exception) {
                return Result.failure(ApiError.fromException(e))
            }
        }

        return Result.success(ConnectionTestResult("Connection successful (no session to validate)", resolvedBaseUrl))
    }

    /**
     * Opens an HTTP connection, manually following redirects (including cross-protocol
     * http↔https redirects that [HttpURLConnection] refuses to follow automatically).
     *
     * Returns the final connection and the URL it resolved to.
     */
    private fun openWithRedirects(
        url: String,
        method: String,
        headers: Map<String, String> = emptyMap(),
    ): Pair<HttpURLConnection, String> {
        var currentUrl = url
        var redirectCount = 0

        while (true) {
            val connection = URI(currentUrl).toURL().openConnection() as HttpURLConnection
            connection.requestMethod = method
            connection.connectTimeout = 5000
            connection.readTimeout = 5000
            connection.instanceFollowRedirects = false

            for ((key, value) in headers) {
                connection.setRequestProperty(key, value)
            }

            val code = connection.responseCode
            if (code !in REDIRECT_CODES) {
                return Pair(connection, currentUrl)
            }

            val location =
                connection.getHeaderField("Location")
                    ?: return Pair(connection, currentUrl)

            // Resolve relative redirects against the current URL
            currentUrl = URI(currentUrl).resolve(location).toString()

            redirectCount++
            if (redirectCount > MAX_REDIRECTS) {
                return Pair(connection, currentUrl)
            }
        }
    }

    /**
     * Extracts the base URL from a full URL by stripping a known path suffix.
     * e.g. `extractBaseUrl("https://glint.xevion.dev/api/device/status", "/api/device/status")`
     *      returns `"https://glint.xevion.dev"`
     */
    private fun extractBaseUrl(
        fullUrl: String,
        pathSuffix: String,
    ): String {
        val idx = fullUrl.indexOf(pathSuffix)
        return if (idx > 0) fullUrl.substring(0, idx) else fullUrl
    }
}

/**
 * Result of a successful connection test.
 *
 * @property message Human-readable status message.
 * @property resolvedUrl The base URL after following any redirects (may differ from the input URL).
 */
data class ConnectionTestResult(
    val message: String,
    val resolvedUrl: String,
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
