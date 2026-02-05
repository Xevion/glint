# Kotlin Style Guide (Mod)

General principles in [STYLE.md](STYLE.md).

## Architecture

### Architectury Multi-Loader

All game logic lives in `common/`. Platform modules (`fabric/`, `neoforge/`) contain only loader-specific glue code (5–20 lines each).

```
mod/
├── common/src/main/
│   ├── kotlin/com/xevion/glint/   # All business logic
│   │   ├── api/          # HTTP clients (GlintApi, AgentApi)
│   │   ├── capture/      # Capture lifecycle, state management
│   │   ├── download/     # Shader pack downloading
│   │   ├── input/        # Keybind handling
│   │   ├── io/           # File I/O utilities
│   │   ├── orchestration/ # Autonomous capture state machine
│   │   ├── scene/        # Scene management
│   │   ├── screenshot/   # Screenshot capture
│   │   ├── session/      # Session lifecycle
│   │   └── ui/           # Minecraft GUI screens
│   └── java/.../mixin/   # Mixins (must be Java)
├── fabric/                # ClientModInitializer + tick registration
└── neoforge/              # @Mod annotation + event bus registration
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

Category-based structured logging via the `Loggers` enum:

```kotlin
private val log = Loggers.Capture.get()

log.info("Capture complete") {
    "shader_id" to shaderId
    "duration_ms" to elapsed
    "scene_id" to sceneId
}
```

- Each domain area uses its dedicated logger category
- Structured fields via the `StructuredLog` DSL (key-value pairs)
- Lazy evaluation for expensive messages: `log.debug { "State: ${expensiveDump()}" }`
- Log exceptions with cause parameter: `log.error(cause, "Operation failed") { "context" to value }`

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

- **Runner**: Gradle test (`just test mod`)
- **Smoke test**: `just smoke` for runtime mixin verification
- Integration testing via smoke test is the highest-value test for mod code
- Unit tests for pure logic (scene resolution, state machine transitions)
