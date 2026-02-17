package com.xevion.glint.scene

import com.xevion.glint.Loggers
import com.xevion.glint.api.GlintJsonFile
import kotlinx.serialization.Serializable
import net.minecraft.client.Minecraft
import java.io.File

/** Lightweight index loaded at startup for fast scene list population. */
@Serializable
data class SceneIndex(
    val scenes: Map<String, SceneIndexEntry> = emptyMap(),
)

@Serializable
data class SceneIndexEntry(
    val name: String,
    val dimension: String,
    val state: SceneState,
    val exportedAt: String,
    val packageSizeBytes: Long,
    val backendSceneId: String? = null,
    val latestVersionId: String? = null,
)

@Serializable
enum class SceneState {
    LOCAL,
    SYNCED,
}

/** Full per-scene metadata, loaded on-demand when a scene is selected. */
@Serializable
data class LocalSceneMetadata(
    val slug: String,
    val name: String,
    val description: String? = null,
    val dimension: String,
    val minecraftVersion: String,
    val state: SceneState,
    val exportedAt: String,
    val camera: CameraPosition,
    val fov: Int,
    val renderDistance: Int,
    val environment: PackageEnvironment,
    val chunkBounds: ChunkBounds,
    val entityCount: Int,
    val packageHash: String,
    val packageSizeBytes: Long,
    val backend: BackendSyncState? = null,
    val presets: List<LocalPreset> = emptyList(),
    val versions: List<LocalVersionEntry> = emptyList(),
)

@Serializable
data class BackendSyncState(
    val sceneId: String,
    val latestVersionId: String,
    val syncedAt: String,
    val syncedVersionHash: String,
)

@Serializable
data class LocalPreset(
    val name: String,
    val slug: String,
    val timeOfDayTicks: Int,
    val weather: String,
    val weatherIntensity: Double = 0.0,
    val moonPhase: Int? = null,
    val backendPresetId: String? = null,
)

@Serializable
data class LocalVersionEntry(
    val version: Int,
    val exportedAt: String,
    val packageHash: String,
    val sizeBytes: Long,
    val backendVersionId: String? = null,
)

/** Manages local scene storage under `.minecraft/glint/scenes/`. */
object LocalSceneStore {
    private val log = Loggers.Scene.get()
    private var index: SceneIndex? = null

    private val scenesDir: File by lazy {
        val mc = Minecraft.getInstance()
        File(mc.gameDirectory, "glint/scenes").also { it.mkdirs() }
    }

    /** Root directory: `.minecraft/glint/scenes/` */
    fun scenesDir(): File = scenesDir

    /** Cached load — only reads from disk on first call. Call [clearCache] to force reload. */
    fun loadIndex(): SceneIndex {
        index?.let { return it }
        val file = File(scenesDir(), "index.json")
        val loaded =
            if (file.exists()) {
                GlintJsonFile.decodeFromString(SceneIndex.serializer(), file.readText())
            } else {
                SceneIndex()
            }
        index = loaded
        return loaded
    }

    fun saveIndex() {
        val idx = index ?: return
        val file = File(scenesDir(), "index.json")
        file.writeText(GlintJsonFile.encodeToString(SceneIndex.serializer(), idx))
    }

    fun loadMetadata(slug: String): LocalSceneMetadata? {
        val file = File(scenesDir(), "$slug/local.json")
        if (!file.exists()) return null
        return try {
            GlintJsonFile.decodeFromString(LocalSceneMetadata.serializer(), file.readText())
        } catch (e: Exception) {
            log.error(e, "Failed to load scene metadata") { "slug" to slug }
            null
        }
    }

    fun saveMetadata(
        slug: String,
        metadata: LocalSceneMetadata,
    ) {
        val dir = File(scenesDir(), slug).also { it.mkdirs() }
        val file = File(dir, "local.json")
        file.writeText(GlintJsonFile.encodeToString(LocalSceneMetadata.serializer(), metadata))
    }

    fun packagePath(slug: String): File = File(scenesDir(), "$slug/package.zip")

    /** Register a newly exported scene in the index and save metadata. */
    fun registerExport(
        slug: String,
        meta: LocalSceneMetadata,
    ) {
        val idx = loadIndex()
        index =
            idx.copy(
                scenes =
                    idx.scenes +
                        (
                            slug to
                                SceneIndexEntry(
                                    name = meta.name,
                                    dimension = meta.dimension,
                                    state = SceneState.LOCAL,
                                    exportedAt = meta.exportedAt,
                                    packageSizeBytes = meta.packageSizeBytes,
                                )
                        ),
            )
        saveIndex()
        saveMetadata(slug, meta)
    }

    /** Update sync state after successful upload. */
    fun markSynced(
        slug: String,
        backendState: BackendSyncState,
    ) {
        val idx = loadIndex()
        val existing = idx.scenes[slug]
        if (existing == null) {
            log.warn("Cannot mark scene as synced — not found in index") { "slug" to slug }
            return
        }

        index =
            idx.copy(
                scenes =
                    idx.scenes +
                        (
                            slug to
                                existing.copy(
                                    state = SceneState.SYNCED,
                                    backendSceneId = backendState.sceneId,
                                    latestVersionId = backendState.latestVersionId,
                                )
                        ),
            )
        saveIndex()

        val metadata = loadMetadata(slug) ?: return
        saveMetadata(
            slug,
            metadata.copy(
                state = SceneState.SYNCED,
                backend = backendState,
            ),
        )
    }

    /** Check if a local scene needs pushing (hash mismatch or no backend state). */
    fun needsPush(slug: String): Boolean {
        val metadata = loadMetadata(slug) ?: return false
        val backend = metadata.backend ?: return true
        return metadata.packageHash != backend.syncedVersionHash
    }

    fun deleteScene(slug: String) {
        require(slug.isNotBlank() && !slug.contains("..")) { "Invalid slug: $slug" }
        val idx = loadIndex()
        index = idx.copy(scenes = idx.scenes - slug)
        saveIndex()
        val dir = File(scenesDir(), slug)
        if (dir.exists()) dir.deleteRecursively()
    }

    /** Clear cached index (force reload on next access). */
    fun clearCache() {
        index = null
    }

    fun allSlugs(): Set<String> = loadIndex().scenes.keys
}
