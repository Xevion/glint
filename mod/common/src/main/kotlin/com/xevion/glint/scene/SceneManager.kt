package com.xevion.glint.scene

import com.xevion.glint.Loggers
import com.xevion.glint.capture.Camera
import com.xevion.glint.capture.Position
import com.xevion.glint.io.AsyncFileIO
import kotlinx.serialization.json.Json
import net.minecraft.client.Minecraft
import java.io.File
import java.util.concurrent.CompletableFuture
import java.util.concurrent.ConcurrentHashMap

/**
 * Manages loading and applying scene configurations.
 */
object SceneManager {
    private val log = Loggers.Scene.get()

    private val JSON =
        Json {
            prettyPrint = true
            ignoreUnknownKeys = true
        }

    private val loadedCollections = ConcurrentHashMap<String, SceneCollection>()

    /**
     * Normalizes a world name into a safe filename.
     * Converts to lowercase, replaces spaces with underscores, removes unsafe characters.
     */
    private fun normalizeFileName(worldName: String): String =
        worldName
            .lowercase()
            .replace(' ', '_')
            .replace(Regex("[^a-z0-9_-]"), "")

    /**
     * Loads a scene by ID from available scene collections.
     *
     * Scene IDs are globally unique and searched across all collections.
     * Variants are automatically expanded during discovery.
     *
     * Examples:
     * - "village_sunrise" - searches all collections for a scene or variant with this ID
     * - "nether_fortress" - base scene ID
     */
    fun loadScene(sceneId: String): ResolvedScene? {
        log.debug("Loading scene") {
            "scene_id" to sceneId
        }

        val collections = discoverCollections()
        log.debug("Searching collections") {
            "count" to collections.size
            "scene_id" to sceneId
        }

        for ((fileName, collection) in collections) {
            // Search base scenes
            val baseScene = collection.scenes.find { it.id == sceneId }
            if (baseScene != null) {
                log.debug("Found base scene") {
                    "scene_id" to sceneId
                    "world" to collection.world
                }
                return resolveScene(baseScene, collection, fileName)
            }

            // Search variants
            for (scene in collection.scenes) {
                val variant = scene.variants.find { it.id == sceneId }
                if (variant != null) {
                    log.debug("Found variant") {
                        "variant_id" to sceneId
                        "base_scene_id" to scene.id
                        "world" to collection.world
                    }
                    val expandedScene = variant.applyTo(scene)
                    return resolveScene(expandedScene, collection, fileName)
                }
            }
        }

        log.warn("Scene not found") {
            "scene_id" to sceneId
        }
        return null
    }

    /**
     * Loads a scene collection from file synchronously (for legacy compatibility).
     * Prefer loadCollectionAsync for non-blocking operations.
     */
    fun loadCollection(worldName: String): SceneCollection? {
        loadedCollections[worldName]?.let { return it }

        val mc = Minecraft.getInstance()
        val sceneFile = File(mc.gameDirectory, "glint/scenes/$worldName.json")

        if (!sceneFile.exists()) {
            log.warn("Scene collection not found") {
                "path" to sceneFile.absolutePath
            }
            return null
        }

        return try {
            val collection = JSON.decodeFromString(SceneCollection.serializer(), sceneFile.readText())
            loadedCollections[worldName] = collection
            log.debug("Loaded scene collection") {
                "world" to worldName
                "scene_count" to collection.scenes.size
            }
            collection
        } catch (e: Exception) {
            log.error(e, "Failed to parse scene collection") {
                "path" to sceneFile.absolutePath
            }
            null
        }
    }

    /**
     * Loads a scene collection from file asynchronously.
     */
    fun loadCollectionAsync(worldName: String): CompletableFuture<SceneCollection?> {
        loadedCollections[worldName]?.let { return CompletableFuture.completedFuture(it) }

        val mc = Minecraft.getInstance()
        val sceneFile = File(mc.gameDirectory, "glint/scenes/$worldName.json")

        if (!sceneFile.exists()) {
            log.warn("Scene collection not found") {
                "path" to sceneFile.absolutePath
            }
            return CompletableFuture.completedFuture(null)
        }

        return AsyncFileIO
            .readJson(sceneFile, SceneCollection.serializer())
            .thenApply { collection ->
                loadedCollections[worldName] = collection
                log.debug("Loaded scene collection") {
                    "world" to worldName
                    "scene_count" to collection.scenes.size
                }
                collection
            }.exceptionally { e ->
                log.error(e, "Failed to parse scene collection") {
                    "path" to sceneFile.absolutePath
                }
                null
            }
    }

