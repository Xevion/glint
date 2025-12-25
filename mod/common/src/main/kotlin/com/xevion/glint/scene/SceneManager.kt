package com.xevion.glint.scene

import com.xevion.glint.Glint
import com.xevion.glint.io.AsyncFileIO
import com.xevion.glint.screenshot.Camera
import com.xevion.glint.screenshot.Position
import kotlinx.serialization.json.Json
import net.minecraft.client.Minecraft
import java.io.File
import java.util.concurrent.CompletableFuture
import java.util.concurrent.ConcurrentHashMap

/**
 * Manages loading and applying scene configurations.
 */
object SceneManager {
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
    private fun normalizeFileName(worldName: String): String = worldName
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
        Glint.LOGGER.info("Loading scene: $sceneId")

        val collections = discoverCollections()
        Glint.LOGGER.debug("Searching ${collections.size} collections for scene ID: $sceneId")

        for ((fileName, collection) in collections) {
            // Search base scenes
            val baseScene = collection.scenes.find { it.id == sceneId }
            if (baseScene != null) {
                Glint.LOGGER.debug("Found base scene '$sceneId' in '${collection.world}'")
                return resolveScene(baseScene, collection, fileName)
            }

            // Search variants
            for (scene in collection.scenes) {
                val variant = scene.variants.find { it.id == sceneId }
                if (variant != null) {
                    Glint.LOGGER.debug("Found variant '$sceneId' of scene '${scene.id}' in '${collection.world}'")
                    val expandedScene = variant.applyTo(scene)
                    return resolveScene(expandedScene, collection, fileName)
                }
            }
        }

        Glint.LOGGER.error("Scene not found: $sceneId")
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
            Glint.LOGGER.error("Scene collection not found: ${sceneFile.absolutePath}")
            return null
        }

        return try {
            val collection = JSON.decodeFromString(SceneCollection.serializer(), sceneFile.readText())
            loadedCollections[worldName] = collection
            Glint.LOGGER.info("Loaded scene collection: $worldName (${collection.scenes.size} scenes)")
            collection
        } catch (e: Exception) {
            Glint.LOGGER.error("Failed to parse scene collection: ${sceneFile.absolutePath}", e)
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
            Glint.LOGGER.error("Scene collection not found: ${sceneFile.absolutePath}")
            return CompletableFuture.completedFuture(null)
        }

        return AsyncFileIO
            .readJson(sceneFile, SceneCollection.serializer())
            .thenApply { collection ->
                loadedCollections[worldName] = collection
                Glint.LOGGER.info("Loaded scene collection: $worldName (${collection.scenes.size} scenes)")
                collection
            }.exceptionally { e ->
                Glint.LOGGER.error("Failed to parse scene collection: ${sceneFile.absolutePath}", e)
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
            Glint.LOGGER.error("Scene capture is not available in multiplayer")
            return null
        }

        val player = mc.player
        if (player == null) {
            Glint.LOGGER.error("Cannot save scene - player is null")
            return null
        }

        val level = mc.level
        if (level == null) {
            Glint.LOGGER.error("Cannot save scene - level is null")
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
            } ?: SceneCollection(world = worldName, scenes = listOf(scene))

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
                    } ?: SceneCollection(world = worldName, scenes = listOf(scene))

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
            Glint.LOGGER.warn("Scene not found for deletion: $sceneId")
            return false
        }

        val updatedCollection = collection.copy(scenes = updatedScenes)
        return saveCollection(fileName, updatedCollection)
    }

    /**
     * Saves a collection to disk synchronously.
     */
    private fun saveCollection(
        worldName: String,
        collection: SceneCollection,
    ): Boolean {
        val mc = Minecraft.getInstance()
        val sceneFile = File(mc.gameDirectory, "glint/scenes/$worldName.json")

        return try {
            sceneFile.writeText(JSON.encodeToString(SceneCollection.serializer(), collection))
            loadedCollections[worldName] = collection
            Glint.LOGGER.info("Saved scene collection: $worldName")
            true
        } catch (e: Exception) {
            Glint.LOGGER.error("Failed to save scene collection: $worldName", e)
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
                Glint.LOGGER.info("Saved scene collection: $worldName")
                true
            }.exceptionally { e ->
                Glint.LOGGER.error("Failed to save scene collection: $worldName", e)
                false
            }
    }

    /**
     * Clears cached scene collections (useful for reloading after file changes).
     */
    fun clearCache() {
        loadedCollections.clear()
        Glint.LOGGER.info("Scene cache cleared")
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
     * The world name from the collection manifest.
     */
    val worldName: String
        get() = collection.world

    /**
     * Entities applicable to this scene.
     */
    val entities: List<SceneEntity>
        get() =
            collection.entities.filter {
                it.frozenForScenes.isEmpty() || scene.id in it.frozenForScenes
            }
}
