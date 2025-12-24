package com.xevion.glint.ui

import net.minecraft.client.gui.GuiGraphics
import net.minecraft.client.gui.components.Button
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import net.minecraft.network.chat.Component

class ConfirmDeleteScreen(
    private val parent: SceneManagerScreen,
    private val worldName: String,
    private val sceneId: String,
    private val sceneName: String,
) : Screen(Component.literal("Delete Scene?")) {
    override fun init() {
        val message = "Delete '$sceneName'?"
        val yPos = height / 2 - 20

        addRenderableWidget(
            Button
                .builder(CommonComponents.GUI_YES) {
                    parent.executeDeleteScene(worldName, sceneId)
                    minecraft?.setScreen(parent)
                }.bounds(width / 2 - 105, yPos, 100, 20)
                .build(),
        )

        addRenderableWidget(
            Button
                .builder(CommonComponents.GUI_NO) {
                    minecraft?.setScreen(parent)
                }.bounds(width / 2 + 5, yPos, 100, 20)
                .build(),
        )
    }

    override fun render(
        guiGraphics: GuiGraphics,
        mouseX: Int,
        mouseY: Int,
        delta: Float,
    ) {
        renderBackground(guiGraphics, mouseX, mouseY, delta)
        guiGraphics.drawCenteredString(font, title, width / 2, height / 2 - 50, 0xFFFFFF)

        val message = "Delete '$sceneName'?"
        guiGraphics.drawCenteredString(font, message, width / 2, height / 2 - 30, 0xAAAAAA)

        super.render(guiGraphics, mouseX, mouseY, delta)
    }
}
