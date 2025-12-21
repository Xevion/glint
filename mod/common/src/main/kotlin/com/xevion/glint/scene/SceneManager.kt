package com.xevion.glint.scene

import com.xevion.glint.Glint
import com.xevion.glint.screenshot.Camera
import com.xevion.glint.screenshot.Position
import kotlinx.serialization.json.Json
import net.minecraft.client.Minecraft
import java.io.File

/**
 * Manages loading and applying scene configurations.
 */
object SceneManager {
    private val JSON =
        Json {
            prettyPrint = true
            ignoreUnknownKeys = true
        }

    private val loadedCollections = mutableMapOf<String, SceneCollection>()

    /**
     * Loads a scene by ID from available scene collections.
     *
     * Scene ID format: "<world_name>.<scene_id>" or just "<scene_id>" (searches all collections)
     *
     * Examples:
     * - "sunset_ocean.main" - loads scene "main" from sunset_ocean.json
     * - "main" - searches all collections for a scene with id "main"
     */
    fun loadScene(sceneId: String): ResolvedScene? {
        Glint.LOGGER.info("Loading scene: $sceneId")

        val (worldName, localSceneId) =
            if (sceneId.contains('.')) {
                val parts = sceneId.split('.', limit = 2)
                parts[0] to parts[1]
            } else {
                null to sceneId
            }

        Glint.LOGGER.debug("Parsed scene ID: world=$worldName, scene=$localSceneId")

        worldName?.let { world ->
            val collection = loadCollection(world) ?: return null
            val scene = collection.scenes.find { it.id == localSceneId }
            if (scene != null) {
                return resolveScene(scene, collection)
            }
        } ?: run {
            val collections = discoverCollections()
            Glint.LOGGER.debug("Searching ${collections.size} collections for scene")
            for (collection in collections) {
                val scene = collection.scenes.find { it.id == localSceneId }
                if (scene != null) {
                    Glint.LOGGER.debug("Found scene '$localSceneId' in '${collection.world}'")
                    return resolveScene(scene, collection)
                }
            }
        }

        Glint.LOGGER.error("Scene not found: $sceneId")
        return null
    }

    /**
     * Loads a scene collection from file.
     */
    private fun loadCollection(worldName: String): SceneCollection? {
        loadedCollections[worldName]?.let { return it }

        val mc = Minecraft.getInstance()
        val sceneFile = File(mc.gameDirectory, "glint_scenes/$worldName.json")

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
     * Discovers all available scene collections.
     */
    private fun discoverCollections(): List<SceneCollection> {
        val mc = Minecraft.getInstance()
        val scenesDir = File(mc.gameDirectory, "glint_scenes")

        if (!scenesDir.exists() || !scenesDir.isDirectory) {
            return emptyList()
        }

        return scenesDir
            .listFiles { file -> file.extension == "json" }
            ?.mapNotNull { file ->
                val worldName = file.nameWithoutExtension
                loadCollection(worldName)
            } ?: emptyList()
    }

    /**
     * Resolves a scene with its config inheritance.
     */
    private fun resolveScene(
        scene: Scene,
        collection: SceneCollection,
    ): ResolvedScene {
        val mergedConfig =
            (scene.config ?: SceneConfig())
                .mergeWith(collection.defaultConfig)
                .mergeWith(SceneConfig.DEFAULT)

        return ResolvedScene(
            scene = scene,
            collection = collection,
            config = mergedConfig,
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
     * Clears cached scene collections (useful for reloading after file changes).
     */
    fun clearCache() {
        loadedCollections.clear()
        Glint.LOGGER.info("Scene cache cleared")
    }
}

/**
 * A scene with fully resolved configuration (all inheritance applied).
 */
data class ResolvedScene(
    val scene: Scene,
    val collection: SceneCollection,
    val config: SceneConfig,
) {
    val worldName: String get() = collection.world
    val worldPath: String get() = "glint_scenes/${collection.world}"
    val entities: List<SceneEntity>
        get() =
            collection.entities.filter {
                it.frozenForScenes.isEmpty() || scene.id in it.frozenForScenes
            }
}