    /**
     * Discovers all available scene collections.
     * Returns map of filename (without .json) to SceneCollection.
     */
    private fun discoverCollections(): Map<String, SceneCollection> {
        val mc = Minecraft.getInstance()
        val scenesDir = File(mc.gameDirectory, "glint/scenes")

        if (!scenesDir.exists() || !scenesDir.isDirectory) {
            return emptyMap()
        }

        return scenesDir
            .listFiles { file -> file.extension == "json" }
            ?.mapNotNull { file ->
                val fileName = file.nameWithoutExtension
                loadCollection(fileName)?.let { fileName to it }
            }?.toMap() ?: emptyMap()
    }

    /**
     * Discovers all available scene collections (public API for orchestrator).
     * Returns list of collections with their filenames.
     */
    fun discoverAllCollections(): List<Pair<String, SceneCollection>> = discoverCollections().toList()

    /**
     * Resolves a scene with its config inheritance.
     */
    private fun resolveScene(
        scene: Scene,
        collection: SceneCollection,
        collectionFileName: String,
    ): ResolvedScene {
        val mergedConfig =
            (scene.config ?: SceneConfig())
                .mergeWith(collection.defaultConfig)
                .mergeWith(SceneConfig.DEFAULT)

        return ResolvedScene(
            scene = scene,
            collection = collection,
            config = mergedConfig,
            collectionFileName = collectionFileName,
        )
    }

    /**
     * Saves the current player state as a scene JSON (for quick scene creation).
     */
    fun saveCurrentStateAsScene(sceneId: String): String? {
        val mc = Minecraft.getInstance()

        // Scene system only works in single-player
        if (mc.singleplayerServer == null) {
            log.error("Scene capture is not available in multiplayer")
            return null
        }

        val player = mc.player
        if (player == null) {
            log.error("Cannot save scene - player is null")
            return null
        }

        val level = mc.level
        if (level == null) {
            log.error("Cannot save scene - level is null")
            return null
        }

        val position = Position(x = player.x, y = player.y, z = player.z)
        val camera = Camera(yaw = player.yRot, pitch = player.xRot)
        val dimension = level.dimension().location().toString()
        val timeOfDay = (level.dayTime % 24000).toInt()

        // Get world name (guaranteed to be non-null since we checked singleplayerServer above)
        val worldName = mc.singleplayerServer!!.worldData.levelName

        val scene =
            Scene(
                id = sceneId,
                name = sceneId.replace('_', ' ').replaceFirstChar { it.uppercase() },
                description = "Scene captured from current state",
                dimension = dimension,
                position = position,
                camera = camera,
                timeOfDay = timeOfDay,
                weather = if (level.levelData.isRaining) Weather.RAIN else Weather.CLEAR,
                weatherIntensity = 0.0f, // Cannot read rain intensity on client
            )

        val collection =
            SceneCollection(
                world = worldName,
                scenes = listOf(scene),
            )

        return JSON.encodeToString(SceneCollection.serializer(), collection)
    }

    /**
     * Adds a scene to a collection and persists to disk.
     * Creates the collection file if it doesn't exist.
     *
     * Uses normalized filename to prevent duplicate collections for the same world
     * (e.g., "New World" and "new_world" would use the same file).
     */
    fun addScene(
        worldName: String,
        scene: Scene,
    ): Boolean {
        val mc = Minecraft.getInstance()
        val scenesDir = File(mc.gameDirectory, "glint/scenes")
        scenesDir.mkdirs()

        val fileName = normalizeFileName(worldName)

        val collection =
            loadCollection(fileName)?.let {
                it.copy(scenes = it.scenes + scene)
            } ?: SceneCollection(world = worldName, folder = fileName, scenes = listOf(scene))

        return saveCollection(fileName, collection)
    }

    /**
     * Adds a scene to a collection and persists to disk asynchronously.
     */
    fun addSceneAsync(
        worldName: String,
        scene: Scene,
    ): CompletableFuture<Boolean> {
        val mc = Minecraft.getInstance()
        val scenesDir = File(mc.gameDirectory, "glint/scenes")
        scenesDir.mkdirs()

        val fileName = normalizeFileName(worldName)

        return loadCollectionAsync(fileName)
            .thenCompose { existingCollection ->
                val collection =
                    existingCollection?.let {
                        it.copy(scenes = it.scenes + scene)
                    } ?: SceneCollection(world = worldName, folder = fileName, scenes = listOf(scene))

                saveCollectionAsync(fileName, collection)
            }
    }

