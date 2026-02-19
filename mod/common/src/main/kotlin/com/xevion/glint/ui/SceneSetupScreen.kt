package com.xevion.glint.ui

import com.xevion.glint.api.ApiConfig
import com.xevion.glint.scene.ParticleMode
import com.xevion.glint.scene.SceneFormatting
import com.xevion.glint.scene.SceneInjection
import com.xevion.glint.scene.SceneSetupState
import com.xevion.glint.scene.Weather
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintPanelScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.component.LabelComponent
import io.wispforest.owo.ui.component.SlimSliderComponent
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.Insets
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.VerticalAlignment
import org.lwjgl.glfw.GLFW
import net.minecraft.network.chat.Component as McComponent

/**
 * Lightweight scene composition screen for adjusting environment settings.
 *
 * Opened via a dedicated keybind (K). Settings apply live to the world and
 * persist when the screen is closed. The iterative workflow is:
 * open -> adjust -> close -> walk around -> reopen -> adjust -> export.
 */
class SceneSetupScreen : GlintPanelScreen(McComponent.literal("Scene Setup")) {
    companion object {
        private const val SLIDER_WIDTH = 180

        private const val TIME_MIN = 0.0
        private const val TIME_MAX = 23999.0
        private const val TIME_SCROLL_STEP = 100
        private const val MOON_MIN = 0.0
        private const val MOON_MAX = 7.0
        private const val FOV_MIN = 30.0
        private const val FOV_MAX = 110.0
        private const val RD_MIN = 2.0
        private const val RD_MAX = 32.0
        private const val WEATHER_INTENSITY_MIN = 0.0
        private const val WEATHER_INTENSITY_MAX = 1.0
        private const val WEATHER_INTENSITY_SCROLL_STEP = 0.05
    }

    private enum class Tab { ENVIRONMENT, RENDER }

    private var activeTab = Tab.ENVIRONMENT

    // Slider tracking for hover-based scroll
    private data class TrackedSlider(
        val slider: SlimSliderComponent,
        val label: LabelComponent?,
        val min: Double,
        val max: Double,
        /** Scroll wheel step (coarse). Arrow key step is always 1 or stepSize. */
        val scrollStep: Double,
        val onChanged: (Double) -> Unit,
        /** Called after scroll completes, mirroring onSlideEnd for drag interactions. */
        val onRelease: (() -> Unit)?,
    )

    private val trackedSliders = mutableListOf<TrackedSlider>()

    // Reference for conditional visibility
    private var weatherIntensityRow: FlowLayout? = null

    override fun buildPanel(panel: FlowLayout) {
        SceneSetupState.activate()
        trackedSliders.clear()

        // Tab bar
        val tabBar = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        tabBar.gap(GlintTheme.GAP_SM)
        tabBar.child(
            GlintComponents.tabButton(
                McComponent.literal("Environment"),
                activeTab == Tab.ENVIRONMENT,
            ) {
                activeTab = Tab.ENVIRONMENT
                rebuildPanel(panel)
            } as Component,
        )
        tabBar.child(
            GlintComponents.tabButton(
                McComponent.literal("Render"),
                activeTab == Tab.RENDER,
            ) {
                activeTab = Tab.RENDER
                rebuildPanel(panel)
            } as Component,
        )
        panel.child(tabBar as Component)

        when (activeTab) {
            Tab.ENVIRONMENT -> buildEnvironmentTab(panel)
            Tab.RENDER -> buildRenderTab(panel)
        }

        // Bottom bar: Reset + Export
        val bottomBar = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        bottomBar.gap(GlintTheme.GAP_MD)
        bottomBar.margins(Insets.top(GlintTheme.GAP_MD))

        bottomBar.child(
            GlintComponents.button(McComponent.literal("Reset")) {
                SceneSetupState.reset()
                onClose()
            } as Component,
        )
        bottomBar.child(
            GlintComponents.button(McComponent.literal("Export Scene")) {
                openExportDialog()
            } as Component,
        )
        panel.child(bottomBar as Component)
    }

