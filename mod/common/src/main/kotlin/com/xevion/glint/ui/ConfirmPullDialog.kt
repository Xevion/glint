package com.xevion.glint.ui

import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintDialogScreen
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Component
import net.minecraft.network.chat.Component as McComponent

/**
 * Confirmation dialog for pulling scenes from the API.
 * Warns that local scenes will be replaced.
 */
class ConfirmPullDialog(
    private val parent: GlintMainScreen,
    private val worldName: String,
    private val localSceneCount: Int,
    private val onConfirm: () -> Unit,
) : GlintDialogScreen(McComponent.literal("Pull Scenes")) {
    override fun buildDialog(dialog: FlowLayout) {
        val message =
            if (localSceneCount > 0) {
                "Replace $localSceneCount local scene(s) with scenes from API for \"$worldName\"?"
            } else {
                "Pull scenes from API for \"$worldName\"?"
            }

        dialog.child(
            GlintComponents.confirmationContent(
                title = McComponent.literal("Pull Scenes"),
                description = McComponent.literal(message),
                confirmText = McComponent.literal("Pull Scenes"),
                onConfirm = {
                    onConfirm()
                    minecraft?.setScreen(parent)
                },
                onCancel = { minecraft?.setScreen(parent) },
            ) as Component,
        )
    }

    override fun onClose() {
        minecraft?.setScreen(parent)
    }
}
