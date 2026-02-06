package com.xevion.glint

import org.apache.logging.log4j.Level
import org.slf4j.LoggerFactory

/**
 * Type-safe logging categories for the Glint mod.
 *
 * Each entry maps to a named SLF4J logger, configured with its own Log4j2 logger context
 * so categories can be independently filtered and routed to different appenders.
 *
 * Usage:
 * ```
 * private val log = Loggers.Capture.get()
 * log.info("Screenshot saved") {
 *     "shader" to shaderName
 *     "resolution" to "${width}x${height}"
 * }
 * ```
 */
enum class Loggers(
    val category: String,
) {
    Api("api"),
    Capture("capture"),
    Download("download"),
    Input("input"),
    Io("io"),
    Mixin("mixin"),
    Orchestration("orchestration"),
    Scene("scene"),
    Session("session"),
    Ui("ui"),
    ;

    fun get(): GlintLogger = GlintLogger(LoggerFactory.getLogger(category))

    companion object {
        /**
         * Ordered from most to least verbose, for use in UI level selectors.
         */
        val LEVELS: List<Level> =
            listOf(Level.TRACE, Level.DEBUG, Level.INFO, Level.WARN, Level.ERROR, Level.OFF)

        init {
            LogConfig.configure()
        }

        @JvmStatic
        fun getGlobalLevel(): Level = LogConfig.getGlobalLevel()

        @JvmStatic
        fun setGlobalLevel(level: Level) = LogConfig.setGlobalLevel(level)

        @JvmStatic
        fun getAllCategories(): Set<String> = entries.map { it.category }.toSet()

        @JvmStatic
        fun fromCategory(category: String): Loggers? = entries.find { it.category == category }

        @JvmStatic
        fun hasCategory(category: String): Boolean = entries.any { it.category == category }
    }
}