    private fun buildEnvironmentTab(panel: FlowLayout) {
        val state = SceneSetupState

        // Time of day
        val timeLabel = Components.label(timeText())
        addTrackedSlider(
            panel,
            timeLabel,
            TIME_MIN,
            TIME_MAX,
            state.time.toDouble(),
            scrollStep = TIME_SCROLL_STEP.toDouble(),
        ) { value ->
            state.time = value.toInt()
            timeLabel.text(timeText())
            state.applyTimeAndMoon()
        }

        // Time presets
        val presetRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        presetRow.gap(GlintTheme.GAP_SM)
        presetRow.margins(Insets.bottom(GlintTheme.GAP_SM))
        for ((name, ticks) in TIME_PRESETS) {
            presetRow.child(
                GlintComponents.smallButton(McComponent.literal(name), width = 45) {
                    state.time = ticks
                    timeLabel.text(timeText())
                    updateSliderValue(state.time.toDouble(), TIME_MIN, TIME_MAX)
                    state.applyTimeAndMoon()
                } as Component,
            )
        }
        panel.child(presetRow as Component)

        // Weather cycle button
        val weatherRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        weatherRow.gap(GlintTheme.GAP_SM)
        weatherRow.verticalAlignment(VerticalAlignment.CENTER)
        weatherRow.child(
            Components
                .label(McComponent.literal("Weather:"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        val weatherButton =
            GlintComponents.smallButton(
                McComponent.literal(state.weather.displayName),
                width = 60,
                tooltip = McComponent.literal("Click to cycle weather"),
            ) { btn ->
                state.weather = state.weather.next()
                btn.setMessage(McComponent.literal(state.weather.displayName))
                state.applyWeather()
                syncWeatherIntensityVisibility(panel)
            }
        weatherRow.child(weatherButton as Component)
        panel.child(weatherRow as Component)

        // Weather intensity
        weatherIntensityRow = Containers.verticalFlow(Sizing.content(), Sizing.content())
        val intensityLabel = Components.label(weatherIntensityText())
        addTrackedSlider(
            weatherIntensityRow!!,
            intensityLabel,
            WEATHER_INTENSITY_MIN,
            WEATHER_INTENSITY_MAX,
            state.weatherIntensity.toDouble(),
            scrollStep = WEATHER_INTENSITY_SCROLL_STEP,
            stepSize = 0.01,
        ) { value ->
            state.weatherIntensity = value.toFloat()
            intensityLabel.text(weatherIntensityText())
            state.applyWeather()
        }
        if (state.weather != Weather.CLEAR) {
            panel.child(weatherIntensityRow as Component)
        }

        // Moon phase
        val moonLabel = Components.label(moonText())
        addTrackedSlider(panel, moonLabel, MOON_MIN, MOON_MAX, state.moonPhase.toDouble()) { value ->
            state.moonPhase = value.toInt()
            moonLabel.text(moonText())
            state.applyTimeAndMoon()
        }

        // FOV
        val fovLabel = Components.label(fovText())
        addTrackedSlider(panel, fovLabel, FOV_MIN, FOV_MAX, state.fov.toDouble()) { value ->
            state.fov = value.toInt()
            fovLabel.text(fovText())
            state.applyFov()
        }

        // Render distance
        val rdLabel = Components.label(rdText())
        addTrackedSlider(
            panel,
            rdLabel,
            RD_MIN,
            RD_MAX,
            state.renderDistance.toDouble(),
            onRelease = { state.applyRenderDistance() },
        ) { value ->
            state.renderDistance = value.toInt()
            rdLabel.text(rdText())
        }

        // Freeze entities
        val freezeRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        freezeRow.gap(GlintTheme.GAP_SM)
        freezeRow.verticalAlignment(VerticalAlignment.CENTER)
        freezeRow.child(
            Components
                .label(McComponent.literal("Freeze Entities:"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        val freezeButton =
            GlintComponents.smallButton(
                McComponent.literal(if (state.entityTicksFrozen) "On" else "Off"),
                width = 40,
                tooltip = McComponent.literal("Freeze entity movement and animations"),
            ) { btn ->
                state.entityTicksFrozen = !state.entityTicksFrozen
                if (state.entityTicksFrozen) {
                    SceneInjection.freezeEntityTick()
                } else {
                    SceneInjection.unfreezeEntityTick()
                }
                btn.setMessage(McComponent.literal(if (state.entityTicksFrozen) "On" else "Off"))
            }
        freezeRow.child(freezeButton as Component)
        panel.child(freezeRow as Component)
    }

    private fun buildRenderTab(panel: FlowLayout) {
        val state = SceneSetupState

        // Particles
        addCycleButton(panel, "Particles:", state.particles?.displayName() ?: "Default") {
            state.particles = nextParticleMode(state.particles)
            state.applyParticles()
            state.particles?.displayName() ?: "Default"
        }

        // View bobbing
        addToggleButton(panel, "View Bobbing:", state.viewBobbing) { newVal ->
            state.viewBobbing = newVal
            state.applyViewBobbing()
        }

        // Hide HUD
        addToggleButton(panel, "Hide HUD:", state.hideHud) { newVal ->
            state.hideHud = newVal
            state.applyHideHud()
        }

        // Screen effects
        addToggleButton(panel, "Screen Effects:", state.screenEffects?.let { it > 0.0 }) { newVal ->
            state.screenEffects = if (newVal == true) 1.0 else 0.0
            state.applyScreenEffects()
        }

        // FOV effects
        addToggleButton(panel, "FOV Effects:", state.fovEffects?.let { it > 0.0 }) { newVal ->
            state.fovEffects = if (newVal == true) 1.0 else 0.0
            state.applyFovEffects()
        }

        // Show hand (deferred — basic toggle for now, needs spectator hand render mixin)
        addToggleButton(panel, "Show Hand:", state.showHand) { newVal ->
            state.showHand = newVal ?: false
        }
    }

    private fun addTrackedSlider(
        container: FlowLayout,
        label: LabelComponent,
        min: Double,
        max: Double,
        initial: Double,
        scrollStep: Double = 1.0,
        stepSize: Double = 1.0,
        onRelease: (() -> Unit)? = null,
        onChanged: (Double) -> Unit,
    ) {
        label.color(Color.ofRgb(GlintTheme.TEXT_SECONDARY))
        container.child(label as Component)

        val slider =
            Components
                .slimSlider(SlimSliderComponent.Axis.HORIZONTAL)
                .min(min)
                .max(max)
                .stepSize(stepSize)
                .value(initial)
        (slider as Component).horizontalSizing(Sizing.fixed(SLIDER_WIDTH))
        slider.onChanged().subscribe { value -> onChanged(value) }
        onRelease?.let { callback -> slider.onSlideEnd().subscribe { callback() } }
        container.child(slider as Component)

        trackedSliders.add(
            TrackedSlider(
                slider = slider,
                label = label,
                min = min,
                max = max,
                scrollStep = scrollStep,
                onChanged = onChanged,
                onRelease = onRelease,
            ),
        )
    }

    private fun addCycleButton(
        panel: FlowLayout,
        labelText: String,
        initialValue: String,
        onCycle: () -> String,
    ) {
        val row = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        row.gap(GlintTheme.GAP_SM)
        row.verticalAlignment(VerticalAlignment.CENTER)
        row.child(
            Components
                .label(McComponent.literal(labelText))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        val button =
            GlintComponents.smallButton(McComponent.literal(initialValue), width = 70) { btn ->
                val newLabel = onCycle()
                btn.setMessage(McComponent.literal(newLabel))
            }
        row.child(button as Component)
        panel.child(row as Component)
    }

    /**
     * Adds a three-state toggle: null (Default) -> true (On) -> false (Off) -> null.
     * For non-nullable booleans, cycles: false -> true -> false.
     */
    private fun addToggleButton(
        panel: FlowLayout,
        labelText: String,
        initialValue: Boolean?,
        onChange: (Boolean?) -> Unit,
    ) {
        val row = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        row.gap(GlintTheme.GAP_SM)
        row.verticalAlignment(VerticalAlignment.CENTER)
        row.child(
            Components
                .label(McComponent.literal(labelText))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        var current = initialValue
        val button =
            GlintComponents.smallButton(McComponent.literal(toggleLabel(current)), width = 50) { btn ->
                current = nextToggle(current)
                onChange(current)
                btn.setMessage(McComponent.literal(toggleLabel(current)))
            }
        row.child(button as Component)
        panel.child(row as Component)
    }

    override fun keyPressed(
        keyCode: Int,
        scanCode: Int,
        modifiers: Int,
    ): Boolean {
        // K toggles this screen closed (no text inputs on this screen to worry about)
        if (keyCode == GLFW.GLFW_KEY_K) {
            onClose()
            return true
        }
        // J opens the Glint Hub
        if (keyCode == GLFW.GLFW_KEY_J) {
            val config = ApiConfig.load()
            if (config.needsValidation()) {
                val hub = GlintMainScreen(null)
                minecraft?.setScreen(ApiConfigWizardScreen(hub, showConnectionFirst = true))
            } else {
                minecraft?.setScreen(GlintMainScreen(null))
            }
            return true
        }
        return super.keyPressed(keyCode, scanCode, modifiers)
    }

    override fun mouseScrolled(
        mouseX: Double,
        mouseY: Double,
        horizontalAmount: Double,
        verticalAmount: Double,
    ): Boolean {
        // Check if cursor is hovering over any tracked slider
        for (tracked in trackedSliders) {
            val component = tracked.slider as Component
            if (component.isInBoundingBox(mouseX, mouseY)) {
                val current = tracked.slider.value()
                val delta = verticalAmount * tracked.scrollStep
                val newValue = (current + delta).coerceIn(tracked.min, tracked.max)
                if (newValue != current) {
                    tracked.slider.value(newValue)
                    tracked.onChanged(newValue)
                    tracked.onRelease?.invoke()
                }
                return true
            }
        }
        return super.mouseScrolled(mouseX, mouseY, horizontalAmount, verticalAmount)
    }

    override fun onBackgroundScroll(amount: Double): Boolean {
        val state = SceneSetupState
        val newFov = (state.fov - amount.toInt()).coerceIn(FOV_MIN.toInt(), FOV_MAX.toInt())
        if (newFov == state.fov) return false
        state.fov = newFov
        state.applyFov()
        return true
    }

    private fun rebuildPanel(panel: FlowLayout) {
        val children = panel.children().toMutableList()
        if (children.size > 1) {
            for (i in children.size - 1 downTo 1) {
                panel.removeChild(children[i])
            }
        }
        trackedSliders.clear()
        buildPanel(panel)
    }

    private fun openExportDialog() {
        minecraft?.setScreen(
            ExportSceneDialog(
                parentScreen = this,
                onExportComplete = {},
            ),
        )
    }

    override fun onClose() {
        minecraft?.setScreen(null)
    }

    override fun onPanelClosed() {
        // Don't reset state — it persists until explicit reset
    }

    private fun timeText(): McComponent = McComponent.literal("Time: ${SceneFormatting.formatTime(SceneSetupState.time)}")

    private fun moonText(): McComponent = McComponent.literal("Moon: ${SceneFormatting.moonPhaseName(SceneSetupState.moonPhase)}")

    private fun fovText(): McComponent = McComponent.literal("FOV (${SceneSetupState.fov})")

    private fun rdText(): McComponent = McComponent.literal("Render Distance (${SceneSetupState.renderDistance} chunks)")

    private fun weatherIntensityText(): McComponent =
        McComponent.literal("Intensity (${"%.0f".format(SceneSetupState.weatherIntensity * 100)}%)")

    private fun syncWeatherIntensityVisibility(panel: FlowLayout) {
        val row = weatherIntensityRow ?: return
        if (SceneSetupState.weather == Weather.CLEAR) {
            panel.removeChild(row as Component)
        } else if ((row as Component).parent() == null) {
            panel.child(row as Component)
        }
    }

    private fun updateSliderValue(
        value: Double,
        min: Double,
        max: Double,
    ) {
        trackedSliders
            .lastOrNull { it.min == min && it.max == max }
            ?.slider
            ?.value(value)
    }
}

private val TIME_PRESETS =
    listOf(
        "Dawn" to 23000,
        "Noon" to 6000,
        "Sunset" to 12000,
        "Night" to 18000,
    )

private fun nextParticleMode(current: ParticleMode?): ParticleMode? =
    when (current) {
        null -> ParticleMode.ALL
        ParticleMode.ALL -> ParticleMode.DECREASED
        ParticleMode.DECREASED -> ParticleMode.MINIMAL
        ParticleMode.MINIMAL -> ParticleMode.OFF
        ParticleMode.OFF -> null
    }

private fun nextToggle(current: Boolean?): Boolean? =
    when (current) {
        null -> true
        true -> false
        false -> null
    }

private fun toggleLabel(value: Boolean?): String =
    when (value) {
        null -> "Default"
        true -> "On"
        false -> "Off"
    }

private fun ParticleMode.displayName(): String = name.lowercase().replaceFirstChar { it.uppercase() }
