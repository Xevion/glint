# Domain Vocabulary

Canonical terms for Glint's domain. Use these names consistently in types, API endpoints, database tables, UI copy, and conversation. The Rust backend models are the source of truth for structure; this document is the source of truth for naming.

## Core Entities

| Term              | Definition |
|-------------------|------------|
| **Shader**        | A shader pack identity. Represents one shader across all its versions, profiles, and platforms. Not "pack", "shaderpack", or "mod". |
| **ShaderVersion** | A specific release of a Shader. Has a version string, download URL, file hash, and optionally a list of supported Iris profiles. The `(shader_id, version)` pair is unique. |
| **World**         | A Minecraft world identity. Contains one or more Scenes. Has a Minecraft version. Artifact data (file URL, hash, size) lives on WorldVersion. |
| **WorldVersion**  | A specific revision of a World's save file. Holds the file URL, hash, and size. Captures record which WorldVersion they were taken against, enabling computed staleness detection when worlds are rebuilt. Follows the same `DISTINCT ON ... ORDER BY created_at DESC` pattern as ShaderVersion for resolving the latest version. |
| **Scene**         | A specific camera position and environment configuration within a World. Defines location (x/y/z), rotation (yaw/pitch), time of day, weather, dimension, biome, and render settings. Scenes are the *where* of a capture. |
| **Capture**       | A single screenshot produced by rendering a specific ShaderVariant in a specific Scene. Contains the image URL, resolution, performance metrics, and metadata about the Minecraft/Iris versions used. Records `world_version_id` to track which WorldVersion was active at capture time. Multiple Captures can exist for the same CaptureTarget. |
| **CaptureRun**    | An auditable session of capture work. Created when the mod begins a batch of captures, tracks progress (total/completed/failed/skipped items), and completed when the batch finishes. |

## Composite Concepts

These are logical groupings that don't (yet) have their own database tables but are referenced throughout the codebase.

| Term               | Composition | Definition |
|--------------------|-------------|------------|
| **ShaderVariant**  | Shader + ShaderVersion + Profile | A specific renderable configuration: a particular version of a shader pack with a particular Iris profile (or the default/none). This is the *what* being rendered. A null profile means either vanilla rendering or a shader without Iris profiles. |
| **CaptureTarget**  | ShaderVariant + Scene | The unique combination identifying *what should be captured where*. The work queue, deduplication, and capture history all revolve around this concept. Concretely: `(shader_version_id, scene_id, profile)`. Staleness is computed by comparing a Capture's `world_version_id` against the latest WorldVersion for the Scene's World. |
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
| **AutonomousRunner**      | 1     | The top-level agent loop. Fetches work from the backend, creates a CaptureRun, delegates to the Orchestrator, uploads results, and repeats until no work remains. |
| **Orchestrator**          | 2     | Manages multi-world, multi-scene capture within a single CaptureRun. Loads worlds, iterates scenes, and delegates each scene to a CaptureSession. Produces an OrchestrationManifest on completion. |
| **CaptureSession**        | 3     | Handles capturing all shaders for a single Scene. Manages the state machine: apply scene environment, then for each shader: load → stabilize → screenshot. Restores original state on completion. |
| **Stabilization**         | —     | The process of waiting for the game to reach a visually stable state before capturing. Includes chunk loading, FPS settling, and render pipeline convergence. Affects capture quality and timing. |
| **OrchestrationManifest** | —     | A JSON file written to the output directory after an Orchestrator run. Contains metadata: timing, scenes captured, screenshots produced, shader/Minecraft/Iris versions. |

### Work & Run Concepts

| Term                | Definition |
|---------------------|------------|
| **WorkItem**        | A denormalized packet returned by `GET /api/work` containing everything the mod needs to capture a single CaptureTarget: shader download URL, scene definition JSON, world file URL, profile, etc. Eliminates additional API lookups. |
| **CaptureRunItem**  | A single unit of work within a CaptureRun: one CaptureTarget to process. Tracks status, timing, error details, and links to the resulting Capture on success. |
| **CaptureSpec**     | The input to the Orchestrator: which scenes to capture, which shaders, output directory, and whether to shut down on completion. Used for both interactive and autonomous captures. Mod-only concept. |
| **ShaderSpec**      | A shader filename + optional Iris profile, used within CaptureSpec to specify what to render. Maps to the mod's local shader pack files. Mod-only concept. |

## Authentication

| Term                 | Definition |
|----------------------|------------|
| **Device Code Flow** | OAuth 2.0 Device Authorization Grant (RFC 8628) used by the Minecraft mod. The mod requests a device code, displays a user code (`GLINT-XXXXXX` format), the user authorizes in their browser, and the mod exchanges for a session token. |
| **Agent**            | The identity of a mod instance performing captures. Stored as `agent_id` on CaptureRuns. Currently a free-form string; may become a formal entity. |
| **Session**          | A database-backed authentication token. Has a source (`web` for browser login, `device` for mod login) and an expiration timestamp. |

## Scene Configuration (Mod)

| Term                | Definition |
|---------------------|------------|
| **SceneCollection** | A JSON file defining all scenes for a single World, stored at `.minecraft/glint/scenes/<world_name>.json`. Contains world metadata, a default SceneConfig, and a list of Scenes. |
| **SceneConfig**     | Render and capture settings: render distance, graphics mode, FOV, resolution, particles, clouds, etc. Supports inheritance — a scene's config merges with the collection's `defaultConfig` as fallback. |
| **SceneVariant**    | An override layer on a Scene that modifies specific properties (time of day, weather, etc.) to create alternative versions of the same location. Used for day/night, seasonal, or weather variations. |
| **SceneEntity**     | An entity definition within a SceneCollection for reproducible scene setup (placing specific mobs or items). |

## Status Lifecycles

| Context        | States | Notes |
|----------------|--------|-------|
| Capture        | `pending` → `uploading` → `completed`, or `pending` → `failed` | `uploading` is transient during file upload |
| CaptureRun     | `running` → `completed`, or `running` → `failed` | Set when all items are processed |
| CaptureRunItem | `pending` → `completed`, `pending` → `failed`, or `pending` → `skipped` | `skipped` = valid capture already existed |
| DeviceCode     | `pending` → `authorized` → `used` | RFC 8628 lifecycle |

## Relationships

```
World
├── WorldVersion (one-to-many, ordered by created_at)
└── Scene (one-to-many)
    └── Tag (many-to-many)

Shader
├── ShaderVersion (one-to-many)
│   └── Profile (discovered, stored as JSON array)
├── ShaderAuthor (one-to-many)
├── Category (many-to-many)
└── Feature (many-to-many)

CaptureTarget = (ShaderVersion + Scene + Profile)
└── Capture (one-to-many, ordered by captured_at)
    ├── Latest Capture (derived: most recent per target)
    └── WorldVersion (many-to-one, records which version was active)

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
| "outdated capture" | stale capture | A capture whose `world_version_id` doesn't match the latest WorldVersion. Staleness is always computed at query time, never stored as a flag. |
