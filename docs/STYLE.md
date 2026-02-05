# Code Style Guide

Style guides for each subsystem: [Rust (backend)](RUST.md) | [Kotlin (mod)](KOTLIN.md) | [Svelte (frontend)](SVELTE.md)

## Formatting

Automated. Rust uses `rustfmt`, Kotlin uses Spotless + KtLint, frontend uses Biome. Don't think about it — `just format` handles everything.

## Naming & Domain Vocabulary

Use language-idiomatic casing (snake_case in Rust, camelCase in Kotlin/TypeScript). Use consistent domain terms across all stacks:

| Concept | Name | Notes |
|---------|------|-------|
| A shader pack | `Shader` | Not "pack", "shaderpack", or "mod" |
| A screenshot configuration | `Scene` | A specific location + time + weather + settings |
| A screenshot result | `Capture` | Not "screenshot" or "image" |
| A capture batch task | `Job` | Backend-assigned unit of work for the mod |
| A Minecraft world save | `World` | The world a scene exists in |
| A shader version | `ShaderVersion` | Specific release of a shader |

These names are used in types, API endpoints, database tables, and UI copy. When in doubt, check the Rust backend models — they're the source of truth.

## Comments

Explain **why**, not **what**. Code should be self-documenting through clear names and small functions. Comments exist for:

- Non-obvious decisions or trade-offs
- Workarounds with context on what they're working around
- Domain knowledge that isn't obvious from the code

Never reference old implementations, migrations, or refactoring history in comments. Never add banner comments (`===`, `---`).

## Logging

### Principles

Static messages, structured fields. All dynamic content goes in fields, never interpolated into the message string. This makes logs greppable and machine-parseable.

### Log Levels

| Level | Use for | Examples |
|-------|---------|----------|
| ERROR | Failures requiring attention | Database connection lost, job failed permanently |
| WARN  | Recoverable issues | Retry succeeded, fallback used |
| INFO  | Significant lifecycle events | Service started, job completed, config loaded |
| DEBUG | Routine operations | Cache hit, file written, polling tick |
| TRACE | Verbose internals | Request/response bodies, full state dumps |

Default to quiet. If an operation happens regularly without issue, it's DEBUG or TRACE.

### Standard Field Names

Use consistent field names across all stacks for values that may be aggregated or queried:

| Field | Type | Description |
|-------|------|-------------|
| `duration_ms` | number | Operation timing |
| `count` | number | Item counts |
| `bytes` | number | Data sizes |
| `job_id` | string | Job identifier |
| `shader_id` / `scene_id` | number | Entity IDs |
| `error` | string | Error with chain |

## Error Handling

### Philosophy

Errors are values, not exceptions. Each stack uses its own idiomatic pattern, but the principles are shared:

- **Expected failures** (not found, validation, conflict) are handled explicitly with typed errors
- **Unexpected failures** (I/O, serialization, OOM) are wrapped and propagated
- **Never swallow errors silently** — log or propagate, never `catch {}` empty
- **User-facing error messages** are separate from internal error details

### API Error Responses

All API errors return a consistent JSON shape:

```json
{
  "error": "Human-readable message",
  "code": "SCREAMING_SNAKE_ERROR_CODE"
}
```

HTTP status codes map to error categories: 400 (validation), 404 (not found), 409 (conflict), 500 (internal).

## API Design

- **Casing**: All JSON fields use `camelCase`
- **List responses**: Wrapped with `{ items: [...], total: N }` for pagination support
- **Single responses**: Return the object directly (no wrapper)
- **IDs in URLs**: Use the resource's primary identifier: `/api/shaders/{id}`
- **Verbs via HTTP methods**: GET (read), POST (create), PUT (full update), PATCH (partial update), DELETE (remove)

## Testing

### Principles

- Test behavior, not implementation. Tests should survive refactors.
- Prefer integration tests that exercise real code paths over unit tests with heavy mocking.
- Name tests descriptively: `test_creating_shader_with_duplicate_slug_returns_conflict`.
- Each stack has its own test runner and conventions — see the language-specific guides.
