# Code Style Guide

## Logging

### Message Format

Use static messages with structured fields. All dynamic content goes in fields, not interpolated into the message string.

```rust
// Correct
info!(endpoint = %addr, bucket = %bucket, "R2 client initialized");
warn!(job_id = %id, error = %e, "Job failed, will retry");

// Wrong - dynamic content in message
info!("R2 client initialized (endpoint: {}, bucket: {})", addr, bucket);
```

Exception: Simple values may appear in the message for critical startup info (e.g., `"Listening on {addr}"`).

### Log Levels

| Level | Use for | Examples |
|-------|---------|----------|
| ERROR | Failures requiring attention | Database connection lost, job failed permanently |
| WARN  | Recoverable issues | Retry succeeded, fallback used, deprecated usage |
| INFO  | Significant lifecycle events | Service started, job completed, config loaded |
| DEBUG | Routine operations | Cache hit, file written, polling tick |
| TRACE | Verbose internals | Request/response bodies, full state dumps |

Default to quiet. If an operation happens regularly without issue, it belongs at DEBUG or TRACE.

### Verbosity Control

- **Default**: `glint_*=info`, all dependencies at `warn`
- **`-v` flag**: Sets app crates to `debug`
- **`-vv` flag**: Sets app crates to `trace`
- **`LOG_LEVEL` env**: Overrides default app level (`LOG_LEVEL=debug`)
- **`RUST_LOG` env**: Full tracing filter syntax, overrides everything

### Imports

Import macros at module top. Do not use qualified `tracing::info!()` calls.

```rust
use tracing::{debug, error, info, instrument, trace, warn};
```

### Request Tracing with Spans

Use `#[instrument]` on handlers and significant functions. Skip large or sensitive arguments.

```rust
#[instrument(skip(db, body), fields(user_id = %user_id))]
async fn create_resource(db: &Pool, user_id: Uuid, body: CreateRequest) -> Result<Resource> {
    // ...
}
```

Spans propagate context automatically—child logs inherit parent span fields.

### Error Logging

Always log the error with `error = %e` to capture the full chain. Include relevant context fields.

```rust
// Correct - error in field, context included
error!(job_id = %id, error = %e, "Failed to process job");

// Wrong - error only in message, loses chain
error!("Failed to process job: {}", e);
```

### Metric-Friendly Fields

Use consistent field names for values that may be aggregated or queried:

| Field | Type | Description |
|-------|------|-------------|
| `duration_ms` | u64 | Operation timing |
| `count` | usize | Item counts |
| `bytes` | u64 | Data sizes |
| `job_id` | Uuid | Job identifier |
| `shader_id` / `scene_id` | i32 | Entity IDs |
| `error` | Display | Error with chain |
