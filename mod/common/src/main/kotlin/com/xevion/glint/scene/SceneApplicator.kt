package com.xevion.glint.scene

import com.xevion.glint.Loggers
import net.minecraft.client.Minecraft
import net.minecraft.world.level.GameRules
import net.minecraft.world.level.GameType
import net.minecraft.world.phys.Vec3
import net.minecraft.client.CameraType as MinecraftCameraType

/** Result of applying a scene. */
enum class SceneApplyResult {
    /** Scene application failed. */
    FAILED,

    /** Scene applied, no chunk rebuild was triggered. */
    APPLIED,

    /** Scene applied, a full chunk rebuild was triggered (allChanged). */
    APPLIED_WITH_REBUILD,
}

/**
 * Applies scene settings to the Minecraft client.
 * Handles world state, player positioning, camera, and render settings.
 */
object SceneApplicator {
    private val log = Loggers.Scene.get()

    /**
     * Applies a resolved scene to the game.
     *
     * Returns [SceneApplyResult.APPLIED_WITH_REBUILD] when render settings triggered
     * [net.minecraft.client.renderer.LevelRenderer.allChanged], meaning the caller
     * should use full stabilization (including rendering pipeline checks).
     */
    fun apply(resolvedScene: ResolvedScene): SceneApplyResult {
        val mc = Minecraft.getInstance()
        val scene = resolvedScene.scene
        val config = resolvedScene.config

        // Scene system only works in single-player
        if (mc.singleplayerServer == null) {
            log.error("Scene system is not available in multiplayer")
            return SceneApplyResult.FAILED
        }

        log.info("Applying scene") {
            "scene_id" to scene.id
            "name" to scene.name
        }

        // 1. Validate world is loaded
        if (!validateWorld(resolvedScene.worldFolderName)) {
            log.error("World not loaded") { "world" to resolvedScene.worldFolderName }
            return SceneApplyResult.FAILED
        }

        // 2. Set position and camera
        applyPositionAndCamera(scene, config)

        // 3. Set time and weather (boolean flags + countdown timers)
        applyTimeAndWeather(scene)

        // 4. Apply render settings (only changes what differs from current state)
        val rebuildTriggered = applyRenderSettings(config)

        // 5. Freeze world state (disables daylight cycle and weather cycle)
        freezeWorldState(config)

        // 6. Snap weather levels AFTER freeze so the server's advanceWeatherCycle
        //    doesn't detect a stale state change and send packets that undo the snap
        snapWeatherLevels(scene)

        // 7. Only force a full chunk reload if a rebuild-triggering setting changed.
        //    Individual option setters for graphicsMode, ambientOcclusion, and biomeBlend
        //    already call allChanged() via Minecraft's internal listeners, but an explicit
        //    call ensures consistency. When settings are unchanged, skip entirely to avoid
        //    destroying and recreating Sodium's RenderSectionManager unnecessarily.
        if (rebuildTriggered) {
            mc.levelRenderer.allChanged()
            log.debug("Scene applied with chunk rebuild")
        } else {
            log.debug("Scene applied successfully")
        }

        return if (rebuildTriggered) SceneApplyResult.APPLIED_WITH_REBUILD else SceneApplyResult.APPLIED
    }

    private fun validateWorld(worldName: String): Boolean {
        val mc = Minecraft.getInstance()
        if (mc.level == null || mc.singleplayerServer == null) {
            return false
        }

        val loadedName = mc.singleplayerServer?.worldData?.levelName
        if (loadedName != worldName) {
            log.error("Wrong world loaded") {
                "expected" to worldName
                "actual" to (loadedName ?: "null")
            }
            return false
        }
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

        // Switch to spectator mode so the player is invulnerable, flying,
        // immune to gravity, and invisible to mobs.
        mc.singleplayerServer?.let { server ->
            val serverPlayer = server.playerList.getPlayer(player.uuid)
            if (serverPlayer != null && !serverPlayer.isSpectator) {
                serverPlayer.setGameMode(GameType.SPECTATOR)
                log.debug("Set spectator mode for capture")
            }
        }

        // Teleport player and zero velocity to prevent any residual drift
        player.setDeltaMovement(Vec3.ZERO)
        player.moveTo(
            scene.position.x,
            scene.position.y,
            scene.position.z,
            scene.camera.yaw,
            scene.camera.pitch,
        )
        player.setDeltaMovement(Vec3.ZERO)

        // Also zero velocity on the server-side player entity
        mc.singleplayerServer?.let { server ->
            val serverPlayer = server.playerList.getPlayer(player.uuid)
            serverPlayer?.setDeltaMovement(Vec3.ZERO)
        }

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
                overworld.setWeatherParameters(6000, 0, false, false)
                log.debug("Set weather") { "weather" to "CLEAR" }
            }

            Weather.RAIN -> {
                overworld.setWeatherParameters(0, 6000, true, false)
                log.debug("Set weather") { "weather" to "RAIN" }
            }