    /**
     * Removes a scene from a collection by ID.
     * The fileName parameter should be the normalized filename (without .json extension).
     */
    fun removeScene(
        fileName: String,
        sceneId: String,
    ): Boolean {
        val collection = loadCollection(fileName) ?: return false
        val updatedScenes = collection.scenes.filter { it.id != sceneId }

        if (updatedScenes.size == collection.scenes.size) {
            log.warn("Scene not found for deletion") {
                "scene_id" to sceneId
            }
            return false
        }

        val updatedCollection = collection.copy(scenes = updatedScenes)
        return saveCollection(fileName, updatedCollection)
    }

    /**
     * Disables a scene locally by removing it from the collection.
     * API-side disabling is handled exclusively through the push diff flow.
     */
    fun disableScene(
        fileName: String,
        sceneId: String,
    ): Boolean = removeScene(fileName, sceneId)

    /**
     * Updates the apiWorldId for a collection, linking it to a backend world.
     */
    fun setApiWorldId(
        fileName: String,
        apiWorldId: String,
    ): Boolean {
        val collection = loadCollection(fileName) ?: return false
        if (collection.apiWorldId == apiWorldId) return true
        return saveCollection(fileName, collection.copy(apiWorldId = apiWorldId))
    }

    /**
     * Creates an empty collection for a newly downloaded API world.
     * If a collection already exists, updates its apiWorldId and folder.
     */
    fun addCollectionForApiWorld(
        worldName: String,
        folder: String,
        apiWorldId: String,
    ): Boolean {
        val fileName = normalizeFileName(worldName)
        val existing = loadCollection(fileName)
        if (existing != null) {
            return saveCollection(fileName, existing.copy(apiWorldId = apiWorldId, folder = folder))
        }
        val collection =
            SceneCollection(
                world = worldName,
                folder = folder,
                apiWorldId = apiWorldId,
                scenes = emptyList(),
            )
        return saveCollection(fileName, collection)
    }

    /**
     * Saves a collection to disk synchronously.
     */
    internal fun saveCollection(
        worldName: String,
        collection: SceneCollection,
    ): Boolean {
        val mc = Minecraft.getInstance()
        val sceneFile = File(mc.gameDirectory, "glint/scenes/$worldName.json")

        return try {
            sceneFile.writeText(JSON.encodeToString(SceneCollection.serializer(), collection))
            loadedCollections[worldName] = collection
            log.debug("Saved scene collection") {
                "world" to worldName
            }
            true
        } catch (e: Exception) {
            log.error(e, "Failed to save scene collection") {
                "world" to worldName
            }
            false
        }
    }

    /**
     * Saves a collection to disk asynchronously.
     */
    private fun saveCollectionAsync(
        worldName: String,
        collection: SceneCollection,
    ): CompletableFuture<Boolean> {
        val mc = Minecraft.getInstance()
        val sceneFile = File(mc.gameDirectory, "glint/scenes/$worldName.json")

        return AsyncFileIO
            .writeJson(sceneFile, collection, SceneCollection.serializer())
            .thenApply {
                loadedCollections[worldName] = collection
                log.debug("Saved scene collection") {
                    "world" to worldName
                }
                true
            }.exceptionally { e ->
                log.error(e, "Failed to save scene collection") {
                    "world" to worldName
                }
                false
            }
    }

    /**
     * Clears cached scene collections (useful for reloading after file changes).
     */
    fun clearCache() {
        loadedCollections.clear()
        log.debug("Scene cache cleared")
    }

    /**
     * Checks if a scene ID already exists in any collection.
     * Searches both base scenes and variants.
     */
    fun sceneIdExists(sceneId: String): Boolean {
        val collections = discoverCollections()

        for ((_, collection) in collections) {
            // Check base scenes
            if (collection.scenes.any { it.id == sceneId }) {
                return true
            }

            // Check variants
            for (scene in collection.scenes) {
                if (scene.variants.any { it.id == sceneId }) {
                    return true
                }
            }
        }

        return false
    }
}

/**
 * A scene with fully resolved configuration (all inheritance applied).
 */
data class ResolvedScene(
    val scene: Scene,
    val collection: SceneCollection,
    val config: SceneConfig,
    val collectionFileName: String,
) {
    /**
     * Display name for the world (human-readable, from the collection manifest).
     */
    val worldDisplayName: String
        get() = collection.world

    /**
     * The world save folder name, used for loading worlds via Minecraft's openWorld().
     *
     * Resolved from the explicit `folder` field in the scene collection manifest,
     * falling back to the collection filename (the established naming convention).
     */
    val worldFolderName: String
        get() = collection.folder ?: collectionFileName

    /**
     * Entities applicable to this scene.
     */
    val entities: List<SceneEntity>
        get() =
            collection.entities.filter {
                it.frozenForScenes.isEmpty() || scene.id in it.frozenForScenes
            }
}
