package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.scene.Scene
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.screenshot.Camera
import com.xevion.glint.screenshot.Position
import net.minecraft.client.Minecraft
import net.minecraft.client.gui.GuiGraphics
import net.minecraft.client.gui.components.Button
import net.minecraft.client.gui.components.EditBox
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import net.minecraft.network.chat.Component

class SaveSceneDialog(
    private val parent: SceneManagerScreen,
) : Screen(Component.literal("Save Current Scene")) {
    private lateinit var sceneIdInput: EditBox
    private lateinit var sceneNameInput: EditBox
    private lateinit var saveButton: Button
    private var errorMessage: String? = null

    override fun init() {
        val centerX = width / 2
        val startY = height / 2 - 60

        sceneIdInput =
            EditBox(font, centerX - 100, startY, 200, 20, Component.literal("Scene ID"))
        sceneIdInput.setHint(Component.literal("scene_id (e.g., village_sunset)"))
        sceneIdInput.setMaxLength(64)
        sceneIdInput.setResponder { validateInput() }
        addRenderableWidget(sceneIdInput)

        sceneNameInput =
            EditBox(font, centerX - 100, startY + 40, 200, 20, Component.literal("Scene Name"))
        sceneNameInput.setHint(Component.literal("Display name (optional)"))
        sceneNameInput.setMaxLength(128)
        addRenderableWidget(sceneNameInput)

        saveButton =
            Button
                .builder(Component.literal("Save")) { saveScene() }
                .bounds(centerX - 105, startY + 80, 100, 20)
                .build()
        saveButton.active = false
        addRenderableWidget(saveButton)

        addRenderableWidget(
            Button
                .builder(CommonComponents.GUI_CANCEL) { minecraft?.setScreen(parent) }
                .bounds(centerX + 5, startY + 80, 100, 20)
                .build(),
        )

        setFocused(sceneIdInput)
    }

    private fun validateInput() {
        val id = sceneIdInput.value.trim()
        errorMessage = null

        if (id.isEmpty()) {
            saveButton.active = false
            return
        }

        if (!id.matches(Regex("[a-z0-9_]+"))) {
            errorMessage = "ID must contain only lowercase letters, numbers, and underscores"
            saveButton.active = false
            return
        }

        if (SceneManager.sceneIdExists(id)) {
            errorMessage = "Scene ID '$id' already exists"
            saveButton.active = false
            return
        }

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

    override fun render(
        guiGraphics: GuiGraphics,
        mouseX: Int,
        mouseY: Int,
        delta: Float,
    ) {
        renderBackground(guiGraphics, mouseX, mouseY, delta)
        guiGraphics.drawCenteredString(font, title, width / 2, height / 2 - 90, 0xFFFFFF)

        val centerX = width / 2
        val startY = height / 2 - 60

        guiGraphics.drawString(font, "Scene ID:", centerX - 100, startY - 12, 0xAAAAAA)
        guiGraphics.drawString(font, "Scene Name:", centerX - 100, startY + 28, 0xAAAAAA)

        // Draw error message if present
        errorMessage?.let { error ->
            guiGraphics.drawCenteredString(font, error, centerX, startY + 105, 0xFF5555)
        }

        super.render(guiGraphics, mouseX, mouseY, delta)
    }
}
