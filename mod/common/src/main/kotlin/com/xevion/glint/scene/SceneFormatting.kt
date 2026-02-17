package com.xevion.glint.scene

/** Shared formatting utilities for Minecraft time-of-day and moon phase display. */
object SceneFormatting {
    /** Minecraft tick-to-clock conversion. Tick 0 = 6:00 AM. Each 1000 ticks = 1 hour. */
    fun formatTime(ticks: Int): String {
        val normalizedTicks = ((ticks % 24000) + 24000) % 24000
        val hour = (normalizedTicks / 1000 + 6) % 24
        val minute = (normalizedTicks % 1000) * 60 / 1000
        val amPm = if (hour < 12) "AM" else "PM"
        val displayHour =
            when {
                hour == 0 -> 12
                hour > 12 -> hour - 12
                else -> hour
            }
        val timeStr = "$displayHour:${minute.toString().padStart(2, '0')} $amPm"
        val label = timeLabel(hour)
        return "$timeStr ($label)"
    }

    private fun timeLabel(hour: Int): String =
        when (hour) {
            in 4..5 -> "Dawn"
            in 6..9 -> "Morning"
            in 10..13 -> "Noon"
            in 14..17 -> "Afternoon"
            in 18..19 -> "Dusk"
            else -> "Night"
        }

    /** Moon phase names matching Minecraft's 8-phase cycle (0-7). */
    fun moonPhaseName(phase: Int): String =
        when (phase) {
            0 -> "Full Moon"
            1 -> "Waning Gibbous"
            2 -> "Third Quarter"
            3 -> "Waning Crescent"
            4 -> "New Moon"
            5 -> "Waxing Crescent"
            6 -> "First Quarter"
            7 -> "Waxing Gibbous"
            else -> "Unknown"
        }
}
