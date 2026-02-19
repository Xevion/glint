package com.xevion.glint.scene

import com.xevion.glint.Loggers
import net.minecraft.client.CloudStatus
import net.minecraft.client.GraphicsStatus
import net.minecraft.client.Minecraft
import net.minecraft.server.level.ParticleStatus
import net.minecraft.world.level.GameRules

/**
 * Holds live environment overrides for the scene setup workflow.
 *
 * When [active], overrides are applied to the world via dedicated apply methods.
 * Settings persist across screen open/close cycles until [reset] is called.
 * The original world state is captured on [activate] and restored on [reset].
 */
object SceneSetupState {
    private val log = Loggers.Ui.get()

    var active: Boolean = false
        private set

    // Environment
    var time: Int = 6000
    var weather: Weather = Weather.CLEAR
    var weatherIntensity: Float = 0f
    var moonPhase: Int = 0
    var fov: Int = 70
    var renderDistance: Int = 16

    // Render (user-controllable)
    var particles: ParticleMode? = null
    var viewBobbing: Boolean? = null
    var hideHud: Boolean? = null
    var screenEffects: Double? = null
    var fovEffects: Double? = null
    var showHand: Boolean = false
    var entityTicksFrozen: Boolean = false

    private var snapshot: SettingsSnapshot? = null

    /**
     * Full snapshot of game state captured when setup begins.
     * Captures both environment and render settings for clean restoration.
     */
    private data class SettingsSnapshot(
        // Environment
        val dayTime: Long,
        val isRaining: Boolean,
        val isThundering: Boolean,
        val rainLevel: Float,
        val thunderLevel: Float,
        val renderDistance: Int,
        val simulationDistance: Int,
        val fov: Int,
        val brightness: Double,
        val doDaylightCycle: Boolean,
        val doWeatherCycle: Boolean,
        val cloudStatus: CloudStatus,
        // Render
        val graphicsMode: GraphicsStatus,
        val particles: ParticleStatus,
        val ambientOcclusion: Boolean,
        val entityShadows: Boolean,
        val biomeBlend: Int,
        val mipmapLevels: Int,
        val viewBobbing: Boolean,
        val hideGui: Boolean,
        val screenEffects: Double,
        val fovEffects: Double,
    ) {
        companion object {
            fun capture(): SettingsSnapshot? {
                val mc = Minecraft.getInstance()
                val server = mc.singleplayerServer ?: return null
                val overworld = server.overworld()
                val options = mc.options

                return SettingsSnapshot(
                    dayTime = overworld.dayTime,
                    isRaining = overworld.levelData.isRaining,
                    isThundering = overworld.isThundering,
                    rainLevel = overworld.getRainLevel(1f),
                    thunderLevel = overworld.getThunderLevel(1f),
                    renderDistance = options.renderDistance().get(),
                    simulationDistance = options.simulationDistance().get(),
                    fov = options.fov().get(),
                    brightness = options.gamma().get(),
                    doDaylightCycle =
                        overworld.gameRules
                            .getRule(GameRules.RULE_DAYLIGHT)
                            .get(),
                    doWeatherCycle =
                        overworld.gameRules
                            .getRule(GameRules.RULE_WEATHER_CYCLE)
                            .get(),
                    cloudStatus = options.cloudStatus().get(),
                    graphicsMode = options.graphicsMode().get(),
                    particles = options.particles().get(),
                    ambientOcclusion = options.ambientOcclusion().get(),
                    entityShadows = options.entityShadows().get(),
                    biomeBlend = options.biomeBlendRadius().get(),
                    mipmapLevels = options.mipmapLevels().get(),
                    viewBobbing = options.bobView().get(),
                    hideGui = options.hideGui,
                    screenEffects = options.screenEffectScale().get(),
                    fovEffects = options.fovEffectScale().get(),
                )
            }
        }

        fun restore() {
            val mc = Minecraft.getInstance()
            val server = mc.singleplayerServer ?: return
            val options = mc.options

            server.execute {
                val overworld = server.overworld()
                overworld.dayTime = dayTime

                if (isThundering) {
                    overworld.setWeatherParameters(0, WEATHER_DURATION_TICKS, true, true)
                } else if (isRaining) {
                    overworld.setWeatherParameters(0, WEATHER_DURATION_TICKS, true, false)
                } else {
                    overworld.setWeatherParameters(WEATHER_DURATION_TICKS, 0, false, false)
                }

                overworld.setRainLevel(rainLevel)
                overworld.setThunderLevel(thunderLevel)

                overworld.gameRules.getRule(GameRules.RULE_DAYLIGHT).set(doDaylightCycle, server)
                overworld.gameRules.getRule(GameRules.RULE_WEATHER_CYCLE).set(doWeatherCycle, server)
            }

            mc.level?.let { clientLevel ->
                clientLevel.setRainLevel(rainLevel)
                clientLevel.setThunderLevel(thunderLevel)
            }

            options.fov().set(fov)
            options.gamma().set(brightness)
            options.cloudStatus().set(cloudStatus)
            options.graphicsMode().set(graphicsMode)
            options.particles().set(particles)
            options.ambientOcclusion().set(ambientOcclusion)
            options.entityShadows().set(entityShadows)
            options.biomeBlendRadius().set(biomeBlend)
            options.mipmapLevels().set(mipmapLevels)
            options.bobView().set(viewBobbing)
            options.hideGui = hideGui
            options.screenEffectScale().set(screenEffects)
            options.fovEffectScale().set(fovEffects)

            val currentRd = options.renderDistance().get()
            if (currentRd != renderDistance) {
                options.renderDistance().set(renderDistance)
                options.simulationDistance().set(simulationDistance)
            }

            // Trigger chunk rebuild to pick up graphics/AO/biome blend changes
            mc.levelRenderer.allChanged()
        }
    }

