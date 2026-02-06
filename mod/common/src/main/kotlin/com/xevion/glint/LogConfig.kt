package com.xevion.glint

import org.apache.logging.log4j.Level
import org.apache.logging.log4j.LogManager
import org.apache.logging.log4j.core.LoggerContext
import org.apache.logging.log4j.core.appender.ConsoleAppender
import org.apache.logging.log4j.core.config.LoggerConfig
import org.apache.logging.log4j.core.layout.PatternLayout

/**
 * Programmatic Log4j2 configuration for all Glint logging categories.
 *
 * Called automatically from [Loggers] companion `init` block on first access.
 * Creates a dedicated console appender and attaches it to every category logger
 * so structured output (including key-value pairs from the SLF4J fluent API)
 * is visible during development.
 */
internal object LogConfig {
    private var configured = false
    private var globalLevel = Level.DEBUG

    private const val APPENDER_NAME = "GlintConsole"
    private const val PATTERN =
        "%style{[%d{HH:mm:ss.SSS}]}{blue} " +
            "%highlight{[%-5level]}{FATAL=red, ERROR=red, WARN=yellow, INFO=green, DEBUG=green, TRACE=blue} " +
            "%style{[%logger{1}]}{cyan} " +
            "%highlight{%msg%n}{FATAL=red, ERROR=red, WARN=normal, INFO=normal, DEBUG=normal, TRACE=normal}"

    @Synchronized
    fun configure() {
        if (configured) return
        configured = true

        runCatching {
            val ctx = LogManager.getContext(false) as LoggerContext
            val config = ctx.configuration
            val appender = getOrCreateAppender(config)

            // Configure each category logger + the mod-level "glint" logger
            val allLoggers = Loggers.getAllCategories() + Glint.MOD_ID
            for (name in allLoggers) {
                val loggerConfig = getOrCreateLoggerConfig(config, name)
                loggerConfig.level = globalLevel
                loggerConfig.addAppender(appender, Level.ALL, null)
            }

            ctx.updateLoggers()
        }.onFailure { e ->
            System.err.println("Failed to configure Glint loggers: ${e.message}")
        }
    }

    @Synchronized
    fun getGlobalLevel(): Level = globalLevel

    @Synchronized
    fun setGlobalLevel(level: Level) {
        globalLevel = level

        runCatching {
            val ctx = LogManager.getContext(false) as LoggerContext
            val config = ctx.configuration

            val allLoggers = Loggers.getAllCategories() + Glint.MOD_ID
            for (name in allLoggers) {
                config.getLoggerConfig(name).takeIf { it.name == name }?.level = level
            }

            ctx.updateLoggers()
        }.onFailure { e ->
            System.err.println("Failed to set Glint log level: ${e.message}")
        }
    }

    private fun getOrCreateAppender(config: org.apache.logging.log4j.core.config.Configuration): ConsoleAppender {
        val existing = config.appenders[APPENDER_NAME]
        if (existing is ConsoleAppender) return existing

        val appender =
            ConsoleAppender
                .newBuilder()
                .setName(APPENDER_NAME)
                .setTarget(ConsoleAppender.Target.SYSTEM_OUT)
                .setFollow(true)
                .setLayout(
                    PatternLayout
                        .newBuilder()
                        .withPattern(PATTERN)
                        .withConfiguration(config)
                        .build(),
                ).build()
                .also { it.start() }

        config.addAppender(appender)
        return appender
    }

    private fun getOrCreateLoggerConfig(
        config: org.apache.logging.log4j.core.config.Configuration,
        name: String,
    ): LoggerConfig {
        val existing = config.getLoggerConfig(name)
        return if (existing.name == name) {
            existing
        } else {
            LoggerConfig(name, globalLevel, false).also {
                config.addLogger(name, it)
            }
        }
    }
}
