package com.xevion.glint.scene

import com.xevion.glint.Glint
import net.minecraft.client.Minecraft
import net.minecraft.world.level.GameRules
import net.minecraft.client.CameraType as MinecraftCameraType

/**
 * Applies scene settings to the Minecraft client.
 * Handles world state, player positioning, camera, and render settings.
 */
object SceneApplicator {
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
            Glint.LOGGER.error("Scene system is not available in multiplayer")
            return false
        }

        Glint.LOGGER.info("Applying scene: ${scene.name} (${scene.id})")

        // 1. Validate world is loaded
        if (!validateWorld(resolvedScene.worldName)) {
            Glint.LOGGER.error("World not loaded or doesn't match: ${resolvedScene.worldName}")
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

        Glint.LOGGER.info("Scene applied successfully")
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
            Glint.LOGGER.warn("Cannot apply position and camera - player is null")
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

        Glint.LOGGER.debug(
            "Teleported to (${scene.position.x}, ${scene.position.y}, ${scene.position.z}), " +
                "camera (${scene.camera.yaw}, ${scene.camera.pitch})",
        )
    }

    private fun applyTimeAndWeather(scene: Scene) {
        val mc = Minecraft.getInstance()
        val level = mc.level as? net.minecraft.client.multiplayer.ClientLevel
        if (level == null) {
            Glint.LOGGER.warn("Cannot apply time and weather - level is null")
            return
        }

        // Set time of day (uses the ClientLevelData inner class)
        val levelData = level.levelData
        if (levelData is net.minecraft.world.level.storage.WritableLevelData) {
            // Time cannot be set directly on client, but we can request it from server
            // For now, just log a warning - this needs server-side support
            Glint.LOGGER.warn("Time setting on client requires server-side support")
        }

        // Set weather (client-side is limited - only rain can be toggled)
        // Note: Thunder and weather intensity require server-side control
        when (scene.weather) {
            Weather.CLEAR -> {
                level.levelData.setRaining(false)
            }

            Weather.RAIN, Weather.THUNDER -> {
                level.levelData.setRaining(true)
                // Note: Cannot distinguish rain from thunder on client
                // Note: Weather intensity cannot be set on client
            }
        }

        Glint.LOGGER.debug("Set time: ${scene.timeOfDay}, weather: ${scene.weather.toMinecraftString()}")
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

        Glint.LOGGER.debug("Render settings applied")
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
            Glint.LOGGER.debug("World state frozen (single-player)")
        } ?: run {
            Glint.LOGGER.warn("Cannot freeze world state on multiplayer client")
        }

        // TODO: Entity freezing
        // This requires more complex logic:
        // - Snapshot current entities
        // - Despawn entities not in scene definition
        // - Spawn/teleport scene entities to exact positions
        // - Disable AI ticking
        val entityAiFrozen = config.entityAiFrozen ?: true
        if (entityAiFrozen) {
            Glint.LOGGER.debug("Entity AI freezing not yet implemented")
        }
    }
}