    /**
     * Captures the current world state and begins applying overrides.
     * Safe to call multiple times — only captures on the first call.
     */
    fun activate() {
        if (active) return

        val mc = Minecraft.getInstance()
        snapshot = SettingsSnapshot.capture()

        // Read current world values as initial override state
        val options = mc.options
        fov = options.fov().get()
        renderDistance = options.renderDistance().get()

        // Read user-controllable render settings
        particles = particleModeFromStatus(options.particles().get())
        viewBobbing = options.bobView().get()
        hideHud = options.hideGui
        screenEffects = options.screenEffectScale().get()
        fovEffects = options.fovEffectScale().get()

        mc.level?.let { level ->
            time = (level.dayTime % 24000).toInt()
            weather =
                when {
                    level.isThundering -> Weather.THUNDER
                    level.levelData.isRaining -> Weather.RAIN
                    else -> Weather.CLEAR
                }
            moonPhase = level.getMoonPhase()
        }

        // Freeze natural time/weather progression
        val server = mc.singleplayerServer
        server?.execute {
            val overworld = server.overworld()
            overworld.gameRules.getRule(GameRules.RULE_DAYLIGHT).set(false, server)
            overworld.gameRules.getRule(GameRules.RULE_WEATHER_CYCLE).set(false, server)
        }

        // Force shader-friendly defaults for settings not exposed in UI
        applyHardcodedDefaults()

        active = true
        log.info("Scene setup activated")
    }

    fun applyTimeAndMoon() {
        if (!active) return
        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer ?: return
        server.execute {
            val overworld = server.overworld()
            overworld.dayTime = (moonPhase.toLong() * 24000L) + time.toLong()
        }
    }

    fun applyWeather() {
        if (!active) return
        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer ?: return
        server.execute {
            val overworld = server.overworld()
            weather.applyParameters(overworld)
            weather.snapLevels(overworld, weatherIntensity)
        }
        val (rain, thunder) = weather.levels(weatherIntensity)
        mc.level?.let { clientLevel ->
            clientLevel.setRainLevel(rain)
            clientLevel.setThunderLevel(thunder)
        }
    }

    fun applyFov() {
        if (!active) return
        Minecraft
            .getInstance()
            .options
            .fov()
            .set(fov)
    }

    fun applyRenderDistance() {
        if (!active) return
        val mc = Minecraft.getInstance()
        val options = mc.options
        if (options.renderDistance().get() != renderDistance) {
            options.renderDistance().set(renderDistance)
            options.simulationDistance().set(renderDistance)
            mc.levelRenderer.allChanged()
        }
    }

    fun applyParticles() {
        if (!active) return
        val mode = particles ?: return
        val desired =
            when (mode) {
                ParticleMode.ALL -> ParticleStatus.ALL
                ParticleMode.DECREASED -> ParticleStatus.DECREASED
                ParticleMode.MINIMAL -> ParticleStatus.MINIMAL
                ParticleMode.OFF -> ParticleStatus.MINIMAL
            }
        Minecraft
            .getInstance()
            .options
            .particles()
            .set(desired)
    }

    fun applyViewBobbing() {
        if (!active) return
        val value = viewBobbing ?: return
        Minecraft
            .getInstance()
            .options
            .bobView()
            .set(value)
    }

    fun applyHideHud() {
        if (!active) return
        val value = hideHud ?: return
        Minecraft.getInstance().options.hideGui = value
    }

    fun applyScreenEffects() {
        if (!active) return
        val value = screenEffects ?: return
        Minecraft
            .getInstance()
            .options
            .screenEffectScale()
            .set(value)
    }

    fun applyFovEffects() {
        if (!active) return
        val value = fovEffects ?: return
        Minecraft
            .getInstance()
            .options
            .fovEffectScale()
            .set(value)
    }

    /**
     * Forces shader-friendly defaults for settings not exposed in the UI.
     * Shaders override most of these anyway, but consistent values ensure
     * predictable behavior with vanilla rendering too.
     */
    private fun applyHardcodedDefaults() {
        val mc = Minecraft.getInstance()
        val options = mc.options

        options.graphicsMode().set(GraphicsStatus.FANCY)
        options.ambientOcclusion().set(true)
        options.entityShadows().set(true)
        options.biomeBlendRadius().set(5)
        options.mipmapLevels().set(4)
        options.cloudStatus().set(CloudStatus.FANCY)
        options.gamma().set(0.5)

        mc.levelRenderer.allChanged()
    }

    /**
     * Restores the original world state and clears all overrides.
     */
    fun reset() {
        if (!active) return

        if (entityTicksFrozen) {
            SceneInjection.unfreezeEntityTick()
            entityTicksFrozen = false
        }

        snapshot?.restore()
        snapshot = null
        active = false
        log.info("Scene setup reset")
    }
}

private fun particleModeFromStatus(status: ParticleStatus): ParticleMode =
    when (status) {
        ParticleStatus.ALL -> ParticleMode.ALL
        ParticleStatus.DECREASED -> ParticleMode.DECREASED
        ParticleStatus.MINIMAL -> ParticleMode.MINIMAL
    }
