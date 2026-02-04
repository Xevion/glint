package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintDialogScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Component
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.Component as McComponent

/**
 * Confirmation dialog for disabling a scene.
 * Disabling removes the scene locally and marks it inactive on the API.
 */
class ConfirmDisableScreen(
    private val parent: Screen,
    private val worldFileName: String,
    private val sceneId: String,
    private val sceneName: String,
) : GlintDialogScreen(McComponent.literal("Disable Scene?")) {
    override fun buildDialog(dialog: FlowLayout) {
        dialog.child(GlintComponents.title(title) as Component)

        dialog.child(
            GlintComponents.textBlock(
                McComponent.literal("Are you sure you want to disable '$sceneName'?") to GlintTheme.TEXT_PRIMARY,
                McComponent.literal("This will remove it locally and mark it inactive on the API.") to GlintTheme.TEXT_MUTED,
            ) as Component,
        )

        dialog.child(
            GlintComponents.buttonRow(
                GlintComponents.wideButton(McComponent.literal("Disable Scene")) { executeDisable() },
                GlintComponents.cancelButton { onClose() },
            ) as Component,
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

                    // Refresh the parent screen if it supports refresh
                    if (parent is GlintMainScreen) {
                        parent.refreshWorlds()
                    }
                }
            }
    }

    override fun onClose() {
        minecraft?.setScreen(parent)
    }
}
