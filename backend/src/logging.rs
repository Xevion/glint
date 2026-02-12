//! Logging configuration with dual-format output (pretty + JSON).
//!
//! ## Output Formats
//!
//! - **Pretty**: `HH:MM:SS.sss LEVEL span{fields} target: message`
//! - **JSON**: Machine-readable structured JSON per line
//!
//! Format selection (via `LOG_JSON`):
//! - `LOG_JSON=true` or `LOG_JSON=1` → JSON
//! - `LOG_JSON=false` or `LOG_JSON=0` → Pretty
//! - Unset → JSON in release builds, pretty in debug builds
//!
//! This matches the frontend's `shouldUseJson()` logic identically.
//!
//! ## Verbosity Control
//!
//! Priority (highest first):
//! 1. `RUST_LOG` - Full tracing filter syntax, overrides everything
//! 2. `-v`/`-vv` flags - Sets app crate level (debug/trace)
//! 3. `LOG_LEVEL` env - Sets app crate level
//! 4. Default - `glint=debug` (dev) / `glint=info` (release), dependencies at `warn`

use std::{borrow::Cow, env, fmt};

use serde::Serialize;
use serde_json::{Map, Value};
use time::{format_description::FormatItem, macros::format_description};
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{
    EnvFilter,
    field::RecordFields,
    fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields, format},
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
};

/// Pretty timestamp format: `HH:MM:SS.sss`
const TIME_FORMAT: &[FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second].[subsecond digits:3]");

/// JSON timestamp format: ISO 8601 with millisecond precision (always UTC)
const JSON_TIME_FORMAT: &[FormatItem<'static>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");

// ── Pretty Formatter ──────────────────────────────────────────────

/// Custom formatter that outputs compact timestamps with colored levels and spans.
///
/// Format: `HH:MM:SS.sss LEVEL span{fields}:span2 target: message`
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

// ── Compact Field Formatter ──────────────────────────────────────

/// Closure that transforms a field's display string.
type FieldTransform = Box<dyn Fn(&str) -> Cow<'_, str> + Send + Sync>;

/// Rule for transforming a named field in pretty output.
enum FieldRule {
    /// Omit the field entirely from display.
    Hide,
    /// Apply a custom transform to the field's formatted value.
    Transform(FieldTransform),
}

/// Field-level display customization for the pretty formatter.
///
/// Wraps DefaultFields — fields without matching rules are formatted
/// identically to the default. Only affects pretty output; the JSON
/// formatter uses DefaultFields directly and is unaffected.
struct CompactFields {
    rules: Vec<(&'static str, FieldRule)>,
}

impl CompactFields {
    fn new() -> Self {
        Self { rules: vec![] }
    }

    #[allow(dead_code)]
    fn hide(mut self, field: &'static str) -> Self {
        self.rules.push((field, FieldRule::Hide));
        self
    }

    fn transform(
        mut self,
        field: &'static str,
        f: impl Fn(&str) -> Cow<'_, str> + Send + Sync + 'static,
    ) -> Self {
        self.rules.push((field, FieldRule::Transform(Box::new(f))));
        self
    }
}

struct CompactVisitor<'a, 'rules> {
    writer: format::Writer<'a>,
    rules: &'rules [(&'static str, FieldRule)],
    is_empty: bool,
    result: fmt::Result,
}

impl Visit for CompactVisitor<'_, '_> {
    fn record_str(&mut self, field: &Field, value: &str) {
        if self.result.is_err() {
            return;
        }
        if field.name() == "message" {
            self.record_debug(field, &format_args!("{}", value))
        } else {
            self.record_debug(field, &value)
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if self.result.is_err() {
            return;
        }

        let name = field.name();
        let rule = self.rules.iter().find(|(n, _)| *n == name).map(|(_, r)| r);

        self.result = match rule {
            Some(FieldRule::Hide) => return,
            Some(FieldRule::Transform(f)) => {
                let formatted = format!("{:?}", value);
                let transformed = f(&formatted);
                if self.is_empty {
                    write!(self.writer, "{}={}", name, transformed)
                } else {
                    write!(self.writer, " {}={}", name, transformed)
                }
            }
            None if name == "message" => {
                if self.is_empty {
                    write!(self.writer, "{:?}", value)
                } else {
                    write!(self.writer, " {:?}", value)
                }
            }
            None => {
                if self.is_empty {
                    write!(self.writer, "{}={:?}", name, value)
                } else {
                    write!(self.writer, " {}={:?}", name, value)
                }
            }
        };

        self.is_empty = false;
    }
}

impl<'writer> FormatFields<'writer> for CompactFields {
    fn format_fields<R: RecordFields>(
        &self,
        writer: format::Writer<'writer>,
        fields: R,
    ) -> fmt::Result {
        let mut visitor = CompactVisitor {
            writer,
            rules: &self.rules,
            is_empty: true,
            result: Ok(()),
        };
        fields.record(&mut visitor);
        visitor.result
    }
}

/// Build the configured CompactFields for the pretty output path.
fn compact_fields() -> CompactFields {
    CompactFields::new().transform("req_id", |v| {
        if v.len() > 12 {
            Cow::Owned(format!("{}..{}", &v[..4], &v[v.len() - 6..]))
        } else {
            Cow::Borrowed(v)
        }
    })
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
                .format(JSON_TIME_FORMAT)
                .unwrap_or_else(|_| String::from("1970-01-01T00:00:00.000Z")),
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

/// Determine whether JSON logging should be used.
///
/// Priority:
/// 1. `LOG_JSON` env var explicitly set → use that value
/// 2. Otherwise → JSON in release builds, pretty in debug builds
///
/// This matches the frontend's `shouldUseJson()` logic so both subsystems
/// behave identically with the same (or absent) env vars.
fn should_use_json() -> bool {
    match env::var("LOG_JSON") {
        Ok(v) => v == "true" || v == "1",
        Err(_) => !cfg!(debug_assertions),
    }
}

/// Build and initialize the subscriber with the given filter, selecting
/// pretty or JSON format based on `LOG_JSON` env var.
fn init_with_filter(filter: EnvFilter) {
    let use_json = should_use_json();

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
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(CompactFormatter)
                    .fmt_fields(compact_fields()),
            )
            .init();
    }
}

/// Like `init_with_filter` but returns an error instead of panicking if already initialized.
fn try_init_with_filter(filter: EnvFilter) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let use_json = should_use_json();

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
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(CompactFormatter)
                    .fmt_fields(compact_fields()),
            )
            .try_init()?)
    }
}
