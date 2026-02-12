package com.xevion.glint.io

import java.io.File
import java.time.Instant
import java.time.format.DateTimeFormatter

/**
 * Manages session directory creation with collision resolution.
 * Provides unique directory names with optional counter suffix on collision.
 */
object SessionDirectoryManager {
    /**
     * Creates a unique session directory with timestamp-based naming.
     * Format: yyyy-MM-dd_HH-mm-ss
     * On collision: yyyy-MM-dd_HH-mm-ss_1, yyyy-MM-dd_HH-mm-ss_2, etc.
     *
     * @param parentDir Parent directory where session dir will be created
     * @param timestamp Instant to use for directory name (defaults to now)
     * @return Pair of (created directory, session ID string)
     */
    fun createSessionDirectory(
        parentDir: File,
        timestamp: Instant = Instant.now(),
    ): Pair<File, String> {
        val baseId =
            DateTimeFormatter
                .ofPattern("yyyy-MM-dd_HH-mm-ss")
                .withZone(java.time.ZoneId.systemDefault())
                .format(timestamp)

        var counter = 0
        var sessionId = baseId
        var sessionDir = File(parentDir, sessionId)

        // Find first available directory name
        while (sessionDir.exists()) {
            counter++
            sessionId = "${baseId}_$counter"
            sessionDir = File(parentDir, sessionId)
        }

        if (!sessionDir.mkdirs()) {
            error("Failed to create session directory: ${sessionDir.absolutePath}")
        }

        return Pair(sessionDir, sessionId)
    }
}
