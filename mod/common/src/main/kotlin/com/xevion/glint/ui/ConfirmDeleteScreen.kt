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
) : Screen(Component.literal("Disable Scene?")) {
    override fun init() {
        val yPos = height / 2 - 20

        addRenderableWidget(
            Button
                .builder(Component.literal("Disable Scene")) {
                    parent.executeDeleteScene(worldName, sceneId)
                    minecraft?.setScreen(parent)
                }.bounds(width / 2 - 105, yPos, 100, 20)
                .build(),
        )

        addRenderableWidget(
            Button
                .builder(CommonComponents.GUI_CANCEL) {
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

        guiGraphics.drawCenteredString(
            font,
            "Are you sure you want to disable '$sceneName'?",
            width / 2,
            height / 2 - 30,
            0xFFFFFF,
        )
        guiGraphics.drawCenteredString(
            font,
            "This will remove it locally and mark it inactive on the API.",
            width / 2,
            height / 2 - 18,
            0x888888,
        )

        super.render(guiGraphics, mouseX, mouseY, delta)
    }
}
