@file:Suppress("NOTHING_TO_INLINE")

package com.xevion.glint

import org.slf4j.Logger
import org.slf4j.spi.LoggingEventBuilder

/**
 * DSL scope for structured log entries. Adds key-value pairs to the underlying SLF4J fluent event
 * and formats them into the message string for console rendering.
 *
 * ```
 * log.info("Request complete") {
 *     "method" to "GET"
 *     "status" to 200
 *     "duration_ms" to elapsed
 * }
 * ```
 *
 * The [to] function inside this scope calls [LoggingEventBuilder.addKeyValue] for structured
 * consumers (JSON appenders, log aggregators) AND appends `key=value` to the message suffix
 * for guaranteed console rendering. The stdlib `to` is shadowed within this receiver.
 *
 * Values containing spaces, `=`, or `"` are automatically quoted.
 */
class StructuredLog(
    @PublishedApi internal val builder: LoggingEventBuilder,
) {
    @PublishedApi internal val suffix = StringBuilder()

    inline infix fun String.to(value: Any?) {
        builder.addKeyValue(this, value)
        val str = value.toString()
        suffix.append(' ').append(this).append('=')
        if (str.contains(' ') || str.contains('=') || str.contains('"')) {
            suffix.append('"').append(str.replace("\"", "\\\"")).append('"')
        } else {
            suffix.append(str)
        }
    }
}

