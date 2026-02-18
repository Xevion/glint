# Rust Style Guide (Backend)

General principles in [STYLE.md](STYLE.md).

## Architecture

### Layer Rules

Strict layering for data integrity:

```
routes (REST handlers) / graphql (GraphQL resolvers)
  → services (business logic, background tasks)
    → repos (database access)
      → DB (PostgreSQL via SQLx)
```

- **Routes** handle REST HTTP concerns: extract params, call services/repos, return responses.
- **GraphQL** resolvers follow the same layering — call repos/services, never touch the DB directly.
- **Services** contain business logic that spans multiple repos or has side effects.
- **Repos** are the only code that touches the database. All SQL lives here.
- Routes/resolvers may call repos directly for simple CRUD. A service layer is required when logic spans multiple repos or has side effects beyond a single query.

### Module Organization

```
src/
├── analytics.rs   # PostHog client wrapper (fire-and-forget event tracking)
├── auth/          # Extractors: AuthUser, AdminUser, MaybeAuthUser
├── cache.rs       # In-memory session cache with TTL eviction
├── cli/           # CLI subcommands (seed, etc.)
├── config/        # Figment-based configuration
├── db/            # Pool initialization, migrations
├── error.rs       # AppError, AppResult
├── extraction/    # Shader metadata extraction (properties, lang, zip) + display name normalization
├── fmt.rs         # Debug formatting helpers
├── graphql/       # GraphQL API (async-graphql): queries, subscriptions, types, pagination
├── id.rs          # Newtype ID wrappers with macro-generated traits
├── lib.rs         # Library root
├── logging.rs     # Dual-format logging setup (pretty + JSON)
├── main.rs        # Server startup, router assembly
├── middleware/     # Axum middleware (rate limiting, cache control, request IDs, security headers)
├── models/        # Domain types, DTOs, request/response shapes
├── platform/      # External API clients (Modrinth, CurseForge)
├── repo/          # One module per domain entity
├── routes/        # One module per resource, each exports router()
├── services/      # Background tasks, complex business logic
├── slug.rs        # URL-safe slug generation
└── state.rs       # AppState (Arc<Inner>)
```

Each route module exports a `router()` function. Repos are unit structs with inherent methods that take `&DbPool`.

## Error Handling

- `AppError` enum with `thiserror` for domain errors (NotFound, BadRequest, Conflict, etc.)
- `anyhow::Error` for internal/unexpected failures, wrapped via `AppError::Internal`
- `AppResult<T>` alias for `Result<T, AppError>`
- `From<sqlx::Error>` and `From<anyhow::Error>` conversions for `?` propagation
- `anyhow::Context` for adding context to repo/service errors

```rust
// Repo: use anyhow context
let shader = sqlx::query_as!(Shader, "...")
    .fetch_optional(pool).await
    .context("Failed to fetch shader")?
    .ok_or(AppError::NotFound)?;

// Route: errors convert automatically
async fn get_shader(State(state): State<AppState>, Path(id): Path<i32>) -> AppResult<Json<Shader>> {
    let shader = ShaderRepo::get(state.db(), id).await?;
    Ok(Json(shader))
}
```

## State Management

`AppState` wraps `Arc<AppStateInner>` with accessor methods. Currently monolithic — will evolve toward modular sub-states with trait-based extraction as the app grows.

```rust
// Access via Axum extractor
async fn handler(State(state): State<AppState>) -> AppResult<Json<T>> {
    let db = state.db();
    let s3 = state.s3(); // Returns Option<&S3Client>
}
```

Optional services return `Option<&T>` — handlers check availability before use.

## Database

- **SQLx with compile-time verification** — all queries checked against the schema at build time
- **Inline queries** via `sqlx::query!` and `sqlx::query_as!` — no separate SQL files
- **Migrations** run automatically on startup via `sqlx::migrate!()`
- Prefer `query_as!` for SELECT (maps to structs), `query!` for mutations
- Use `Option<T>` for nullable columns
- **Never store JSON arrays as TEXT** — use JSONB columns with `Json<Vec<T>>` in Rust so the API sends proper typed arrays, not stringified JSON that frontend must parse. This prevents the entire class of bugs where templates render raw `["a","b","c"]` strings.

## Serialization

