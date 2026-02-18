# Domain Vocabulary

Canonical terms for Glint's domain. Use these names consistently in types, API endpoints, database tables, UI copy, and conversation. The Rust backend models are the source of truth for structure; this document is the source of truth for naming.

## Core Entities

| Term              | Definition |
|-------------------|------------|
| **Shader**        | A shader pack identity. Represents one shader across all its versions, profiles, and platforms. Not "pack", "shaderpack", or "mod". |
| **ShaderVersion** | A specific release of a Shader. Has a version string, download URL, file hash, and optionally a list of supported Iris profiles. The `(shader_id, version)` pair is unique. |
| **Scene**         | A named camera location and environment. Has a slug, dimension, and active flag. Configuration details (position, camera, time, weather, biome, render settings, scene package) live on SceneVersion. Scenes are the *where* of a capture. |
| **SceneVersion**  | A specific revision of a Scene's configuration: position (x/y/z), camera (yaw/pitch), time of day, weather, biome, render settings (FOV, render distance), and optional scene package (world file zip stored in R2). Captures record which SceneVersion they were taken against for freshness tracking. Follows the same `DISTINCT ON ... ORDER BY created_at DESC` pattern as ShaderVersion. |
| **ScenePreset**   | A named time/weather/moon-phase variation within a Scene (e.g., "Sunset", "Stormy Night"). Has a `sort_order` for UI ordering. Captures can target a specific preset, enabling multiple environment variations per scene without duplicating the scene definition. |
| **Capture**       | A single screenshot produced by rendering a specific ShaderVariant in a specific Scene (with optional ScenePreset). Contains `image_path` (public URLs derived at runtime via CDN/imgproxy), resolution, performance metrics, and Minecraft/Iris version metadata. Records `scene_version_id` and optional `preset_id` for freshness tracking. Multiple Captures can exist for the same CaptureTarget. |
| **CaptureRun**    | An auditable session of capture work. Created when the mod begins a batch of captures, tracks progress (total/completed/failed/skipped items), and completed when the batch finishes. Status can be `running`, `completed`, `partial`, `failed`, or `timed_out`. |

## Composite Concepts

These are logical groupings that don't (yet) have their own database tables but are referenced throughout the codebase.

| Term               | Composition | Definition |
|--------------------|-------------|------------|
| **ShaderVariant**  | Shader + ShaderVersion + ShaderVersionProfile | A specific renderable configuration: a particular version of a shader pack with a particular Iris profile (or the default/none). This is the *what* being rendered. A null profile means either vanilla rendering or a shader without Iris profiles. |
| **CaptureTarget**  | ShaderVariant + Scene + ScenePreset | The unique combination identifying *what should be captured where*. The work queue, deduplication, and capture history all revolve around this concept. Concretely: `(shader_version_id, scene_id, profile_id, preset_id)`. Freshness is computed by comparing a Capture's `scene_version_id` against the latest SceneVersion (see CaptureFreshness). |
| **CaptureFreshness** | Enum: `fresh` / `stale` / `superseded` | Computed at query time. `fresh` = latest capture for a target using the current SceneVersion. `stale` = latest capture but against an outdated SceneVersion or preset. `superseded` = a newer capture exists for this target. Replaces the old WorldVersion-based staleness concept. |
| **Latest Capture** | CaptureTarget → Capture | The most recent Capture for a given CaptureTarget, derived by `captured_at DESC`. This is what users see in the gallery and comparison views. Historical captures for the same target are retained but not displayed by default. Determined at query time, not a stored flag. |

## Taxonomy & Metadata

| Term         | Definition |
|--------------|------------|
| **Category** | A stylistic classification for a Shader (e.g., realistic, fantasy, cartoon). Many-to-many. Describes the visual *feel*. |
| **Feature**  | A technical capability of a Shader (e.g., volumetric lighting, PBR, ray tracing). Many-to-many. Describes what the shader *does*. |
| **Tag**      | A descriptive label for a Scene (e.g., indoor, sunset, underwater, village). Many-to-many. Describes the scene's characteristics. |
| **Profile**  | An Iris shader profile configuration for a shader version. Profiles are *discovered* — extracted from shader pack zip files during the extraction pipeline (parsing `shaders.properties` and `.lang` files). A null profile means the shader's default configuration. Each profile has three name representations (see below). |

