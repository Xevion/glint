package com.xevion.glint.scene

import com.xevion.glint.Loggers
import net.minecraft.client.Minecraft
import net.minecraft.world.level.GameRules
import net.minecraft.client.CameraType as MinecraftCameraType

/**
 * Applies scene settings to the Minecraft client.
 * Handles world state, player positioning, camera, and render settings.
 */
object SceneApplicator {
    private val log = Loggers.Scene.get()

    /**
     * Applies a resolved scene to the game.
     * Returns true if successful, false if failed.
     */
    fun apply(resolvedScene: ResolvedScene): Boolean {
        val mc = Minecraft.getInstance()
        val scene = resolvedScene.scene
        val config = resolvedScene.config

        // Scene system only works in single-player
        if (mc.singleplayerServer == null) {
            log.error("Scene system is not available in multiplayer")
            return false
        }

        log.info("Applying scene") {
            "scene_id" to scene.id
            "name" to scene.name
        }

        // 1. Validate world is loaded
        if (!validateWorld(resolvedScene.worldFolderName)) {
            log.error("World not loaded") { "world" to resolvedScene.worldFolderName }
            return false
        }

        // 2. Set position and camera
        applyPositionAndCamera(scene, config)

        // 3. Set time and weather
        applyTimeAndWeather(scene)

        // 4. Apply render settings
        applyRenderSettings(config)

        // 5. Freeze world state
        freezeWorldState(config)

        // 6. Force chunk reload
        mc.levelRenderer.allChanged()

        log.debug("Scene applied successfully")
        return true
    }

    private fun validateWorld(worldName: String): Boolean {
        val mc = Minecraft.getInstance()
        if (mc.level == null) {
            return false
        }

        // For now, we just check if a world is loaded
        // TODO: Actually validate world name matches when we implement world loading
        return true
    }

    private fun applyPositionAndCamera(
        scene: Scene,
        config: SceneConfig,
    ) {
        val mc = Minecraft.getInstance()
        val player = mc.player
        if (player == null) {
            log.warn("Cannot apply position and camera - player is null")
            return
        }

        // Teleport player
        player.moveTo(
            scene.position.x,
            scene.position.y,
            scene.position.z,
            scene.camera.yaw,
            scene.camera.pitch,
        )

        // Set camera type
        val cameraType = config.cameraType ?: CameraType.FIRST_PERSON
        mc.options.setCameraType(
            when (cameraType) {
                CameraType.FIRST_PERSON -> MinecraftCameraType.FIRST_PERSON
                CameraType.THIRD_PERSON_BACK -> MinecraftCameraType.THIRD_PERSON_BACK
                CameraType.THIRD_PERSON_FRONT -> MinecraftCameraType.THIRD_PERSON_FRONT
            },
        )

        log.debug("Teleported") {
            "x" to scene.position.x
            "y" to scene.position.y
            "z" to scene.position.z
            "yaw" to scene.camera.yaw
            "pitch" to scene.camera.pitch
        }
    }

    private fun applyTimeAndWeather(scene: Scene) {
        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer

        if (server == null) {
            log.warn("Cannot apply time and weather - not in single-player")
            return
        }

        // Set time of day on server
        val overworld = server.overworld()
        overworld.dayTime = scene.timeOfDay.toLong()
        log.debug("Set server time") { "time" to scene.timeOfDay }

        // Set weather on server using the public API
        when (scene.weather) {
            Weather.CLEAR -> {
                // setWeatherParameters(clearWeatherTime, rainTime, isRaining, isThundering)
                overworld.setWeatherParameters(6000, 0, false, false)
                log.debug("Set weather") { "weather" to "CLEAR" }
            }

            Weather.RAIN -> {
                // setWeatherParameters(clearWeatherTime, rainTime, isRaining, isThundering)
                overworld.setWeatherParameters(0, 6000, true, false)
                log.debug("Set weather") { "weather" to "RAIN" }

                // Note: Weather intensity (rainLevel/thunderLevel) is protected in Level class
                // It's automatically calculated by the game based on isRaining/isThundering state
                // Manual intensity control would require mixins or reflection
                if (scene.weatherIntensity != null) {
                    log.warn("Weather intensity setting not supported (requires protected field access)")
                }
            }

            Weather.THUNDER -> {
                // setWeatherParameters(clearWeatherTime, rainTime, isRaining, isThundering)
                overworld.setWeatherParameters(0, 6000, true, true)
                log.debug("Set weather") { "weather" to "THUNDER" }

                if (scene.weatherIntensity != null) {
                    log.warn("Weather intensity setting not supported (requires protected field access)")
                }
            }
        }
    }

