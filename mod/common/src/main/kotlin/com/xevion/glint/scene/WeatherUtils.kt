package com.xevion.glint.scene

import net.minecraft.client.Minecraft
import net.minecraft.server.level.ServerLevel

const val WEATHER_DURATION_TICKS = 6000

/**
 * Sets the weather flags on [overworld] for this weather type.
 * Does NOT snap rain/thunder levels — call [Weather.snapLevels] after for instant effect.
 */
fun Weather.applyParameters(overworld: ServerLevel) {
    when (this) {
        Weather.CLEAR -> overworld.setWeatherParameters(WEATHER_DURATION_TICKS, 0, false, false)
        Weather.RAIN -> overworld.setWeatherParameters(0, WEATHER_DURATION_TICKS, true, false)
        Weather.THUNDER -> overworld.setWeatherParameters(0, WEATHER_DURATION_TICKS, true, true)
    }
}

/**
 * Computes the target rain and thunder levels for this weather type.
 * For [Weather.CLEAR], both are 0. For rain/thunder, uses [intensity] (defaults to 1.0).
 */
fun Weather.levels(intensity: Float = 1f): Pair<Float, Float> =
    when (this) {
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

/**
 * Snaps rain and thunder levels on both server and client for instant visual effect.
 * Call after [applyParameters] to avoid the gradual weather transition.
 */
fun Weather.snapLevels(
    overworld: ServerLevel,
    intensity: Float = 1f,
) {
    val (rain, thunder) = levels(intensity)
    overworld.setRainLevel(rain)
    overworld.setThunderLevel(thunder)
    Minecraft.getInstance().level?.let { clientLevel ->
        clientLevel.setRainLevel(rain)
        clientLevel.setThunderLevel(thunder)
    }
}
