# Kotlin Style Guide (Mod)

General principles in [STYLE.md](STYLE.md).

## Architecture

### Architectury Multi-Loader

All game logic lives in `common/`. Platform modules (`fabric/`, `neoforge/`) contain only loader-specific glue code (5–20 lines each).

```
mod/
├── common/src/main/
│   ├── kotlin/com/xevion/glint/   # All business logic
│   │   ├── Glint.kt, MinecraftContext.kt, Loggers.kt  # Entry point, typed world access, logging
│   │   ├── api/           # HTTP clients (HttpClient, AgentClient, AuthClient), scene sync, API config
│   │   ├── capture/       # Capture lifecycle, Iris/Sodium integration, framebuffer/WebP writing
│   │   ├── command/       # In-game chat commands (GlintCommands)
│   │   ├── input/         # Keybind handling
│   │   ├── io/            # File I/O utilities
│   │   ├── orchestration/ # Autonomous capture (AutonomousRunner, LinearOrchestrator, AssetPreparer)
│   │   ├── scene/         # Scene packages, injection, chunk provision, local storage, export
│   │   ├── session/       # Session registry (SessionRegistry)
│   │   ├── ui/            # Minecraft GUI screens
│   │   │   └── base/      # Shared screen framework (GlintPanelScreen, GlintScreen, GlintTheme)
│   │   └── upload/        # Upload progress models and error types
│   └── java/.../mixin/    # Mixins (must be Java)
├── fabric/                 # ClientModInitializer + tick registration
└── neoforge/               # @Mod annotation + event bus registration
```

### State Management

Use a service locator pattern. A central registry holds references to singleton services, making dependencies explicit and enabling future testability.

- Each service is a singleton `object` with clear ownership of its state
- Services declare dependencies explicitly rather than reaching into other singletons
- One object owns a piece of state — others read but don't mutate directly
- Thread-safe state uses `AtomicBoolean`, `ConcurrentHashMap`, or synchronized access

```kotlin
// CaptureStateManager owns all capture lifecycle state
object CaptureStateManager {
    private val isCapturing = AtomicBoolean(false)
    fun startCapture(): Boolean = isCapturing.compareAndSet(false, true)
    fun isActive(): Boolean = isCapturing.get()
}
```

### Tick-Driven State Machines

Minecraft mods run on a tick loop. Long-running operations use explicit state machines polled each tick, not blocking calls.

```kotlin
private enum class State { Idle, Loading, Running, Finishing }
private var state = State.Idle

fun tick() {
    when (state) {
        State.Idle -> { /* wait for trigger */ }
        State.Loading -> { if (futureReady()) state = State.Running }
        State.Running -> { /* do work, advance when done */ }
        State.Finishing -> { cleanup(); state = State.Idle }
    }
}
```

## Error Handling

- Return `Result<T>` for operations that can fail (HTTP, file I/O, reflection)
- Sealed class hierarchies for domain-specific error types (e.g., `ApiError`)
- `runCatching {}` at integration boundaries (Class.forName, reflection)
- Never catch `Exception` broadly — catch specific types or use Result

```kotlin
sealed class ApiError : Exception() {
    abstract val userMessage: String
    data class NetworkError(override val message: String, override val cause: Throwable?) : ApiError()
    data class HttpError(val statusCode: Int, val responseBody: String?) : ApiError()
}
```

## Async / Concurrency

Adopt **Kotlin coroutines** for async work. Use `CompletableFuture` only for legacy interop.

- Structured concurrency with `CoroutineScope` tied to mod lifecycle
- `Dispatchers.IO` for HTTP and file I/O
- Game-thread work stays synchronous in tick handlers
- Suspend functions for all new async operations

```kotlin
// Preferred: coroutines
suspend fun fetchShaderList(): Result<List<Shader>> = withContext(Dispatchers.IO) {
    runCatching { api.get("/shaders").decode() }
}

// Legacy: CompletableFuture (existing code, migrate over time)
fun loadAsync(): CompletableFuture<SceneCollection?> { ... }
```

Dedicated thread pools for I/O remain acceptable when coroutine integration isn't practical (e.g., deep Minecraft API interop).

## Kotlin Idioms

