package com.xevion.glint

import org.apache.logging.log4j.Level
import org.apache.logging.log4j.LogManager
import org.apache.logging.log4j.core.LoggerContext
import org.apache.logging.log4j.core.appender.ConsoleAppender
import org.apache.logging.log4j.core.config.LoggerConfig
import org.apache.logging.log4j.core.layout.PatternLayout

/**
 * Configures debug-level logging for Glint mod.
 * Isolated from main mod initialization to reduce complexity.
 */
internal object LogConfig {
    fun setupDebugLogging(modId: String) {
        runCatching {
            val ctx = LogManager.getContext(false) as LoggerContext
            val config = ctx.configuration

            val appenderName = "GlintDebugConsole"
            val appender = config.getAppender(appenderName) ?: createAppender(appenderName, config)

            val loggerConfig = getOrCreateLoggerConfig(config, modId)
            loggerConfig.level = Level.DEBUG
            loggerConfig.addAppender(appender, Level.DEBUG, null)

            ctx.updateLoggers()
        }.onFailure { e ->
            System.err.println("Failed to configure Glint debug logger: ${e.message}")
        }
    }

    private fun createAppender(
        name: String,
        config: org.apache.logging.log4j.core.config.Configuration,
    ): ConsoleAppender {
        val appender =
            ConsoleAppender
                .newBuilder()
                .setName(name)
                .setTarget(ConsoleAppender.Target.SYSTEM_OUT)
                .setFollow(true)
                .setLayout(
                    PatternLayout
                        .newBuilder()
                        .withPattern("[%d{HH:mm:ss}] [%t/%level] (%logger{1}) %msg%n")
                        .withConfiguration(config)
                        .build(),
                ).build()
                .also { it.start() }

        config.addAppender(appender)
        return appender
    }

    private fun getOrCreateLoggerConfig(
        config: org.apache.logging.log4j.core.config.Configuration,
        modId: String,
    ): LoggerConfig {
        val existing = config.getLoggerConfig(modId)
        return if (existing.name == modId) {
            existing
        } else {
            val newConfig = LoggerConfig(modId, Level.DEBUG, true)
            config.addLogger(modId, newConfig)
            newConfig
        }
    }
}
