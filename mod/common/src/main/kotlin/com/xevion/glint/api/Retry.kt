package com.xevion.glint.api

import com.xevion.glint.Loggers

private val log = Loggers.Core.get()

/**
 * Retries an API call with exponential backoff when rate limited.
 *
 * On [ApiError.RateLimited], sleeps for the server-specified duration
 * (with a minimum of [minBackoffMs] and exponential growth on repeated retries),
 * then retries up to [maxRetries] times.
 *
 * Non-rate-limit failures are returned immediately without retrying.
 */
fun <T> retryOnRateLimit(
    maxRetries: Int = 3,
    minBackoffMs: Long = 1000,
    operationName: String = "API call",
    block: () -> Result<T>,
): Result<T> {
    var lastResult: Result<T>? = null

    for (attempt in 0..maxRetries) {
        val result = block()

        if (result.isSuccess) return result

        val error = result.exceptionOrNull()
        if (error !is ApiError.RateLimited) return result

        lastResult = result

        if (attempt < maxRetries) {
            val serverDelay = error.retryAfterSeconds * 1000
            val backoff = minBackoffMs * (1L shl attempt)
            val sleepMs = maxOf(serverDelay, backoff)
            log.warn("Rate limited, retrying") {
                "operation" to operationName
                "attempt" to "${attempt + 1}/$maxRetries"
                "sleep_ms" to sleepMs
            }
            Thread.sleep(sleepMs)
        }
    }

    return lastResult!!
}