### Profile Name Representations

A profile has three distinct name fields. These serve different purposes and must not be confused:

| Field | DB Column | Example | Purpose |
|-------|-----------|---------|---------|
| **Profile Name** | `name` | `HIGH`, `ULTRA`, `POTATO` | Internal identifier from `shaders.properties`. Used by Iris to load profiles. The mod **must** use this value — display names will cause `Profile not found` errors. |
| **Profile Label** | `label` | `§aHigh §7Quality`, `§6✦ Ultra §7Premium` | Raw label from `.lang` files. Contains Minecraft formatting codes (`§`), decorative Unicode, and metadata suffixes. Admin-only — never shown to end users. |
| **Profile Display Name** | `display_name` | `High Quality`, `Ultra Premium` | Normalized label with formatting codes, decorative characters, and metadata stripped. Used in all user-facing UI. Computed during extraction and backfilled for existing profiles. |

**In API responses:**
- `profile_name` = the internal name (for the mod and internal references)
- `profile_display_name` = the display name (for UI rendering)
- The label is only exposed on the `ShaderVersionProfile` entity (admin detail views)

## Platform Integration

| Term         | Definition |
|--------------|------------|
| **Adoption** | The process of importing a Shader from an external platform (Modrinth or CurseForge) into Glint's catalog. Creates the Shader, its ShaderVersions, and ShaderAuthors. A two-step flow: preview then confirm. |
| **Upstream** | The external platform (Modrinth/CurseForge) that a Shader was adopted from. "Upstream" fields track platform-side metadata: download counts, update timestamps, sync timestamps. |
| **Sync**     | Refreshing a Shader's metadata and versions from its upstream platform. Tracked via `last_synced_at`. |

## Capture Orchestration

The mod has a layered capture architecture. From outermost to innermost:

| Term                      | Layer | Definition |
|---------------------------|-------|------------|
| **AutonomousRunner**      | 1     | The top-level agent loop. Fetches work from the backend, creates a CaptureRun, delegates to the LinearOrchestrator, uploads results, and repeats until no work remains. |
| **LinearOrchestrator**    | 2     | Manages capture within a single CaptureRun. Processes work items linearly: loads scene packages, applies environment (with presets), then for each shader: load → stabilize → capture. Detects scene/preset/shader transitions to minimize expensive operations. Produces an OrchestrationManifest on completion. |
| **Stabilization**         | —     | The process of waiting for the game to reach a visually stable state before capturing. Includes chunk loading, FPS settling, and render pipeline convergence. Affects capture quality and timing. |
| **OrchestrationManifest** | —     | A JSON file written to the output directory after an orchestration run. Contains metadata: timing, scenes captured, screenshots produced, shader/Minecraft/Iris versions. |

### Work & Run Concepts

| Term                | Definition |
|---------------------|------------|
| **WorkItem**        | A composed packet returned by `GET /api/work` containing everything the mod needs to capture a single CaptureTarget. Contains four sub-types: `WorkShader` (version, download URL, hash, profile info), `WorkScene` (position, camera, environment, render settings), optional `WorkPreset` (environment overrides), and optional `WorkPackage` (scene package URL/hash/size). Eliminates additional API lookups. |
| **CaptureRunItem**  | A single unit of work within a CaptureRun: one CaptureTarget to process. Tracks status (`pending` → `running` → `completed`/`failed`/`skipped`), timing, error details, and links to the resulting Capture on success. |
| **ShaderSpec**      | A shader filename + Iris profile ID, used within the LinearOrchestrator to specify what to render. Built from `WorkShader.toShaderSpec()`. Maps to the mod's local shader pack files. Mod-only concept. |

