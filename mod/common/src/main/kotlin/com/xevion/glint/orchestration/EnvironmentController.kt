package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.WorkItem
import com.xevion.glint.api.effectiveMoonPhase
import com.xevion.glint.api.effectiveTimeOfDayTicks
import com.xevion.glint.api.effectiveWeather
import com.xevion.glint.api.effectiveWeatherIntensity
import com.xevion.glint.capture.IrisIntegration
import net.minecraft.client.Minecraft
import net.minecraft.world.level.GameRules
import net.minecraft.world.level.GameType

/**
 * Owns all Minecraft client/server environment state manipulation during capture.
 *
 * Constructed fresh per orchestration run in [LinearOrchestrator.start].
 */
class EnvironmentController {
    private val log = Loggers.Orchestration.get()

    private var renderSettingsApplied: Boolean = false
    private var originalShaderPack: String? = null
    private val failedShaderPacks = mutableSetOf<String>()

    /**
     * Saves the current shader pack name so it can be restored on [close].
     */
    fun saveOriginalShader() {
        if (IrisIntegration.isAvailable) {
            originalShaderPack =
                if (IrisIntegration.isShaderPackInUse().getOrDefault(false)) {
                    IrisIntegration.getShaderPackName().getOrNull()
                } else {
                    null
                }
        }
    }

    /** Restores the original shader state saved by [saveOriginalShader]. */
    fun restoreOriginalShader() {
        if (IrisIntegration.isAvailable) {
            originalShaderPack?.let { IrisIntegration.enableShaders(it) }
                ?: IrisIntegration.disableShaders()
        }
    }

    /**
     * Loads or disables a shader pack via Iris.
     *
     * @return [Result.success] if the shader was loaded, [Result.failure] if it failed.
     */
    fun loadShader(spec: ShaderSpec): Result<Unit> {
        if (!IrisIntegration.isAvailable) return Result.success(Unit)

        val result =
            if (spec.filename == null) {
                IrisIntegration.disableShaders()
            } else {
                IrisIntegration.enableShaders(spec.filename, spec.profile)
            }

        return if (result.isFailure) {
            log.error("Failed to load shader") { "shader" to spec.displayName }
            if (spec.filename != null) {
                failedShaderPacks.add(spec.filename)
            }
            Result.failure(result.exceptionOrNull() ?: RuntimeException("Shader load failed"))
        } else {
            Result.success(Unit)
        }
    }

    fun isShaderFailed(filename: String): Boolean = filename in failedShaderPacks

    fun markShaderFailed(filename: String) {
        failedShaderPacks.add(filename)
    }

    /**
     * Applies the effective environment for a work item.
     * Preset values override scene defaults when present.
     */
    fun applyPresetEnvironment(item: WorkItem) {
        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer ?: return
        val overworld = server.overworld()

        val time = item.effectiveTimeOfDayTicks
        overworld.dayTime = time.toLong()

        val weather = item.effectiveWeather
        when (weather) {
            "clear" -> overworld.setWeatherParameters(6000, 0, false, false)
            "rain" -> overworld.setWeatherParameters(0, 6000, true, false)
            "thunder" -> overworld.setWeatherParameters(0, 6000, true, true)
        }

        applyWeatherSnap(item)

        val moonPhase = item.effectiveMoonPhase
        if (moonPhase != null) {
            val dayCount = moonPhase.toLong()
            overworld.dayTime = dayCount * 24000L + time.toLong()
        }
    }

    /** Snaps weather levels on both server and client to target values. */
    fun applyWeatherSnap(item: WorkItem) {
        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer ?: return
        val overworld = server.overworld()

        val weather = item.effectiveWeather
        val intensity = item.effectiveWeatherIntensity.toFloat()

        val (targetRain, targetThunder) =
            when (weather) {
                "clear" -> 0f to 0f
                "rain" -> (if (intensity > 0f) intensity else 1f) to 0f
                "thunder" -> {
                    val level = if (intensity > 0f) intensity else 1f
                    level to level
                }
                else -> 0f to 0f
            }

        overworld.setRainLevel(targetRain)
        overworld.setThunderLevel(targetThunder)
        mc.level?.let { clientLevel ->
            clientLevel.setRainLevel(targetRain)
            clientLevel.setThunderLevel(targetThunder)
        }
    }

    /**
     * Applies standardized render settings for captures.
     * Called once after the staging world is ready.
     */
    fun applyStandardRenderSettings() {
        val mc = Minecraft.getInstance()
        val options = mc.options

        options.graphicsMode().set(net.minecraft.client.GraphicsStatus.FANCY)
        options.ambientOcclusion().set(true)
        options.entityShadows().set(true)
        options.particles().set(net.minecraft.server.level.ParticleStatus.ALL)
        options.cloudStatus().set(net.minecraft.client.CloudStatus.FANCY)
        options.bobView().set(false)
        options.gamma().set(0.0)
        options.screenEffectScale().set(0.0)
        options.fovEffectScale().set(0.0)
        options.hideGui = true

        mc.singleplayerServer?.let { server ->
            val player = server.playerList.players.firstOrNull()
            if (player != null && !player.isSpectator) {
                player.setGameMode(GameType.SPECTATOR)
            }
        }

        renderSettingsApplied = true
        log.debug("Standard render settings applied")
    }

    /** Applies per-scene FOV and render distance from the work item. */
    fun applySceneViewSettings(item: WorkItem) {
        val mc = Minecraft.getInstance()
        mc.options.fov().set(item.scene.fov)
        mc.options.renderDistance().set(item.scene.renderDistance)
        mc.options.simulationDistance().set(item.scene.renderDistance)

        log.debug("Scene view settings applied") {
            "fov" to item.scene.fov
            "render_distance" to item.scene.renderDistance
        }
    }

    /** Freezes daylight cycle and weather cycle via game rules. */
    fun freezeWorldState() {
        val mc = Minecraft.getInstance()
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
            log.debug("World state frozen")
        }
    }

    /** Restores original shader and clears state. */
    fun close() {
        restoreOriginalShader()
        renderSettingsApplied = false
        originalShaderPack = null
        failedShaderPacks.clear()
    }
}
