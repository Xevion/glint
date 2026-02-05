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

// ---------------------------------------------------------------------------
// TRACE
// ---------------------------------------------------------------------------

/** Lazy trace message — lambda is never invoked if TRACE is disabled. */
inline fun Logger.trace(message: () -> String) {
    if (isTraceEnabled) atTrace().log(message())
}

/** Trace with structured key-value pairs. */
inline fun Logger.trace(
    message: String,
    block: StructuredLog.() -> Unit,
) {
    if (isTraceEnabled) {
        val b = atTrace()
        val s = StructuredLog(b)
        s.block()
        b.log(message + s.suffix)
    }
}

/** Lazy trace message with structured key-value pairs. */
inline fun Logger.trace(
    message: () -> String,
    block: StructuredLog.() -> Unit,
) {
    if (isTraceEnabled) {
        val b = atTrace()
        val s = StructuredLog(b)
        s.block()
        b.log(message() + s.suffix)
    }
}

/** Trace with exception and static message. */
inline fun Logger.trace(
    cause: Throwable,
    message: String,
) {
    if (isTraceEnabled) atTrace().setCause(cause).log(message)
}

/** Trace with exception and lazy message. */
inline fun Logger.trace(
    cause: Throwable,
    message: () -> String,
) {
    if (isTraceEnabled) atTrace().setCause(cause).log(message())
}

/** Trace with exception, static message, and structured key-value pairs. */
inline fun Logger.trace(
    cause: Throwable,
    message: String,
    block: StructuredLog.() -> Unit,
) {
    if (isTraceEnabled) {
        val b = atTrace().setCause(cause)
        val s = StructuredLog(b)
        s.block()
        b.log(message + s.suffix)
    }
}

/** Trace with exception, lazy message, and structured key-value pairs. */
inline fun Logger.trace(
    cause: Throwable,
    message: () -> String,
    block: StructuredLog.() -> Unit,
) {
    if (isTraceEnabled) {
        val b = atTrace().setCause(cause)
        val s = StructuredLog(b)
        s.block()
        b.log(message() + s.suffix)
    }
}

// ---------------------------------------------------------------------------
// DEBUG
// ---------------------------------------------------------------------------

inline fun Logger.debug(message: () -> String) {
    if (isDebugEnabled) atDebug().log(message())
}

inline fun Logger.debug(
    message: String,
    block: StructuredLog.() -> Unit,
) {
    if (isDebugEnabled) {
        val b = atDebug()
        val s = StructuredLog(b)
        s.block()
        b.log(message + s.suffix)
    }
}

inline fun Logger.debug(
    message: () -> String,
    block: StructuredLog.() -> Unit,
) {
    if (isDebugEnabled) {
        val b = atDebug()
        val s = StructuredLog(b)
        s.block()
        b.log(message() + s.suffix)
    }
}

/** Debug with exception and static message. */
inline fun Logger.debug(
    cause: Throwable,
    message: String,
) {
    if (isDebugEnabled) atDebug().setCause(cause).log(message)
}

/** Debug with exception and lazy message. */
inline fun Logger.debug(
    cause: Throwable,
    message: () -> String,
) {
    if (isDebugEnabled) atDebug().setCause(cause).log(message())
}

/** Debug with exception, static message, and structured key-value pairs. */
inline fun Logger.debug(
    cause: Throwable,
    message: String,
    block: StructuredLog.() -> Unit,
) {
    if (isDebugEnabled) {
        val b = atDebug().setCause(cause)
        val s = StructuredLog(b)
        s.block()
        b.log(message + s.suffix)
    }
}

/** Debug with exception, lazy message, and structured key-value pairs. */
inline fun Logger.debug(
    cause: Throwable,
    message: () -> String,
    block: StructuredLog.() -> Unit,
) {
    if (isDebugEnabled) {
        val b = atDebug().setCause(cause)
        val s = StructuredLog(b)
        s.block()
        b.log(message() + s.suffix)
    }
}

// ---------------------------------------------------------------------------
// INFO
// ---------------------------------------------------------------------------

inline fun Logger.info(message: () -> String) {
    if (isInfoEnabled) atInfo().log(message())
}