- **Data classes** for all DTOs, config objects, and value types
- **Sealed classes/interfaces** for state machines, error hierarchies, and discriminated unions
- **Extension functions** freely for adding behavior to types you don't own (Minecraft types, Java stdlib)
- **Scope functions** (`let`, `run`, `apply`, `also`) sparingly — avoid nesting them
- **Nullable types** with `?.let`, `?.takeIf` for null-safe chains
- **Named arguments** when calling functions with multiple parameters of the same type
- **Trailing lambda** syntax for DSL-style APIs and callbacks

## Logging

Category-based structured logging via the `Loggers` enum and `StructuredLog` DSL.

### Setup

```kotlin
private val log = Loggers.Capture.get()
```

Each domain area uses its dedicated logger category from the `Loggers` enum.

### Structured Fields

Use the DSL block to attach key-value context. Fields are rendered as `key=value` pairs after the message on the console, and also attached to the SLF4J `LoggingEventBuilder` via `addKeyValue()` for structured consumers (JSON appenders, log aggregators).

```kotlin
log.info("Capture complete") {
    "shader_id" to shaderId
    "duration_ms" to elapsed
}
// Console: [01:47:49.328] [INFO ] [capture] Capture complete shader_id=iris duration_ms=42
```

Values containing spaces, `=`, or `"` are automatically quoted:

```kotlin
log.error("Failed to set shader pack") { "pack" to "Complementary Reimagined" }
// Console: [01:47:49.329] [ERROR] [capture] Failed to set shader pack pack="Complementary Reimagined"
```

### When to Use Structured Fields vs Plain Messages

- **Structured fields** — variable data that you'd want to filter/search on: IDs, paths, counts, durations
- **Plain messages** — static lifecycle events with no variable context: `"Capture started"`, `"Shutdown complete"`

### Lazy Messages

For expensive message construction, use a lambda. The lambda is only invoked if the level is enabled:

```kotlin
log.debug { "State: ${expensiveDump()}" }
```

### Exceptions

Pass the cause as the first parameter:

```kotlin
log.error(cause, "Operation failed") { "context" to value }
log.warn(cause, "Retrying") { "attempt" to n }
```

### Never Use SLF4J Parameterized Style

```kotlin
// WRONG — bypasses the DSL, renders as raw {} on some backends
log.info("Downloaded {} files", count)

// RIGHT — structured and always renders correctly
log.info("Downloaded files") { "count" to count }
```

### Available Overloads

Every level (TRACE through ERROR) has a symmetrical set of 7 extension overloads:
- `log.info { "lazy" }` — lazy message
- `log.info("static") { "k" to v }` — static message + structured fields
- `log.info({ "lazy" }) { "k" to v }` — lazy message + structured fields
- `log.info(cause, "static")` — exception + static message
- `log.info(cause) { "lazy" }` — exception + lazy message
- `log.info(cause, "static") { "k" to v }` — exception + static message + structured fields
- `log.info(cause, { "lazy" }) { "k" to v }` — exception + lazy message + structured fields

Plus SLF4J's own `log.info("static")` for plain static messages.

## Mixin Development

- Mixins are **Java** (required by the Mixin library)
- Use **Mojang mappings** — never Fabric/Yarn names
- Verify method signatures with `just mcjar` before writing mixins
- Prefix `@Unique` fields with `glint$` to avoid conflicts
- Test all mixin changes with `just smoke`

## Serialization

- `kotlinx.serialization` for all JSON (de)serialization
- `@Serializable` on data classes
- `Json { ignoreUnknownKeys = true; encodeDefaults = true }` as default config
- Config files stored as JSON in `.minecraft/glint/`

## Reflection & Runtime Dependencies

When using reflection for optional mod integration (e.g., Iris, Sodium):

- `runCatching {}` with `Class.forName` to detect optional mods
- Validate method signatures — catch `NoSuchMethodException`
- Validate return types — catch `ClassCastException`
- Log at appropriate levels:
  - `debug` for expected missing classes (mod not installed)
  - `error` for API signature changes (breaking changes)
  - `warn` for unexpected failures

## Testing

- **Runner**: Gradle test (`just test mod` or `just test m`)
- **Smoke test**: `just smoke` for runtime mixin verification
- Integration testing via smoke test is the highest-value test for mod code
- Unit tests for pure logic (scene resolution, state machine transitions)
