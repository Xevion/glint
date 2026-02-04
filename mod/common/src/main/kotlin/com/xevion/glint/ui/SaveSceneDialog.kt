package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.scene.Scene
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.screenshot.Camera
import com.xevion.glint.screenshot.Position
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
import net.minecraft.network.chat.Component as McComponent

class SaveSceneDialog(
    private val parent: SceneManagerScreen,
) : GlintDialogScreen(McComponent.literal("Save Current Scene")) {
    private lateinit var sceneIdInput: TextBoxComponent
    private lateinit var sceneNameInput: TextBoxComponent
    private lateinit var saveButton: ButtonComponent
    private lateinit var errorLabel: LabelComponent

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
        sceneIdInput.setSuggestion("scene_id (e.g., village_sunset)")
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
        sceneNameInput.setSuggestion("Display name (optional)")
        nameContainer.child(sceneNameInput as Component)
        dialog.child(nameContainer as Component)

        // Error label
        errorLabel = Components.label(McComponent.literal(""))
        errorLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
        errorLabel.margins(Insets.top(GlintTheme.GAP_SM))
        dialog.child(errorLabel as Component)

        // Buttons
        saveButton = GlintComponents.button(McComponent.literal("Save")) { saveScene() }
        saveButton.active = false
        dialog.child(
            GlintComponents.buttonRow(
                saveButton,
                GlintComponents.cancelButton { minecraft?.setScreen(parent) },
            ) as Component,
        )

        // Focus the first input
        val input = sceneIdInput
        uiAdapter.rootComponent.focusHandler()?.focus(input as Component, null)
    }

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

        if (SceneManager.sceneIdExists(id)) {
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
            Glint.LOGGER.error("Cannot save scene - player or level is null")
            return
        }

        if (mc.singleplayerServer == null) {
            Glint.LOGGER.error("Cannot save scene - not in singleplayer")
            return
        }

        val sceneId = sceneIdInput.value.trim()
        val sceneName =
            sceneNameInput.value.trim().ifEmpty {
                sceneId.replace('_', ' ').replaceFirstChar { it.uppercase() }
            }

        val worldName = mc.singleplayerServer!!.worldData.levelName
        val position = Position(x = player.x, y = player.y, z = player.z)
        val camera = Camera(yaw = player.yRot, pitch = player.xRot)
        val dimension = level.dimension().location().toString()
        val timeOfDay = (level.dayTime % 24000).toInt()

        val scene =
            Scene(
                id = sceneId,
                name = sceneName,
                description = "Scene captured from in-game menu",
                dimension = dimension,
                position = position,
                camera = camera,
                timeOfDay = timeOfDay,
                weather = if (level.levelData.isRaining) com.xevion.glint.scene.Weather.RAIN else com.xevion.glint.scene.Weather.CLEAR,
                weatherIntensity = 0.0f,
            )

        if (SceneManager.addScene(worldName, scene)) {
            Glint.LOGGER.info("Saved scene: $sceneId to world: $worldName")
            parent.refreshSceneList()
            minecraft?.setScreen(parent)
        } else {
            Glint.LOGGER.error("Failed to save scene: $sceneId")
        }
    }
}
