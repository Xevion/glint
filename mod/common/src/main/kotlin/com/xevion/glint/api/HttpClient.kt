package com.xevion.glint.api

import kotlinx.serialization.KSerializer
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonNamingStrategy
import kotlinx.serialization.serializer
import java.net.HttpURLConnection
import java.net.URI
import java.nio.charset.StandardCharsets

/** HTTP status codes that indicate a redirect. */
private val REDIRECT_CODES = setOf(301, 302, 307, 308)

/** Maximum number of redirects to follow before giving up. */
private const val MAX_REDIRECTS = 5

/** Shared Json configuration for all Glint API communication. */
val GlintJson: Json =
    Json {
        namingStrategy = JsonNamingStrategy.SnakeCase
        ignoreUnknownKeys = true
        encodeDefaults = true
    }

/** Shared Json configuration for file I/O (scenes, manifests, async file ops). */
val GlintJsonFile: Json =
    Json {
        namingStrategy = JsonNamingStrategy.SnakeCase
        ignoreUnknownKeys = true
        encodeDefaults = true
        prettyPrint = true
    }

/**
 * Shared HTTP client for Glint backend API communication.
 * Provides a request-builder DSL that eliminates per-endpoint boilerplate.
 */
class HttpClient(
    val baseUrl: String,
    private val token: String? = null,
    private val connectTimeout: Int = 5000,
    private val readTimeout: Int = 10000,
) {
    val json: Json get() = GlintJson

    enum class Method { GET, POST, PUT, DELETE }

    /**
     * Executes an HTTP request and deserializes the response.
     * Type parameter T is the expected response type.
     */
    inline fun <reified T> request(block: RequestBuilder.() -> Unit): Result<T> {
        val builder = RequestBuilder().apply(block)
        return executeRequest(builder, serializer<T>())
    }

    /**
     * Executes an HTTP request that returns no body (Unit).
     */
    fun requestUnit(block: RequestBuilder.() -> Unit): Result<Unit> {
        val builder = RequestBuilder().apply(block)
        return executeUnitRequest(builder)
    }

    fun <T> executeRequest(
        builder: RequestBuilder,
        serializer: KSerializer<T>,
    ): Result<T> {
        val startUrl = if (builder.fullUrl != null) builder.fullUrl!! else "$baseUrl${builder.path}"

        return try {
            val (connection, _) = executeWithRedirects(startUrl, builder)

            val responseCode = connection.responseCode
            val statusHandler = builder.statusHandlers[responseCode]
            if (statusHandler != null) {
                return Result.failure(statusHandler())
            }

            if (responseCode in builder.expectedStatus) {
                val body = connection.inputStream.readBytes().toString(StandardCharsets.UTF_8)
                try {
                    Result.success(json.decodeFromString(serializer, body))
                } catch (e: Exception) {
                    Result.failure(ApiError.ParseError("Failed to parse response", e))
                }
            } else if (responseCode == 429) {
                val retryAfter = connection.getHeaderField("Retry-After")?.toLongOrNull() ?: 1L
                Result.failure(ApiError.RateLimited(retryAfter))
            } else {
                val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                Result.failure(ApiError.HttpError(responseCode, errorBody))
            }
        } catch (e: Exception) {
            Result.failure(ApiError.fromException(e))
        }
    }

    fun executeUnitRequest(builder: RequestBuilder): Result<Unit> {
        val startUrl = if (builder.fullUrl != null) builder.fullUrl!! else "$baseUrl${builder.path}"

        return try {
            val (connection, _) = executeWithRedirects(startUrl, builder)

            val responseCode = connection.responseCode
            val statusHandler = builder.statusHandlers[responseCode]
            if (statusHandler != null) {
                return Result.failure(statusHandler())
            }

            if (responseCode in builder.expectedStatus) {
                Result.success(Unit)
            } else if (responseCode == 429) {
                val retryAfter = connection.getHeaderField("Retry-After")?.toLongOrNull() ?: 1L
                Result.failure(ApiError.RateLimited(retryAfter))
            } else {
                val errorBody = connection.errorStream?.readBytes()?.toString(StandardCharsets.UTF_8)
                Result.failure(ApiError.HttpError(responseCode, errorBody))
            }
        } catch (e: Exception) {
            Result.failure(ApiError.fromException(e))
        }
    }

    /**
     * Opens a connection and follows redirects (including cross-protocol http↔https
     * redirects that [HttpURLConnection] refuses to follow automatically).
     *
     * Returns the final connection and the URL it resolved to.
     */
    private fun executeWithRedirects(
        url: String,
        builder: RequestBuilder,
    ): Pair<HttpURLConnection, String> {
        var currentUrl = url
        var redirectCount = 0

        while (true) {
            val connection = openConnection(currentUrl, builder)
            writeBody(connection, builder)

            val code = connection.responseCode
            if (code !in REDIRECT_CODES) {
                return Pair(connection, currentUrl)
            }

            val location =
                connection.getHeaderField("Location")
                    ?: return Pair(connection, currentUrl)

            currentUrl = URI(currentUrl).resolve(location).toString()

            redirectCount++
            if (redirectCount > MAX_REDIRECTS) {
                return Pair(connection, currentUrl)
            }
        }
    }

    private fun openConnection(
        url: String,
        builder: RequestBuilder,
    ): HttpURLConnection {
        val connection = URI(url).toURL().openConnection() as HttpURLConnection
        connection.requestMethod = builder.method.name
        connection.instanceFollowRedirects = false
        connection.connectTimeout = builder.connectTimeoutOverride ?: connectTimeout
        connection.readTimeout = builder.readTimeoutOverride ?: readTimeout

        if (!token.isNullOrBlank() && builder.useAuth) {
            connection.setRequestProperty("Authorization", "Bearer $token")
        }

        if (builder.body != null) {
            connection.setRequestProperty("Content-Type", "application/json")
            connection.doOutput = true
        }

        return connection
    }

    private fun writeBody(
        connection: HttpURLConnection,
        builder: RequestBuilder,
    ) {
        val body = builder.body ?: return
        connection.outputStream.use { it.write(body.toByteArray(StandardCharsets.UTF_8)) }
    }

    class RequestBuilder {
        var method: Method = Method.GET
        var path: String = ""
        var fullUrl: String? = null
        var body: String? = null
        var expectedStatus: Set<Int> = setOf(200)
        var useAuth: Boolean = true
        var connectTimeoutOverride: Int? = null
        var readTimeoutOverride: Int? = null
        internal val statusHandlers = mutableMapOf<Int, () -> ApiError>()

        fun onStatus(
            code: Int,
            handler: () -> ApiError,
        ) {
            statusHandlers[code] = handler
        }

        /** Serialize a value as JSON and set it as the request body. */
        fun <T> jsonBody(
            serializer: KSerializer<T>,
            value: T,
        ) {
            body = GlintJson.encodeToString(serializer, value)
        }
    }
}