## Authentication

| Term                 | Definition |
|----------------------|------------|
| **Device Code Flow** | OAuth 2.0 Device Authorization Grant (RFC 8628) used by the Minecraft mod. The mod requests a device code, displays a user code (`GLINT-XXXXXX` format), the user authorizes in their browser, and the mod exchanges for a session token. |
| **Agent**            | The identity of a mod instance performing captures. Stored as `agent_id` on CaptureRuns. Currently a free-form string; may become a formal entity. |
| **Session**          | A database-backed authentication token. Has a source (`web` for browser login, `device` for mod login) and an expiration timestamp. |

## Scene Configuration (Mod)

| Term                | Definition |
|---------------------|------------|
| **SceneCollection** | A mod-local JSON file defining scenes, stored at `.minecraft/glint/scenes/<name>.json`. Contains a default SceneConfig and a list of Scenes. This is a mod-side concept — the backend uses Scene + SceneVersion + ScenePreset instead. |
| **SceneConfig**     | Render and capture settings: render distance, graphics mode, FOV, resolution, particles, clouds, etc. Supports inheritance — a scene's config merges with the collection's `defaultConfig` as fallback. |
| **SceneVariant**    | An override layer on a Scene that modifies specific properties (time of day, weather, etc.) to create alternative versions of the same location. Used for day/night, seasonal, or weather variations. |
| **SceneEntity**     | An entity definition within a SceneCollection for reproducible scene setup (placing specific mobs or items). |

## Status Lifecycles

| Context        | States | Notes |
|----------------|--------|-------|
| Capture        | `uploading` → `completed`, or `uploading` → `failed` | `uploading` is transient during file upload |
| CaptureRun     | `running` → `completed` / `partial` / `failed` / `timed_out` | `partial` = some items completed, some failed; `timed_out` = run exceeded time limit |
| CaptureRunItem | `pending` → `running` → `completed` / `failed` / `skipped` | `skipped` = valid capture already existed |
| DeviceCode     | `pending` → `authorized` → `used` | RFC 8628 lifecycle |

## Relationships

```
Scene
├── SceneVersion (one-to-many, ordered by created_at)
│   └── Scene Package (optional, stored in R2)
├── ScenePreset (one-to-many, ordered by sort_order)
└── Tag (many-to-many)

Shader
├── ShaderVersion (one-to-many)
│   └── ShaderVersionProfile (one-to-many, discovered during extraction)
├── ShaderAuthor (one-to-many)
├── Category (many-to-many)
└── Feature (many-to-many)

CaptureTarget = (ShaderVersion + ShaderVersionProfile + Scene + ScenePreset)
└── Capture (one-to-many, ordered by captured_at)
    ├── Latest Capture (derived: most recent per target)
    ├── SceneVersion (many-to-one, records which version was active)
    └── CaptureFreshness (derived: fresh / stale / superseded)

CaptureRun
└── CaptureRunItem (one-to-many)
    └── Capture (optional, on success)
```

## Anti-Patterns

Avoid these terms — they've been sources of confusion or have been superseded:

| Don't Say | Say Instead | Why |
|-----------|-------------|-----|
| "shader pack", "shaderpack" | Shader | Shorter, consistent |
| "screenshot", "image" | Capture | Domain-specific, includes metadata beyond the image |
| "Job" | CaptureRun | `Job` was the legacy name, replaced in the capture redesign |
| "config" (for shader settings) | Profile | "Config" is ambiguous — could mean SceneConfig, game settings, etc. |
| "outdated capture" | stale capture | A capture whose `scene_version_id` doesn't match the latest SceneVersion, or whose preset is outdated. Freshness is computed at query time via the `CaptureFreshness` enum, never stored as a flag. |
| "world", "world file" | scene package | Worlds were replaced by scene packages attached to SceneVersion. The mod injects scene packages into a staging world rather than loading separate world saves. |