    private fun applyRenderSettings(config: SceneConfig) {
        val mc = Minecraft.getInstance()
        val options = mc.options

        // Camera & View
        config.fov?.let { options.fov().set(it) }
        config.viewBobbing?.let { options.bobView().set(it) }

        // Render Settings
        config.renderDistance?.let { options.renderDistance().set(it) }
        config.simulationDistance?.let { options.simulationDistance().set(it) }
        config.entityDistanceScaling?.let { options.entityDistanceScaling().set(it) }

        config.graphicsMode?.let {
            options.graphicsMode().set(
                when (it) {
                    GraphicsMode.FAST -> net.minecraft.client.GraphicsStatus.FAST
                    GraphicsMode.FANCY -> net.minecraft.client.GraphicsStatus.FANCY
                    GraphicsMode.FABULOUS -> net.minecraft.client.GraphicsStatus.FABULOUS
                },
            )
        }

        config.particles?.let {
            options.particles().set(
                when (it) {
                    ParticleMode.ALL -> net.minecraft.server.level.ParticleStatus.ALL
                    ParticleMode.DECREASED -> net.minecraft.server.level.ParticleStatus.DECREASED
                    ParticleMode.MINIMAL -> net.minecraft.server.level.ParticleStatus.MINIMAL
                    ParticleMode.OFF -> net.minecraft.server.level.ParticleStatus.MINIMAL // MC doesn't have OFF
                },
            )
        }

        config.clouds?.let {
            options.cloudStatus().set(
                when (it) {
                    CloudMode.OFF -> net.minecraft.client.CloudStatus.OFF
                    CloudMode.FAST -> net.minecraft.client.CloudStatus.FAST
                    CloudMode.FANCY -> net.minecraft.client.CloudStatus.FANCY
                },
            )
        }

        config.ambientOcclusion?.let { options.ambientOcclusion().set(it) }
        config.entityShadows?.let { options.entityShadows().set(it) }
        config.biomeBlend?.let { options.biomeBlendRadius().set(it) }
        config.brightness?.let { options.gamma().set(it) }
        config.guiScale?.let { options.guiScale().set(it) }
        config.hideHud?.let { options.hideGui = it }
        config.screenEffects?.let { options.screenEffectScale().set(it) }
        config.fovEffects?.let { options.fovEffectScale().set(it) }
        config.mipmapLevels?.let { options.mipmapLevels().set(it) }

        log.debug("Render settings applied")
    }

    private fun freezeWorldState(config: SceneConfig) {
        val mc = Minecraft.getInstance()

        // Note: GameRules can only be modified server-side
        // For single-player, we need to access the integrated server
        // For multiplayer, this won't work (server controls game rules)

        mc.singleplayerServer?.let { server ->
            server
                .overworld()
                .gameRules
                .getRule(GameRules.RULE_DAYLIGHT)
                .set(false, server)
            server
                .overworld()
                .gameRules
                .getRule(GameRules.RULE_WEATHER_CYCLE)
                .set(false, server)
            log.debug("World state frozen (single-player)")
        } ?: run {
            log.warn("Cannot freeze world state on multiplayer client")
        }

        // TODO: Entity freezing
        // This requires more complex logic:
        // - Snapshot current entities
        // - Despawn entities not in scene definition
        // - Spawn/teleport scene entities to exact positions
        // - Disable AI ticking
        val entityAiFrozen = config.entityAiFrozen ?: true
        if (entityAiFrozen) {
            log.debug("Entity AI freezing not yet implemented")
        }
    }
}
