package com.xevion.glint.api

import java.net.ConnectException
import java.net.SocketTimeoutException
import java.net.UnknownHostException

/**
 * Represents errors that can occur during API operations.
 */
sealed class ApiError : Exception() {
    abstract override val message: String
    abstract val userMessage: String

    data class NetworkError(
        override val message: String,
        override val cause: Throwable?,
    ) : ApiError() {
        override val userMessage: String
            get() =
                when (cause) {
                    is ConnectException -> "Cannot connect to backend server. Is it running?"
                    is UnknownHostException -> "Cannot resolve backend hostname. Check your API URL."
                    is SocketTimeoutException -> "Connection to backend timed out. Server may be overloaded."
                    else -> "Network error: ${cause?.message ?: message}"
                }
    }

    data class HttpError(
        val statusCode: Int,
        val responseBody: String?,
    ) : ApiError() {
        override val message: String = "HTTP $statusCode: $responseBody"
        override val userMessage: String
            get() =
                when (statusCode) {
                    400 -> "Bad request: Invalid data sent to backend"
                    401 -> "Unauthorized: Check your API credentials"
                    403 -> "Forbidden: You don't have permission for this operation"
                    404 -> "Not found: Resource doesn't exist on backend"
                    409 -> "Conflict: Resource already exists"
                    500 -> "Backend server error: Please check server logs"
                    502, 503, 504 -> "Backend server unavailable: Try again later"
                    else -> "HTTP error $statusCode: $responseBody"
                }
    }

    data class ParseError(
        override val message: String,
        override val cause: Throwable?,
    ) : ApiError() {
        override val userMessage: String = "Failed to parse backend response: ${cause?.message ?: message}"
    }

    data class ConfigError(
        override val message: String,
    ) : ApiError() {
        override val userMessage: String = message
    }

    data class UnknownError(
        override val message: String,
        override val cause: Throwable?,
    ) : ApiError() {
        override val userMessage: String = "Unknown error: ${cause?.message ?: message}"
    }

    companion object {
        fun fromException(e: Exception): ApiError = when (e) {
            is ConnectException, is UnknownHostException, is SocketTimeoutException -> {
                NetworkError(e.message ?: "Network error", e)
            }

            else -> {
                UnknownError(e.message ?: "Unknown error", e)
            }
        }
    }
}
