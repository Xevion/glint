package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.scene.SceneManager
import net.minecraft.client.gui.GuiGraphics
import net.minecraft.client.gui.components.Button
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import net.minecraft.network.chat.Component

/**
 * Confirmation dialog for disabling a scene.
 * Disabling removes the scene locally and marks it inactive on the API.
 */
class ConfirmDisableScreen(
    private val parent: Screen,
    private val worldFileName: String,
    private val sceneId: String,
    private val sceneName: String,
) : Screen(Component.literal("Disable Scene?")) {
    override fun init() {
        val centerX = width / 2
        val buttonY = height / 2 + 40

        addRenderableWidget(
            Button
                .builder(Component.literal("Disable Scene")) { executeDisable() }
                .bounds(centerX - 155, buttonY, 150, 20)
                .build(),
        )

        addRenderableWidget(
            Button
                .builder(CommonComponents.GUI_CANCEL) { onClose() }
                .bounds(centerX + 5, buttonY, 150, 20)
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
        super.render(guiGraphics, mouseX, mouseY, delta)

        guiGraphics.drawCenteredString(font, title, width / 2, height / 2 - 20, 0xFFFFFF)
        guiGraphics.drawCenteredString(
            font,
            "Are you sure you want to disable '$sceneName'?",
            width / 2,
            height / 2,
            0xFFFFFF,
        )
        guiGraphics.drawCenteredString(
            font,
            "This will remove it locally and mark it inactive on the API.",
            width / 2,
            height / 2 + 12,
            0x888888,
        )
    }

    private fun executeDisable() {
        SceneManager
            .disableScene(worldFileName, sceneId)
            .thenAccept { success ->
                minecraft?.execute {
                    if (success) {
                        Glint.LOGGER.info("Disabled scene: $sceneId")
                    } else {
                        Glint.LOGGER.error("Failed to disable scene: $sceneId")
                    }
                    minecraft?.setScreen(parent)

                    // Refresh the parent screen if it's GlintHubScreen
                    if (parent is GlintHubScreen) {
                        parent.refreshWorlds()
                    }
                }
            }
    }

    override fun onClose() {
        minecraft?.setScreen(parent)
    }
}
