package com.xevion.glint.upload

import java.io.IOException

data class UploadProgress(
    val bytesProcessed: Long,
    val totalBytes: Long,
    val state: State,
    val errorMessage: String? = null,
) {
    enum class State {
        SAVING,
        PACKAGING,
        HASHING,
        UPLOADING,
        FINALIZING,
        COMPLETE,
        FAILED,
    }

    val percentComplete: Int
        get() = if (totalBytes > 0) ((bytesProcessed.toDouble() / totalBytes) * 100).toInt() else 0

    val bytesRemaining: Long
        get() = (totalBytes - bytesProcessed).coerceAtLeast(0)

    companion object {
        fun saving() = UploadProgress(0, 0, State.SAVING)

        fun packaging(
            bytesProcessed: Long,
            totalBytes: Long,
        ) = UploadProgress(bytesProcessed, totalBytes, State.PACKAGING)

        fun hashing() = UploadProgress(0, 0, State.HASHING)

        fun uploading(
            bytesProcessed: Long,
            totalBytes: Long,
        ) = UploadProgress(bytesProcessed, totalBytes, State.UPLOADING)

        fun finalizing() = UploadProgress(0, 0, State.FINALIZING)

        fun complete() = UploadProgress(0, 0, State.COMPLETE)

        fun failed(errorMessage: String? = null) = UploadProgress(0, 0, State.FAILED, errorMessage)
    }
}

sealed class UploadException(
    message: String,
    cause: Throwable? = null,
) : IOException(message, cause) {
    class SaveFailed(
        cause: Throwable,
    ) : UploadException("Failed to force-save world: ${cause.message}", cause)

    class PackagingFailed(
        cause: Throwable,
    ) : UploadException("Failed to create world ZIP: ${cause.message}", cause)

    class FileTooLarge(
        sizeBytes: Long,
        maxBytes: Long,
    ) : UploadException("World file too large: $sizeBytes bytes exceeds limit of $maxBytes bytes")

    class NetworkError(
        url: String,
        cause: Throwable,
    ) : UploadException("Failed to connect to $url: ${cause.message}", cause)

    class HttpError(
        statusCode: Int,
        responseBody: String?,
    ) : UploadException("HTTP $statusCode error${responseBody?.let { ": $it" } ?: ""}")

    class UploadInterrupted(
        message: String,
        cause: Throwable? = null,
    ) : UploadException(message, cause)

    class FinalizationFailed(
        message: String,
    ) : UploadException("Failed to finalize upload: $message")
}
