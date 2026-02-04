package com.xevion.glint.ui

import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintDialogScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Component
import net.minecraft.network.chat.Component as McComponent

class ConfirmDeleteScreen(
    private val parent: SceneManagerScreen,
    private val worldName: String,
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
                GlintComponents.button(McComponent.literal("Disable Scene")) {
                    parent.executeDeleteScene(worldName, sceneId)
                    minecraft?.setScreen(parent)
                },
                GlintComponents.cancelButton { minecraft?.setScreen(parent) },
            ) as Component,
        )
    }
}