- All Glint-owned types use `snake_case` field names — no `#[serde(rename_all)]` needed since Rust's default matches. This includes response models, request bodies, **and query parameter structs** (`Query<T>` extractors).
- External API client types (CurseForge, Modrinth) keep whatever casing the upstream API uses — add `#[serde(rename_all = "camelCase")]` or per-field `#[serde(rename)]` as needed.
- Enum variants use their own conventions: `snake_case` for statuses, lowercase for sorts, `SCREAMING_SNAKE_CASE` where appropriate.
- Types exported to frontend derive `TS` with `#[ts(export)]`
- Request types: derive `Deserialize`. Response types: derive `Serialize`. Shared types: both.

```rust
// Query param structs are snake_case too — clients send ?scene_id=..., not ?sceneId=...
#[derive(Deserialize)]
struct SceneQuery {
    scene_id: Option<String>,
}
```

### Null Omission in API Responses

Response structs with `Option<T>` fields use `#[skip_serializing_none]` (from `serde_with`) to omit null fields from JSON, and `#[ts(optional_fields)]` so ts-rs generates `field?: T` instead of `field: T | null`. This reduces payload size and produces idiomatic TypeScript.

```rust
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Serialize, TS)]
#[ts(export, optional_fields)]
pub struct Shader {
    pub id: ShaderId,
    pub name: String,
    pub description: Option<String>,  // omitted from JSON when None, TS: description?: string
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,    // non-optional, TS: created_at: string
    #[ts(as = "Option<String>")]
    pub updated_at: Option<DateTime<Utc>>,  // optional DateTime, TS: updated_at?: string
}
```

**`#[ts(type)]` vs `#[ts(as)]` with optional fields:**

`#[ts(type = "string")]` replaces the **entire** TypeScript type for a field, bypassing `optional_fields`. On an `Option<DateTime<Utc>>` field, it generates `field: string` (non-optional) — **wrong**.

Use `#[ts(as = "Option<String>")]` instead, which tells ts-rs to treat the field as `Option<String>`. With `optional_fields`, this correctly generates `field?: string`.

| Rust field type | Annotation | Generated TypeScript | Correct? |
|---|---|---|---|
| `DateTime<Utc>` | `#[ts(type = "string")]` | `field: string` | ✓ |
| `Option<DateTime<Utc>>` | `#[ts(type = "string")]` | `field: string` | ✗ (lost optionality) |
| `Option<DateTime<Utc>>` | `#[ts(as = "Option<String>")]` | `field?: string` | ✓ |
| `Option<Json<Vec<T>>>` | `#[ts(optional, type = "Array<string>")]` | `field?: Array<string>` | ✓ |
| `Option<serde_json::Value>` | `#[ts(optional, type = "Record<string, unknown>")]` | `field?: Record<string, unknown>` | ✓ |

For complex types without a `TS` impl (like `serde_json::Value`, `Json<Vec<T>>`), combine `#[ts(optional)]` with `#[ts(type = "...")]` to preserve optionality while overriding the type.

## Async

- `tokio` runtime. All I/O is async.
- `tokio::spawn` for background tasks (heartbeat monitor, cleanup jobs)
- Background tasks log errors and continue — no panics.
- No explicit locking — SQLx pool handles concurrent DB access.

## Logging

- Import macros at module top: `use tracing::{debug, error, info, warn};`
- Use `#[instrument]` on handlers and significant functions. Skip large/sensitive args.
- Log errors in structured fields: `error!(error = %e, "Failed to process")`
- Spans propagate context — child logs inherit parent span fields.

```rust
#[instrument(skip(state, body), fields(shader_id = %id))]
async fn update_shader(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateRequest>,
) -> AppResult<Json<Shader>> {
    // tracing context automatically includes shader_id
}
```

## Linting

- Zero clippy warnings allowed (`--deny warnings`)
- Run `just check` to validate (includes clippy)

## Optionality

- Use `Option<T>` for genuinely optional data (nullable DB columns, optional config)
- Prefer requiring values when the domain demands them — don't default to `Option` for convenience
- Use newtypes for critical domain identifiers where type safety matters

## Testing

- **Runner**: `cargo nextest`
- **Integration tests** in `tests/` for handler-level testing
- **Unit tests** alongside code in `#[cfg(test)]` modules for repo/service logic
- Name tests descriptively: `test_<action>_<condition>_<expected_result>`
- Use `assert2` crate when available
