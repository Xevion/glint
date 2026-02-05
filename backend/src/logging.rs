//! Logging configuration with a compact timestamp formatter.
//!
//! Provides a less verbose log format compared to the default:
//! - Before: `2026-02-04T05:02:06.344416Z  INFO glint_backend: Message`
//! - After:  `05:02:06.34441  INFO glint_backend: Message`
//!
//! ## Verbosity Control
//!
//! Priority (highest first):
//! 1. `RUST_LOG` - Full tracing filter syntax, overrides everything
//! 2. `-v`/`-vv` flags - Sets app crate level (debug/trace)
//! 3. `LOG_LEVEL` env - Sets app crate level
//! 4. Default - `glint_*=info`, dependencies at `warn`

use std::{env, fmt};

use time::{format_description::FormatItem, macros::format_description};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{
    EnvFilter,
    fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields, format},
    registry::LookupSpan,
};

/// Timestamp format: `HH:MM:SS.sssss` (5 decimal places for subseconds)
const TIME_FORMAT: &[FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second].[subsecond digits:5]");

/// Custom formatter that outputs compact timestamps.
///
/// Format: `HH:MM:SS.sssss LEVEL target: message`
#[derive(Default)]
pub struct CompactFormatter;

impl<S, N> FormatEvent<S, N> for CompactFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Timestamp (dimmed)
        let now = time::OffsetDateTime::now_utc();
        write_dimmed(&mut writer, &now.format(TIME_FORMAT).unwrap_or_default())?;
        write!(writer, " ")?;

        // Level (colored, 5-char aligned)
        write_colored_level(&mut writer, *event.metadata().level())?;
        write!(writer, " ")?;

        // Span scope (if any)
        if let Some(scope) = ctx.event_scope() {
            let mut first = true;
            for span in scope.from_root() {
                if !first {
                    write_dimmed(&mut writer, ":")?;
                }
                write_bold(&mut writer, span.name())?;

                // Include span fields
                let ext = span.extensions();
                if let Some(fields) = ext.get::<FormattedFields<N>>()
                    && !fields.is_empty()
                {
                    write!(writer, "{{{fields}}}")?;
                }
                first = false;
            }
            if !first {
                write!(writer, " ")?;
            }
        }

        // Target (dimmed)
        write_dimmed(&mut writer, event.metadata().target())?;
        write_dimmed(&mut writer, ":")?;
        write!(writer, " ")?;

        // Event fields (the actual message)
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

fn write_dimmed(writer: &mut format::Writer<'_>, text: &str) -> fmt::Result {
    if writer.has_ansi_escapes() {
        write!(writer, "\x1b[2m{text}\x1b[0m")
    } else {
        write!(writer, "{text}")
    }
}

fn write_bold(writer: &mut format::Writer<'_>, text: &str) -> fmt::Result {
    if writer.has_ansi_escapes() {
        write!(writer, "\x1b[1m{text}\x1b[0m")
    } else {
        write!(writer, "{text}")
    }
}

fn write_colored_level(writer: &mut format::Writer<'_>, level: Level) -> fmt::Result {
    if writer.has_ansi_escapes() {
        let (color, name) = match level {
            Level::TRACE => ("\x1b[35m", "TRACE"), // magenta
            Level::DEBUG => ("\x1b[34m", "DEBUG"), // blue
            Level::INFO => ("\x1b[32m", " INFO"),  // green
            Level::WARN => ("\x1b[33m", " WARN"),  // yellow
            Level::ERROR => ("\x1b[31m", "ERROR"), // red
        };
        write!(writer, "{color}{name}\x1b[0m")
    } else {
        let name = match level {
            Level::TRACE => "TRACE",
            Level::DEBUG => "DEBUG",
            Level::INFO => " INFO",
            Level::WARN => " WARN",
            Level::ERROR => "ERROR",
        };
        write!(writer, "{name}")
    }
}

/// Initialize logging with the compact formatter.
///
/// Uses `RUST_LOG` environment variable for filtering, with a fallback default.
///
/// # Arguments
/// * `default_filter` - Filter used if `RUST_LOG` is not set (e.g., `"info,sqlx=warn"`)
pub fn init(default_filter: &str) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .event_format(CompactFormatter)
        .init();
}

/// Initialize logging with the compact formatter, returning an error if already initialized.
///
/// This is useful in tests or when multiple init attempts may occur.
pub fn try_init(default_filter: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .event_format(CompactFormatter)
        .try_init()
}

/// Initialize logging with verbosity level support.
///
/// Determines log level from (in priority order):
/// 1. `RUST_LOG` env - Full tracing filter, overrides everything
/// 2. `verbose_count` - 1 = debug, 2+ = trace (from `-v`/`-vv` flags)
/// 3. `LOG_LEVEL` env - Level name (debug, trace, etc.)
/// 4. Default - info
///
/// App crates (`glint_*`) use the determined level; all other crates are set to `warn`.
pub fn init_with_verbosity(verbose_count: u8) {
    // RUST_LOG takes absolute precedence
    if env::var("RUST_LOG").is_ok() {
        let filter = EnvFilter::from_default_env();
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .event_format(CompactFormatter)
            .init();
        return;
    }

    // Determine app log level: -v flags override LOG_LEVEL env
    let app_level = if verbose_count >= 2 {
        "trace"
    } else if verbose_count == 1 {
        "debug"
    } else {
        env::var("LOG_LEVEL")
            .ok()
            .map(|s| s.to_lowercase())
            .filter(|s| ["trace", "debug", "info", "warn", "error"].contains(&s.as_str()))
            .unwrap_or_else(|| "info".to_string())
            .leak()
    };

    // Build filter: app crates at determined level, deps at warn
    let filter_str = format!("warn,glint_backend={app_level}");
    let filter = EnvFilter::new(filter_str);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .event_format(CompactFormatter)
        .init();
}