            Weather.THUNDER -> {
                overworld.setWeatherParameters(0, 6000, true, true)
                log.debug("Set weather") { "weather" to "THUNDER" }
            }
        }
    }

    /** Compute the target rain/thunder levels for a scene. */
    private fun weatherLevelsFor(scene: Scene): Pair<Float, Float> {
        val intensity = scene.weatherIntensity
        return when (scene.weather) {
            Weather.CLEAR -> {
                0f to 0f
            }

            Weather.RAIN -> {
                (if (intensity > 0f) intensity else 1f) to 0f
            }

            Weather.THUNDER -> {
                val level = if (intensity > 0f) intensity else 1f
                level to level
            }
        }
    }

    /**
     * Snaps weather levels on both server and client to the scene's target values.
     *
     * Must be called after [applyTimeAndWeather] and [freezeWorldState].
     * Also safe to call again later to counteract stale weather packets —
     * the server's `advanceWeatherCycle` sends `START_RAINING`/`STOP_RAINING`
     * packets when it detects a state change, and the client handler
     * counterintuitively resets `rainLevel` (e.g. `STOP_RAINING` sets it to 1.0
     * for gradual fadeout), overwriting our previous snap.
     */
    fun snapWeatherLevels(scene: Scene) {
        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer ?: return

        val (targetRainLevel, targetThunderLevel) = weatherLevelsFor(scene)

        val overworld = server.overworld()
        overworld.setRainLevel(targetRainLevel)
        overworld.setThunderLevel(targetThunderLevel)
        mc.level?.let { clientLevel ->
            clientLevel.setRainLevel(targetRainLevel)
            clientLevel.setThunderLevel(targetThunderLevel)
        }

        log.debug("Weather levels snapped") {
            "rain" to targetRainLevel
            "thunder" to targetThunderLevel
        }
    }

    /**
     * Applies render settings, skipping values that already match.
     *
     * @return true if any setting that triggers [net.minecraft.client.renderer.LevelRenderer.allChanged]
     *   was changed (graphicsMode, ambientOcclusion, biomeBlend), meaning a full chunk rebuild
     *   will occur via Minecraft's internal option listeners.
     */
    private fun applyRenderSettings(config: SceneConfig): Boolean {
        val mc = Minecraft.getInstance()
        val options = mc.options
        var rebuildTriggered = false
        var settingsChanged = 0

        // Helper: only set an option if the value differs from current
        fun <T> setIfChanged(
            option: net.minecraft.client.OptionInstance<T>,
            desired: T?,
            triggersRebuild: Boolean = false,
        ) {
            if (desired == null) return
            if (option.get() == desired) return
            option.set(desired)
            settingsChanged++
            if (triggersRebuild) rebuildTriggered = true
        }

        // Camera & View
        setIfChanged(options.fov(), config.fov)
        setIfChanged(options.bobView(), config.viewBobbing)

        // Render Settings
        setIfChanged(options.renderDistance(), config.renderDistance)
        setIfChanged(options.simulationDistance(), config.simulationDistance)
        setIfChanged(options.entityDistanceScaling(), config.entityDistanceScaling)

        config.graphicsMode?.let {
            val desired =
                when (it) {
                    GraphicsMode.FAST -> net.minecraft.client.GraphicsStatus.FAST
                    GraphicsMode.FANCY -> net.minecraft.client.GraphicsStatus.FANCY
                    GraphicsMode.FABULOUS -> net.minecraft.client.GraphicsStatus.FABULOUS
                }
            setIfChanged(options.graphicsMode(), desired, triggersRebuild = true)
        }

        config.particles?.let {
            val desired =
                when (it) {
                    ParticleMode.ALL -> net.minecraft.server.level.ParticleStatus.ALL
                    ParticleMode.DECREASED -> net.minecraft.server.level.ParticleStatus.DECREASED
                    ParticleMode.MINIMAL -> net.minecraft.server.level.ParticleStatus.MINIMAL
                    ParticleMode.OFF -> net.minecraft.server.level.ParticleStatus.MINIMAL
                }
            setIfChanged(options.particles(), desired)
        }

        config.clouds?.let {
            val desired =
                when (it) {
                    CloudMode.OFF -> net.minecraft.client.CloudStatus.OFF
                    CloudMode.FAST -> net.minecraft.client.CloudStatus.FAST
                    CloudMode.FANCY -> net.minecraft.client.CloudStatus.FANCY
                }
            setIfChanged(options.cloudStatus(), desired)
        }

        setIfChanged(options.ambientOcclusion(), config.ambientOcclusion, triggersRebuild = true)
        setIfChanged(options.entityShadows(), config.entityShadows)
        setIfChanged(options.biomeBlendRadius(), config.biomeBlend, triggersRebuild = true)
        setIfChanged(options.gamma(), config.brightness)
        setIfChanged(options.guiScale(), config.guiScale)
        config.hideHud?.let {
            if (options.hideGui != it) {
                options.hideGui = it
                settingsChanged++
            }
        }
        setIfChanged(options.screenEffectScale(), config.screenEffects)
        setIfChanged(options.fovEffectScale(), config.fovEffects)
        setIfChanged(options.mipmapLevels(), config.mipmapLevels)

        if (settingsChanged > 0) {
            log.debug("Render settings applied") {
                "changed" to settingsChanged
                "rebuild_triggered" to rebuildTriggered
            }
        } else {
            log.debug("Render settings unchanged, skipped")
        }

        return rebuildTriggered
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
