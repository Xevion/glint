package com.xevion.glint.scene

import com.xevion.glint.Loggers
import net.minecraft.client.Minecraft
import net.minecraft.world.level.GameRules

/** Result of applying a scene. */
enum class SceneApplyResult {
    /** Scene application failed. */
    FAILED,

    /** Scene applied successfully. */
    APPLIED,
}

/**
 * Applies scene settings to the Minecraft client.
 */
object SceneApplicator {
    private val log = Loggers.Scene.get()

    /**
     * Applies environment settings (time, weather, moon phase) without a full scene.
     * Used by preset management to preview environment variants in the current world.
     */
    fun applyEnvironment(
        timeOfDay: Int,
        weather: Weather,
        weatherIntensity: Float,
        moonPhase: Int,
    ): SceneApplyResult {
        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer

        if (server == null) {
            log.error("Cannot apply environment - not in single-player")
            return SceneApplyResult.FAILED
        }

        val overworld = server.overworld()

        // Compute dayTime that yields the requested moon phase while preserving time of day.
        // Moon phase = (dayTime / 24000 % 8), so dayTime = moonPhase * 24000 + timeOfDay
        overworld.dayTime = (moonPhase.toLong() * 24000L) + timeOfDay.toLong()
        log.debug("Set environment time") {
            "time" to timeOfDay
            "moon_phase" to moonPhase
        }

        weather.applyParameters(overworld)

        // Freeze daylight and weather cycles
        overworld.gameRules.getRule(GameRules.RULE_DAYLIGHT).set(false, server)
        overworld.gameRules.getRule(GameRules.RULE_WEATHER_CYCLE).set(false, server)

        weather.snapLevels(overworld, weatherIntensity)

        log.info("Applied environment preset") {
            "time" to timeOfDay
            "weather" to weather.name
            "intensity" to weatherIntensity
            "moon_phase" to moonPhase
        }

        return SceneApplyResult.APPLIED
    }
}
