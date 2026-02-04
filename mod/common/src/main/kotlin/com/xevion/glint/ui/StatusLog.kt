package com.xevion.glint.ui

import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.util.concurrent.ConcurrentLinkedDeque

/**
 * Session-scoped log for UI status messages.
 * Ring buffer capped at [MAX_ENTRIES], newest first.
 */
object StatusLog {
    private const val MAX_ENTRIES = 200
    private val entries = ConcurrentLinkedDeque<Entry>()
    private val timeFormatter = DateTimeFormatter.ofPattern("HH:mm:ss").withZone(ZoneId.systemDefault())

    enum class Level { INFO, WARNING, ERROR }

    data class Entry(
        val timestamp: Instant,
        val message: String,
        val level: Level,
    ) {
        fun formattedTime(): String = timeFormatter.format(timestamp)
    }

    fun log(
        message: String,
        level: Level = Level.INFO,
    ) {
        entries.addFirst(Entry(Instant.now(), message, level))
        while (entries.size > MAX_ENTRIES) {
            entries.removeLast()
        }
    }

    fun info(message: String) = log(message, Level.INFO)

    fun warn(message: String) = log(message, Level.WARNING)

    fun error(message: String) = log(message, Level.ERROR)

    fun entries(): List<Entry> = entries.toList()

    fun recent(n: Int): List<Entry> = entries.take(n)

    fun clear() = entries.clear()
}