/**
 * Wrapper around SLF4J [Logger] that provides structured logging methods as member functions.
 *
 * This avoids Kotlin's overload resolution issue where SLF4J's member methods like
 * `debug(String, Object)` take precedence over extension functions, causing structured
 * blocks to be silently discarded.
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
@JvmInline
value class GlintLogger(
    @PublishedApi internal val delegate: Logger,
) {
    /** The underlying SLF4J logger name. */
    inline val name: String get() = delegate.name

    /** Trace with a plain string message. */
    inline fun trace(message: String) {
        if (delegate.isTraceEnabled) delegate.atTrace().log(message)
    }

    /** Trace with SLF4J-style format placeholders. */
    inline fun trace(
        message: String,
        vararg args: Any?,
    ) {
        if (delegate.isTraceEnabled) delegate.atTrace().log(message, *args)
    }

    /** Lazy trace message — lambda is never invoked if TRACE is disabled. */
    inline fun trace(message: () -> String) {
        if (delegate.isTraceEnabled) delegate.atTrace().log(message())
    }

    /** Trace with structured key-value pairs. */
    inline fun trace(
        message: String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isTraceEnabled) {
            val b = delegate.atTrace()
            val s = StructuredLog(b)
            s.block()
            b.log(message + s.suffix)
        }
    }

    /** Lazy trace message with structured key-value pairs. */
    inline fun trace(
        message: () -> String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isTraceEnabled) {
            val b = delegate.atTrace()
            val s = StructuredLog(b)
            s.block()
            b.log(message() + s.suffix)
        }
    }

    /** Trace with exception and static message. */
    inline fun trace(
        cause: Throwable,
        message: String,
    ) {
        if (delegate.isTraceEnabled) delegate.atTrace().setCause(cause).log(message)
    }

    /** Trace with exception and lazy message. */
    inline fun trace(
        cause: Throwable,
        message: () -> String,
    ) {
        if (delegate.isTraceEnabled) delegate.atTrace().setCause(cause).log(message())
    }

    /** Trace with exception, static message, and structured key-value pairs. */
    inline fun trace(
        cause: Throwable,
        message: String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isTraceEnabled) {
            val b = delegate.atTrace().setCause(cause)
            val s = StructuredLog(b)
            s.block()
            b.log(message + s.suffix)
        }
    }

    /** Trace with exception, lazy message, and structured key-value pairs. */
    inline fun trace(
        cause: Throwable,
        message: () -> String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isTraceEnabled) {
            val b = delegate.atTrace().setCause(cause)
            val s = StructuredLog(b)
            s.block()
            b.log(message() + s.suffix)
        }
    }

    /** Debug with a plain string message. */
    inline fun debug(message: String) {
        if (delegate.isDebugEnabled) delegate.atDebug().log(message)
    }

    /** Debug with SLF4J-style format placeholders. */
    inline fun debug(
        message: String,
        vararg args: Any?,
    ) {
        if (delegate.isDebugEnabled) delegate.atDebug().log(message, *args)
    }

    /** Lazy debug message — lambda is never invoked if DEBUG is disabled. */
    inline fun debug(message: () -> String) {
        if (delegate.isDebugEnabled) delegate.atDebug().log(message())
    }

    /** Debug with structured key-value pairs. */
    inline fun debug(
        message: String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isDebugEnabled) {
            val b = delegate.atDebug()
            val s = StructuredLog(b)
            s.block()
            b.log(message + s.suffix)
        }
    }

    /** Lazy debug message with structured key-value pairs. */
    inline fun debug(
        message: () -> String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isDebugEnabled) {
            val b = delegate.atDebug()
            val s = StructuredLog(b)
            s.block()
            b.log(message() + s.suffix)
        }
    }

    /** Debug with exception and static message. */
    inline fun debug(
        cause: Throwable,
        message: String,
    ) {
        if (delegate.isDebugEnabled) delegate.atDebug().setCause(cause).log(message)
    }

    /** Debug with exception and lazy message. */
    inline fun debug(
        cause: Throwable,
        message: () -> String,
    ) {
        if (delegate.isDebugEnabled) delegate.atDebug().setCause(cause).log(message())
    }

    /** Debug with exception, static message, and structured key-value pairs. */
    inline fun debug(
        cause: Throwable,
        message: String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isDebugEnabled) {
            val b = delegate.atDebug().setCause(cause)
            val s = StructuredLog(b)
            s.block()
            b.log(message + s.suffix)
        }
    }

    /** Debug with exception, lazy message, and structured key-value pairs. */
    inline fun debug(
        cause: Throwable,
        message: () -> String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isDebugEnabled) {
            val b = delegate.atDebug().setCause(cause)
            val s = StructuredLog(b)
            s.block()
            b.log(message() + s.suffix)
        }
    }

    /** Info with a plain string message. */
    inline fun info(message: String) {
        if (delegate.isInfoEnabled) delegate.atInfo().log(message)
    }

    /** Info with SLF4J-style format placeholders. */
    inline fun info(
        message: String,
        vararg args: Any?,
    ) {
        if (delegate.isInfoEnabled) delegate.atInfo().log(message, *args)
    }

    /** Lazy info message — lambda is never invoked if INFO is disabled. */
    inline fun info(message: () -> String) {
        if (delegate.isInfoEnabled) delegate.atInfo().log(message())
    }

    /** Info with structured key-value pairs. */
    inline fun info(
        message: String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isInfoEnabled) {
            val b = delegate.atInfo()
            val s = StructuredLog(b)
            s.block()
            b.log(message + s.suffix)
        }
    }

    /** Lazy info message with structured key-value pairs. */
    inline fun info(
        message: () -> String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isInfoEnabled) {
            val b = delegate.atInfo()
            val s = StructuredLog(b)
            s.block()
            b.log(message() + s.suffix)
        }
    }

    /** Info with exception and static message. */
    inline fun info(
        cause: Throwable,
        message: String,
    ) {
        if (delegate.isInfoEnabled) delegate.atInfo().setCause(cause).log(message)
    }

    /** Info with exception and lazy message. */
    inline fun info(
        cause: Throwable,
        message: () -> String,
    ) {
        if (delegate.isInfoEnabled) delegate.atInfo().setCause(cause).log(message())
    }

    /** Info with exception, static message, and structured key-value pairs. */
    inline fun info(
        cause: Throwable,
        message: String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isInfoEnabled) {
            val b = delegate.atInfo().setCause(cause)
            val s = StructuredLog(b)
            s.block()
            b.log(message + s.suffix)
        }
    }

    /** Info with exception, lazy message, and structured key-value pairs. */
    inline fun info(
        cause: Throwable,
        message: () -> String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isInfoEnabled) {
            val b = delegate.atInfo().setCause(cause)
            val s = StructuredLog(b)
            s.block()
            b.log(message() + s.suffix)
        }
    }

    /** Warn with a plain string message. */
    inline fun warn(message: String) {
        if (delegate.isWarnEnabled) delegate.atWarn().log(message)
    }

    /** Warn with SLF4J-style format placeholders. */
    inline fun warn(
        message: String,
        vararg args: Any?,
    ) {
        if (delegate.isWarnEnabled) delegate.atWarn().log(message, *args)
    }

    /** Lazy warn message — lambda is never invoked if WARN is disabled. */
    inline fun warn(message: () -> String) {
        if (delegate.isWarnEnabled) delegate.atWarn().log(message())
    }

    /** Warn with structured key-value pairs. */
    inline fun warn(
        message: String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isWarnEnabled) {
            val b = delegate.atWarn()
            val s = StructuredLog(b)
            s.block()
            b.log(message + s.suffix)
        }
    }

    /** Lazy warn message with structured key-value pairs. */
    inline fun warn(
        message: () -> String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isWarnEnabled) {
            val b = delegate.atWarn()
            val s = StructuredLog(b)
            s.block()
            b.log(message() + s.suffix)
        }
    }

    /** Warn with exception and static message. */
    inline fun warn(
        cause: Throwable,
        message: String,
    ) {
        if (delegate.isWarnEnabled) delegate.atWarn().setCause(cause).log(message)
    }

    /** Warn with exception and lazy message. */
    inline fun warn(
        cause: Throwable,
        message: () -> String,
    ) {
        if (delegate.isWarnEnabled) delegate.atWarn().setCause(cause).log(message())
    }

    /** Warn with exception, static message, and structured key-value pairs. */
    inline fun warn(
        cause: Throwable,
        message: String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isWarnEnabled) {
            val b = delegate.atWarn().setCause(cause)
            val s = StructuredLog(b)
            s.block()
            b.log(message + s.suffix)
        }
    }

    /** Warn with exception, lazy message, and structured key-value pairs. */
    inline fun warn(
        cause: Throwable,
        message: () -> String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isWarnEnabled) {
            val b = delegate.atWarn().setCause(cause)
            val s = StructuredLog(b)
            s.block()
            b.log(message() + s.suffix)
        }
    }

    /** Error with a plain string message. */
    inline fun error(message: String) {
        if (delegate.isErrorEnabled) delegate.atError().log(message)
    }

    /** Error with SLF4J-style format placeholders. */
    inline fun error(
        message: String,
        vararg args: Any?,
    ) {
        if (delegate.isErrorEnabled) delegate.atError().log(message, *args)
    }

    /** Lazy error message — lambda is never invoked if ERROR is disabled. */
    inline fun error(message: () -> String) {
        if (delegate.isErrorEnabled) delegate.atError().log(message())
    }

    /** Error with structured key-value pairs. */
    inline fun error(
        message: String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isErrorEnabled) {
            val b = delegate.atError()
            val s = StructuredLog(b)
            s.block()
            b.log(message + s.suffix)
        }
    }

    /** Lazy error message with structured key-value pairs. */
    inline fun error(
        message: () -> String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isErrorEnabled) {
            val b = delegate.atError()
            val s = StructuredLog(b)
            s.block()
            b.log(message() + s.suffix)
        }
    }

    /** Error with exception and static message. */
    inline fun error(
        cause: Throwable,
        message: String,
    ) {
        if (delegate.isErrorEnabled) delegate.atError().setCause(cause).log(message)
    }

    /** Error with exception and lazy message. */
    inline fun error(
        cause: Throwable,
        message: () -> String,
    ) {
        if (delegate.isErrorEnabled) delegate.atError().setCause(cause).log(message())
    }

    /** Error with exception, static message, and structured key-value pairs. */
    inline fun error(
        cause: Throwable,
        message: String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isErrorEnabled) {
            val b = delegate.atError().setCause(cause)
            val s = StructuredLog(b)
            s.block()
            b.log(message + s.suffix)
        }
    }

    /** Error with exception, lazy message, and structured key-value pairs. */
    inline fun error(
        cause: Throwable,
        message: () -> String,
        block: StructuredLog.() -> Unit,
    ) {
        if (delegate.isErrorEnabled) {
            val b = delegate.atError().setCause(cause)
            val s = StructuredLog(b)
            s.block()
            b.log(message() + s.suffix)
        }
    }
}
