package com.xevion.glint.download

import java.io.IOException

/**
 * Base exception for world download failures.
 */
sealed class WorldDownloadException(
    message: String,
    cause: Throwable? = null,
) : IOException(message, cause) {
    /**
     * Network connection failed (SSL, timeout, DNS, etc.)
     */
    class NetworkError(
        url: String,
        cause: Throwable,
    ) : WorldDownloadException(
            "Failed to connect to $url: ${cause.message}",
            cause,
        )

    /**
     * Download interrupted or incomplete.
     */
    class DownloadInterrupted(
        message: String,
        cause: Throwable? = null,
    ) : WorldDownloadException(message, cause)

    /**
     * File hash verification failed.
     */
    class HashMismatch(
        expected: String,
        actual: String,
    ) : WorldDownloadException(
            "World file corrupted: expected hash $expected but got $actual",
        )

    /**
     * ZIP extraction failed.
     */
    class ExtractionFailed(
        cause: Throwable,
    ) : WorldDownloadException("Failed to extract world file: ${cause.message}", cause)

    /**
     * HTTP error response (4xx, 5xx).
     */
    class HttpError(
        statusCode: Int,
        url: String,
    ) : WorldDownloadException("HTTP $statusCode error downloading from $url")
}
