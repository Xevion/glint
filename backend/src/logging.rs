//! Logging configuration with dual-format output (pretty + JSON).
//!
//! ## Output Formats
//!
//! - **Pretty** (default): `HH:MM:SS.sssss LEVEL span{fields} target: message`
//! - **JSON** (`LOG_JSON=true`): Machine-readable structured JSON per line
//!
//! ## Verbosity Control
//!
//! Priority (highest first):
//! 1. `RUST_LOG` - Full tracing filter syntax, overrides everything
//! 2. `-v`/`-vv` flags - Sets app crate level (debug/trace)
//! 3. `LOG_LEVEL` env - Sets app crate level
//! 4. Default - `glint=debug` (dev) / `glint=info` (release), dependencies at `warn`

use std::{env, fmt};

use serde::Serialize;
use serde_json::{Map, Value};
use time::{format_description::FormatItem, macros::format_description};
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{
    EnvFilter,
    fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields, format},
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
};

/// Timestamp format: `HH:MM:SS.sssss` (5 decimal places for subseconds)
const TIME_FORMAT: &[FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second].[subsecond digits:5]");

// ── Pretty Formatter ──────────────────────────────────────────────

/// Custom formatter that outputs compact timestamps with colored levels and spans.
///
/// Format: `HH:MM:SS.sssss LEVEL span{fields}:span2 target: message`
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

// ── JSON Formatter ────────────────────────────────────────────────

/// Machine-readable JSON formatter for production/log aggregation.
///
/// Each log event is a single JSON line with timestamp, message, level,
/// target, and all structured fields (including span fields) flattened.
pub struct JsonFormatter;

impl<S, N> FormatEvent<S, N> for JsonFormatter
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
        let meta = event.metadata();

        #[derive(Serialize)]
        struct EventFields {
            timestamp: String,
            message: String,
            level: String,
            target: String,
            #[serde(flatten)]
            fields: Map<String, Value>,
        }

        let (message, fields) = {
            let mut message: Option<String> = None;
            let mut fields: Map<String, Value> = Map::new();

            struct FieldVisitor<'a> {
                message: &'a mut Option<String>,
                fields: &'a mut Map<String, Value>,
            }

            impl Visit for FieldVisitor<'_> {
                fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                    let key = field.name();
                    if key == "message" {
                        *self.message = Some(format!("{value:?}"));
                    } else {
                        self.fields
                            .insert(key.to_string(), Value::String(format!("{value:?}")));
                    }
                }

                fn record_str(&mut self, field: &Field, value: &str) {
                    let key = field.name();
                    if key == "message" {
                        *self.message = Some(value.to_string());
                    } else {
                        self.fields
                            .insert(key.to_string(), Value::String(value.to_string()));
                    }
                }

                fn record_i64(&mut self, field: &Field, value: i64) {
                    let key = field.name();
                    if key != "message" {
                        self.fields.insert(
                            key.to_string(),
                            Value::Number(serde_json::Number::from(value)),
                        );
                    }
                }

                fn record_u64(&mut self, field: &Field, value: u64) {
                    let key = field.name();
                    if key != "message" {
                        self.fields.insert(
                            key.to_string(),
                            Value::Number(serde_json::Number::from(value)),
                        );
                    }
                }

                fn record_bool(&mut self, field: &Field, value: bool) {
                    let key = field.name();
                    if key != "message" {
                        self.fields.insert(key.to_string(), Value::Bool(value));
                    }
                }
            }

            let mut visitor = FieldVisitor {
                message: &mut message,
                fields: &mut fields,
            };
            event.record(&mut visitor);

            // Extract fields from parent spans
            if let Some(scope) = ctx.event_scope() {
                for span in scope.from_root() {
                    let ext = span.extensions();
                    if let Some(formatted_fields) = ext.get::<FormattedFields<N>>() {
                        let field_str = formatted_fields.fields.as_str();
                        for pair in field_str.split_whitespace() {
                            if let Some((key, value)) = pair.split_once('=') {
                                let value = value.trim_matches('"').trim_matches('\'');
                                fields.insert(key.to_string(), Value::String(value.to_string()));
                            }
                        }
                    }
                }
            }

            (message, fields)
        };

        let json = EventFields {
            timestamp: time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z")),
            message: message.unwrap_or_default(),
            level: meta.level().to_string().to_lowercase(),
            target: meta.target().to_string(),
            fields,
        };

        writeln!(
            writer,
            "{}",
            serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
        )
    }
}

// ── ANSI Helpers ──────────────────────────────────────────────────

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

// ── Initialization ───────────────────────────────────────────────

/// Default log level based on build profile.
fn default_level() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
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

    init_with_filter(filter);
}

/// Initialize logging with the compact formatter, returning an error if already initialized.
///
/// This is useful in tests or when multiple init attempts may occur.
pub fn try_init(default_filter: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    try_init_with_filter(filter)
}

/// Initialize logging with verbosity level support.
///
/// Determines log level from (in priority order):
/// 1. `RUST_LOG` env - Full tracing filter, overrides everything
/// 2. `verbose_count` - 1 = debug, 2+ = trace (from `-v`/`-vv` flags)
/// 3. `LOG_LEVEL` env - Level name (debug, trace, etc.)
/// 4. Default - debug (dev) / info (release)
///
/// App crates (`glint*`) use the determined level; all other crates are set to `warn`.
pub fn init_with_verbosity(verbose_count: u8) {
    // RUST_LOG takes absolute precedence
    if env::var("RUST_LOG").is_ok() {
        let filter = EnvFilter::from_default_env();
        init_with_filter(filter);
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
            .unwrap_or_else(|| default_level().to_string())
            .leak()
    };

    let filter_str = format!("warn,glint={app_level}");
    let filter = EnvFilter::new(filter_str);

    init_with_filter(filter);
}

/// Build and initialize the subscriber with the given filter, selecting
/// pretty or JSON format based on `LOG_JSON` env var.
fn init_with_filter(filter: EnvFilter) {
    let use_json = env::var("LOG_JSON")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    if use_json {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(JsonFormatter)
                    .fmt_fields(tracing_subscriber::fmt::format::DefaultFields::new())
                    .with_ansi(false),
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().event_format(CompactFormatter))
            .init();
    }
}

/// Like `init_with_filter` but returns an error instead of panicking if already initialized.
fn try_init_with_filter(filter: EnvFilter) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let use_json = env::var("LOG_JSON")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    if use_json {
        Ok(tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(JsonFormatter)
                    .fmt_fields(tracing_subscriber::fmt::format::DefaultFields::new())
                    .with_ansi(false),
            )
            .try_init()?)
    } else {
        Ok(tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().event_format(CompactFormatter))
            .try_init()?)
    }
}
