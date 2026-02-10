package com.xevion.glint.ui

import com.xevion.glint.Loggers
import com.xevion.glint.capture.Camera
import com.xevion.glint.capture.Position
import com.xevion.glint.scene.Scene
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.scene.Weather
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintDialogScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.ButtonComponent
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.component.LabelComponent
import io.wispforest.owo.ui.component.TextBoxComponent
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.Insets
import io.wispforest.owo.ui.core.Sizing
import net.minecraft.client.Minecraft
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.Component as McComponent

class SaveSceneDialog(
    private val parentScreen: Screen,
    private val worldName: String,
    private val existingScene: Scene? = null,
    private val onSave: () -> Unit = {},
) : GlintDialogScreen(
        McComponent.literal(if (existingScene != null) "Edit Scene" else "Save Current Scene"),
    ) {
    /**
     * Legacy constructor for SceneManagerScreen compatibility.
     */
    constructor(parent: SceneManagerScreen) : this(
        parentScreen = parent,
        worldName =
            Minecraft
                .getInstance()
                .singleplayerServer
                ?.worldData
                ?.levelName ?: "Unknown",
        existingScene = null,
        onSave = { parent.refreshSceneList() },
    )

    companion object {
        private val TIME_PRESETS =
            listOf(
                "Morning" to 1000,
                "Noon" to 6000,
                "Afternoon" to 9000,
                "Evening" to 13000,
                "Night" to 18000,
            )
    }

    private lateinit var sceneIdInput: TextBoxComponent
    private lateinit var sceneNameInput: TextBoxComponent
    private lateinit var saveButton: ButtonComponent
    private lateinit var errorLabel: LabelComponent

    // Capture state
    private var capturedPosition: Position? = null
    private var capturedCamera: Camera? = null
    private var capturedDimension: String? = null
    private var selectedTimeOfDay: Int = 6000
    private var selectedWeather: Weather = Weather.CLEAR
    private var timePresetIndex: Int = 1

    private lateinit var positionLabel: LabelComponent
    private lateinit var cameraLabel: LabelComponent
    private lateinit var dimensionLabel: LabelComponent
    private lateinit var timeButton: ButtonComponent
    private lateinit var weatherButton: ButtonComponent

    private val isEditMode get() = existingScene != null

    override fun buildDialog(dialog: FlowLayout) {
        dialog.child(GlintComponents.title(title) as Component)

        // Scene ID input
        val idContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
        idContainer.horizontalAlignment(HorizontalAlignment.LEFT)
        idContainer.gap(GlintTheme.GAP_SM)
        idContainer.child(
            Components
                .label(McComponent.literal("Scene ID:"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        sceneIdInput = Components.textBox(Sizing.fixed(200))
        sceneIdInput.setMaxLength(64)
        if (isEditMode) {
            sceneIdInput.text(existingScene!!.id)
            sceneIdInput.active = false
        } else {
            sceneIdInput.setSuggestion("scene_id (e.g., village_sunset)")
        }
        sceneIdInput.onChanged().subscribe { validateInput() }
        idContainer.child(sceneIdInput as Component)
        dialog.child(idContainer as Component)

        // Scene Name input
        val nameContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
        nameContainer.horizontalAlignment(HorizontalAlignment.LEFT)
        nameContainer.gap(GlintTheme.GAP_SM)
        nameContainer.child(
            Components
                .label(McComponent.literal("Scene Name:"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        sceneNameInput = Components.textBox(Sizing.fixed(200))
        sceneNameInput.setMaxLength(128)
        if (isEditMode) {
            sceneNameInput.text(existingScene!!.name)
        } else {
            sceneNameInput.setSuggestion("Display name (optional)")
        }
        nameContainer.child(sceneNameInput as Component)
        dialog.child(nameContainer as Component)

        // Capture details section
        initCaptureState()
        buildCaptureDetails(dialog)

        // Error label
        errorLabel = Components.label(McComponent.literal(""))
        errorLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
        errorLabel.margins(Insets.top(GlintTheme.GAP_SM))
        dialog.child(errorLabel as Component)

        // Buttons
        val saveText = if (isEditMode) "Update" else "Save"
        saveButton = GlintComponents.button(McComponent.literal(saveText)) { saveScene() }
        saveButton.active = isEditMode
        dialog.child(
            GlintComponents.buttonRow(
                saveButton,
                GlintComponents.cancelButton { minecraft?.setScreen(parentScreen) },
            ) as Component,
        )

        // Focus the appropriate input
        val focusTarget = if (isEditMode) sceneNameInput else sceneIdInput
        uiAdapter.rootComponent.focusHandler()?.focus(focusTarget as Component, null)
    }

    private fun initCaptureState() {
        if (isEditMode) {
            val scene = existingScene!!
            capturedPosition = scene.position
            capturedCamera = scene.camera
            capturedDimension = scene.dimension
            selectedTimeOfDay = scene.timeOfDay
            selectedWeather = scene.weather
            timePresetIndex = TIME_PRESETS.indexOfFirst { it.second >= selectedTimeOfDay }.coerceAtLeast(0)
        } else {
            val mc = Minecraft.getInstance()
            val player = mc.player
            val level = mc.level
            if (player != null && level != null) {
                capturedPosition = Position(player.x, player.y, player.z)
                capturedCamera = Camera(player.yRot, player.xRot)
                capturedDimension = level.dimension().location().toString()
                selectedTimeOfDay = (level.dayTime % 24000).toInt()
                selectedWeather = if (level.levelData.isRaining) Weather.RAIN else Weather.CLEAR
                timePresetIndex = TIME_PRESETS.indexOfFirst { it.second >= selectedTimeOfDay }.coerceAtLeast(0)
            }
        }
    }

    private fun buildCaptureDetails(dialog: FlowLayout) {
        val section = Containers.verticalFlow(Sizing.content(), Sizing.content())
        section.horizontalAlignment(HorizontalAlignment.LEFT)
        section.gap(GlintTheme.GAP_SM)
        section.padding(Insets.top(GlintTheme.GAP_SM))

        section.child(
            Components
                .label(McComponent.literal("Capture Details:"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )

        // Position row
        val posRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        posRow.gap(GlintTheme.GAP_SM)
        posRow.child(
            Components
                .label(McComponent.literal("Position:"))
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
        positionLabel = Components.label(McComponent.literal(formatPosition()))
        positionLabel.color(Color.ofRgb(GlintTheme.TEXT_PRIMARY))
        posRow.child(positionLabel as Component)
        if (!isEditMode) {
            posRow.child(
                GlintComponents.smallButton(McComponent.literal("Refresh"), width = 50) {
                    refreshCapture()
                } as Component,
            )
        }
        section.child(posRow as Component)

        // Camera row
        val camRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        camRow.gap(GlintTheme.GAP_SM)
        camRow.child(
            Components
                .label(McComponent.literal("Camera:"))
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
        cameraLabel = Components.label(McComponent.literal(formatCamera()))
        cameraLabel.color(Color.ofRgb(GlintTheme.TEXT_PRIMARY))
        camRow.child(cameraLabel as Component)
        section.child(camRow as Component)

        // Dimension row
        val dimRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        dimRow.gap(GlintTheme.GAP_SM)
        dimRow.child(
            Components
                .label(McComponent.literal("Dimension:"))
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
        dimensionLabel = Components.label(McComponent.literal(capturedDimension ?: "unknown"))
        dimensionLabel.color(Color.ofRgb(GlintTheme.TEXT_PRIMARY))
        dimRow.child(dimensionLabel as Component)
        section.child(dimRow as Component)

        // Time cycle button
        val timeRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        timeRow.gap(GlintTheme.GAP_SM)
        timeRow.child(
            Components
                .label(McComponent.literal("Time:"))
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
        val (timeName, _) = TIME_PRESETS[timePresetIndex]
        timeButton =
            GlintComponents.smallButton(
                McComponent.literal(timeName),
                width = 70,
                tooltip = McComponent.literal("$selectedTimeOfDay ticks"),
            ) { cycleTime() }
        timeRow.child(timeButton as Component)
        section.child(timeRow as Component)

        // Weather cycle button
        val weatherRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        weatherRow.gap(GlintTheme.GAP_SM)
        weatherRow.child(
            Components
                .label(McComponent.literal("Weather:"))
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
        weatherButton =
            GlintComponents.smallButton(
                McComponent.literal(formatWeather(selectedWeather)),
                width = 70,
            ) { cycleWeather() }
        weatherRow.child(weatherButton as Component)
        section.child(weatherRow as Component)

        dialog.child(section as Component)
    }

    private fun refreshCapture() {
        val mc = Minecraft.getInstance()
        val player = mc.player ?: return
        val level = mc.level ?: return
        capturedPosition = Position(player.x, player.y, player.z)
        capturedCamera = Camera(player.yRot, player.xRot)
        capturedDimension = level.dimension().location().toString()
        updateCaptureLabels()
    }

    private fun updateCaptureLabels() {
        positionLabel.text(McComponent.literal(formatPosition()))
        cameraLabel.text(McComponent.literal(formatCamera()))
        capturedDimension?.let {
            dimensionLabel.text(McComponent.literal(it))
        }
    }

    private fun cycleTime() {
        timePresetIndex = (timePresetIndex + 1) % TIME_PRESETS.size
        val (name, ticks) = TIME_PRESETS[timePresetIndex]
        selectedTimeOfDay = ticks
        timeButton.setMessage(McComponent.literal(name))
        (timeButton as Component).tooltip(McComponent.literal("$ticks ticks"))
    }

    private fun cycleWeather() {
        val values = Weather.entries
        val nextIndex = (values.indexOf(selectedWeather) + 1) % values.size
        selectedWeather = values[nextIndex]
        weatherButton.setMessage(McComponent.literal(formatWeather(selectedWeather)))
    }

    private fun formatPosition(): String {
        val pos = capturedPosition ?: return "unknown"
        return "%.1f, %.1f, %.1f".format(pos.x, pos.y, pos.z)
    }

    private fun formatCamera(): String {
        val cam = capturedCamera ?: return "unknown"
        return "Yaw %.1f, Pitch %.1f".format(cam.yaw, cam.pitch)
    }

    private fun formatWeather(weather: Weather): String = weather.name.lowercase().replaceFirstChar { it.uppercase() }

    private fun validateInput() {
        val id = sceneIdInput.value.trim()

        if (id.isEmpty()) {
            errorLabel.text(McComponent.literal(""))
            saveButton.active = false
            return
        }

        if (!id.matches(Regex("[a-z0-9_]+"))) {
            errorLabel.text(McComponent.literal("ID must contain only lowercase letters, numbers, and underscores"))
            saveButton.active = false
            return
        }

        // In edit mode, skip uniqueness check for the existing scene's own ID
        if (!isEditMode && SceneManager.sceneIdExists(id)) {
            errorLabel.text(McComponent.literal("Scene ID '$id' already exists"))
            saveButton.active = false
            return
        }

        errorLabel.text(McComponent.literal(""))
        saveButton.active = true
    }

    private fun saveScene() {
        val mc = Minecraft.getInstance()
        val player = mc.player
        val level = mc.level

        if (player == null || level == null) {
            Loggers.Ui.get().error("Cannot save scene - player or level is null")
            return
        }

        if (mc.singleplayerServer == null) {
            Loggers.Ui.get().error("Cannot save scene - not in singleplayer")
            return
        }

        val sceneId = sceneIdInput.value.trim()
        val sceneName =
            sceneNameInput.value.trim().ifEmpty {
                sceneId.replace('_', ' ').replaceFirstChar { it.uppercase() }
            }

        val position = capturedPosition ?: Position(x = player.x, y = player.y, z = player.z)
        val camera = capturedCamera ?: Camera(yaw = player.yRot, pitch = player.xRot)
        val dimension = capturedDimension ?: level.dimension().location().toString()

        val scene =
            Scene(
                id = sceneId,
                name = sceneName,
                description = if (isEditMode) existingScene!!.description else "Scene captured from in-game menu",
                dimension = dimension,
                position = position,
                camera = camera,
                timeOfDay = selectedTimeOfDay,
                weather = selectedWeather,
                weatherIntensity =
                    if (selectedWeather == Weather.CLEAR) 0.0f else 1.0f,
            )

        if (isEditMode) {
            // Remove old scene first, then add updated one
            val fileName =
                SceneManager
                    .discoverAllCollections()
                    .find { (_, col) -> col.scenes.any { it.id == sceneId } }
                    ?.first
            if (fileName != null) {
                SceneManager.removeScene(fileName, sceneId)
            }
        }

        if (SceneManager.addScene(worldName, scene)) {
            Loggers.Ui.get().info("Saved scene") {
                "scene_id" to sceneId
                "world" to worldName
            }
            onSave()
            minecraft?.setScreen(parentScreen)
        } else {
            Loggers.Ui.get().error("Failed to save scene") {
                "scene_id" to sceneId
            }
        }
    }
}