inline fun Logger.info(
    message: String,
    block: StructuredLog.() -> Unit,
) {
    if (isInfoEnabled) {
        val b = atInfo()
        val s = StructuredLog(b)
        s.block()
        b.log(message + s.suffix)
    }
}

inline fun Logger.info(
    message: () -> String,
    block: StructuredLog.() -> Unit,
) {
    if (isInfoEnabled) {
        val b = atInfo()
        val s = StructuredLog(b)
        s.block()
        b.log(message() + s.suffix)
    }
}

// ---------------------------------------------------------------------------
// WARN
// ---------------------------------------------------------------------------

inline fun Logger.warn(message: () -> String) {
    if (isWarnEnabled) atWarn().log(message())
}

inline fun Logger.warn(
    message: String,
    block: StructuredLog.() -> Unit,
) {
    if (isWarnEnabled) {
        val b = atWarn()
        val s = StructuredLog(b)
        s.block()
        b.log(message + s.suffix)
    }
}

inline fun Logger.warn(
    message: () -> String,
    block: StructuredLog.() -> Unit,
) {
    if (isWarnEnabled) {
        val b = atWarn()
        val s = StructuredLog(b)
        s.block()
        b.log(message() + s.suffix)
    }
}

/** Warn with exception and static message. */
inline fun Logger.warn(
    cause: Throwable,
    message: String,
) {
    if (isWarnEnabled) atWarn().setCause(cause).log(message)
}

/** Warn with exception and lazy message. */
inline fun Logger.warn(
    cause: Throwable,
    message: () -> String,
) {
    if (isWarnEnabled) atWarn().setCause(cause).log(message())
}

/** Warn with exception, static message, and structured key-value pairs. */
inline fun Logger.warn(
    cause: Throwable,
    message: String,
    block: StructuredLog.() -> Unit,
) {
    if (isWarnEnabled) {
        val b = atWarn().setCause(cause)
        val s = StructuredLog(b)
        s.block()
        b.log(message + s.suffix)
    }
}

/** Warn with exception, lazy message, and structured key-value pairs. */
inline fun Logger.warn(
    cause: Throwable,
    message: () -> String,
    block: StructuredLog.() -> Unit,
) {
    if (isWarnEnabled) {
        val b = atWarn().setCause(cause)
        val s = StructuredLog(b)
        s.block()
        b.log(message() + s.suffix)
    }
}

// ---------------------------------------------------------------------------
// ERROR
// ---------------------------------------------------------------------------

inline fun Logger.error(message: () -> String) {
    if (isErrorEnabled) atError().log(message())
}

inline fun Logger.error(
    message: String,
    block: StructuredLog.() -> Unit,
) {
    if (isErrorEnabled) {
        val b = atError()
        val s = StructuredLog(b)
        s.block()
        b.log(message + s.suffix)
    }
}

inline fun Logger.error(
    message: () -> String,
    block: StructuredLog.() -> Unit,
) {
    if (isErrorEnabled) {
        val b = atError()
        val s = StructuredLog(b)
        s.block()
        b.log(message() + s.suffix)
    }
}

/** Error with exception and static message. */
inline fun Logger.error(
    cause: Throwable,
    message: String,
) {
    if (isErrorEnabled) atError().setCause(cause).log(message)
}

/** Error with exception and lazy message. */
inline fun Logger.error(
    cause: Throwable,
    message: () -> String,
) {
    if (isErrorEnabled) atError().setCause(cause).log(message())
}

/** Error with exception, static message, and structured key-value pairs. */
inline fun Logger.error(
    cause: Throwable,
    message: String,
    block: StructuredLog.() -> Unit,
) {
    if (isErrorEnabled) {
        val b = atError().setCause(cause)
        val s = StructuredLog(b)
        s.block()
        b.log(message + s.suffix)
    }
}

/** Error with exception, lazy message, and structured key-value pairs. */
inline fun Logger.error(
    cause: Throwable,
    message: () -> String,
    block: StructuredLog.() -> Unit,
) {
    if (isErrorEnabled) {
        val b = atError().setCause(cause)
        val s = StructuredLog(b)
        s.block()
        b.log(message() + s.suffix)
    }
}
