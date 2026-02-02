package com.xevion.glint.download

/**
 * Tracks download progress for world downloads.
 */
data class DownloadProgress(
    val bytesDownloaded: Long,
    val totalBytes: Long,
    val state: State,
    val errorMessage: String? = null,
) {
    enum class State {
        DOWNLOADING,
        EXTRACTING,
        COMPLETE,
        FAILED,
    }

    val percentComplete: Int
        get() =
            if (totalBytes > 0) {
                ((bytesDownloaded.toDouble() / totalBytes) * 100).toInt()
            } else {
                0
            }

    val bytesRemaining: Long
        get() = (totalBytes - bytesDownloaded).coerceAtLeast(0)

    companion object {
        fun downloading(
            bytesDownloaded: Long,
            totalBytes: Long,
        ) = DownloadProgress(bytesDownloaded, totalBytes, State.DOWNLOADING)

        fun extracting() = DownloadProgress(0, 0, State.EXTRACTING)

        fun complete() = DownloadProgress(0, 0, State.COMPLETE)

        fun failed(errorMessage: String? = null) = DownloadProgress(0, 0, State.FAILED, errorMessage)
    }
}
