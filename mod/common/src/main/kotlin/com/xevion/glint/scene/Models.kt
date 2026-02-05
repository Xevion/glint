package com.xevion.glint.scene

import com.xevion.glint.screenshot.Camera
import com.xevion.glint.screenshot.Position
import kotlinx.serialization.Serializable

/**
 * A collection of scenes for a single world.
 * File format: .minecraft/glint/scenes/<world_name>.json
 */
@Serializable
data class SceneCollection(
    val world: String,
    val folder: String? = null,
    val version: String = "1.21.4",
    val description: String? = null,
    val defaultConfig: SceneConfig? = null,
    val scenes: List<Scene>,
    val entities: List<SceneEntity> = emptyList(),
)

/**
 * A single scene definition within a world.
 */
@Serializable
data class Scene(
    val id: String,
    val name: String,
    val description: String? = null,
    val dimension: String = "minecraft:overworld",
    val position: Position,
    val camera: Camera,
    val timeOfDay: Int,
    val weather: Weather = Weather.CLEAR,
    val weatherIntensity: Float = 0.0f,
    val moonPhase: Int? = null,
    val biome: String? = null,
    val tags: List<String> = emptyList(),
    val config: SceneConfig? = null,
    val variants: List<SceneVariant> = emptyList(),
)

/**
 * A variant of a scene that modifies specific settings.
 * Inherits all base scene properties and overrides specific fields.
 */
@Serializable
data class SceneVariant(
    val id: String,
    val name: String,
    val description: String? = null,
    val timeOfDay: Int? = null,
    val weather: Weather? = null,
    val weatherIntensity: Float? = null,
    val moonPhase: Int? = null,
    val tags: List<String> = emptyList(),
    val config: SceneConfig? = null,
) {
    /**
     * Applies this variant's overrides to a base scene, creating a new Scene instance.
     */
    fun applyTo(base: Scene): Scene =
        base.copy(
            id = id,
            name = name,
            description = description ?: base.description,
            timeOfDay = timeOfDay ?: base.timeOfDay,
            weather = weather ?: base.weather,
            weatherIntensity = weatherIntensity ?: base.weatherIntensity,
            moonPhase = moonPhase ?: base.moonPhase,
            tags = if (tags.isNotEmpty()) tags else base.tags,
            config = config?.mergeWith(base.config) ?: base.config,
            variants = emptyList(),
        )
}

/**
 * Render and capture configuration for a scene.
 * All fields are optional - missing values use defaults.
 */
@Serializable
data class SceneConfig(
    // Camera & View
    val cameraType: CameraType? = null,
    val fov: Int? = null,
    val viewBobbing: Boolean? = null,
    // Render Settings
    val renderDistance: Int? = null,
    val simulationDistance: Int? = null,
    val entityDistanceScaling: Double? = null,
    val graphicsMode: GraphicsMode? = null,
    val particles: ParticleMode? = null,
    val clouds: CloudMode? = null,
    val ambientOcclusion: Boolean? = null,
    val entityShadows: Boolean? = null,
    val biomeBlend: Int? = null,
    // Display & Post-Processing
    val brightness: Double? = null,
    val guiScale: Int? = null,
    val hideHud: Boolean? = null,
    val screenEffects: Double? = null,
    val fovEffects: Double? = null,
    // Resolution
    val screenshotWidth: Int? = null,
    val screenshotHeight: Int? = null,
    // Advanced
    val mipmapLevels: Int? = null,
    val entityAiFrozen: Boolean? = null,
    val hidePlayerInThirdPerson: Boolean? = null,
) {
    /**
     * Merges this config with another, preferring this config's values.
     * Used for inheritance: scene.config.mergeWith(collection.defaultConfig)
     */
    fun mergeWith(fallback: SceneConfig?): SceneConfig {
        if (fallback == null) return this
        return SceneConfig(
            cameraType = cameraType ?: fallback.cameraType,
            fov = fov ?: fallback.fov,
            viewBobbing = viewBobbing ?: fallback.viewBobbing,
            renderDistance = renderDistance ?: fallback.renderDistance,
            simulationDistance = simulationDistance ?: fallback.simulationDistance,
            entityDistanceScaling = entityDistanceScaling ?: fallback.entityDistanceScaling,
            graphicsMode = graphicsMode ?: fallback.graphicsMode,
            particles = particles ?: fallback.particles,
            clouds = clouds ?: fallback.clouds,
            ambientOcclusion = ambientOcclusion ?: fallback.ambientOcclusion,
            entityShadows = entityShadows ?: fallback.entityShadows,
            biomeBlend = biomeBlend ?: fallback.biomeBlend,
            brightness = brightness ?: fallback.brightness,
            guiScale = guiScale ?: fallback.guiScale,
            hideHud = hideHud ?: fallback.hideHud,
            screenEffects = screenEffects ?: fallback.screenEffects,
            fovEffects = fovEffects ?: fallback.fovEffects,
            screenshotWidth = screenshotWidth ?: fallback.screenshotWidth,
            screenshotHeight = screenshotHeight ?: fallback.screenshotHeight,
            mipmapLevels = mipmapLevels ?: fallback.mipmapLevels,
            entityAiFrozen = entityAiFrozen ?: fallback.entityAiFrozen,
            hidePlayerInThirdPerson = hidePlayerInThirdPerson ?: fallback.hidePlayerInThirdPerson,
        )
    }

    companion object {
        /**
         * Default configuration optimized for shader comparison screenshots.
         */
        val DEFAULT =
            SceneConfig(
                cameraType = CameraType.FIRST_PERSON,
                fov = 70,
                viewBobbing = false,
                renderDistance = 16,
                simulationDistance = 8,
                entityDistanceScaling = 1.0,
                graphicsMode = GraphicsMode.FABULOUS,
                particles = ParticleMode.ALL,
                clouds = CloudMode.FANCY,
                ambientOcclusion = true,
                entityShadows = true,
                biomeBlend = 5,
                brightness = 0.5,
                guiScale = 0,
                hideHud = true,
                screenEffects = 1.0,
                fovEffects = 1.0,
                screenshotWidth = null,
                screenshotHeight = null,
                mipmapLevels = 4,
                entityAiFrozen = true,
                hidePlayerInThirdPerson = false,
            )
    }
}

@Serializable
enum class CameraType {
    FIRST_PERSON,
    THIRD_PERSON_BACK,
    THIRD_PERSON_FRONT,
}

@Serializable
enum class GraphicsMode {
    FAST,
    FANCY,
    FABULOUS,
}

@Serializable
enum class ParticleMode {
    ALL,
    DECREASED,
    MINIMAL,
    OFF,
}

@Serializable
enum class CloudMode {
    OFF,
    FAST,
    FANCY,
}

@Serializable
enum class Weather {
    CLEAR,
    RAIN,
    THUNDER,
    ;

    fun toMinecraftString(): String = name.lowercase()

    companion object {
        fun fromString(value: String): Weather =
            when (value.lowercase()) {
                "clear" -> CLEAR
                "rain" -> RAIN
                "thunder" -> THUNDER
                else -> CLEAR
            }
    }
}

/**
 * Entity definition for reproducible scene setup.
 */
@Serializable
data class SceneEntity(
    val type: String,
    val position: Position,
    val rotation: Camera,
    val nbt: String? = null,
    val frozenForScenes: List<String> = emptyList(),
)
