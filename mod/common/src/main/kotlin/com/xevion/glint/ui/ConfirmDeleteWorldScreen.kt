package com.xevion.glint.ui

import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintDialogScreen
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Component
import net.minecraft.network.chat.Component as McComponent

/**
 * Confirmation dialog for deleting a world's local scene collection.
 */
class ConfirmDeleteWorldScreen(
    private val parent: GlintMainScreen,
    private val fileName: String,
    private val worldName: String,
) : GlintDialogScreen(McComponent.literal("Delete World")) {
    override fun buildDialog(dialog: FlowLayout) {
        dialog.child(
            GlintComponents.confirmationContent(
                title = McComponent.literal("Delete World Collection?"),
                description = McComponent.literal("This will delete all local scenes for \"$worldName\". This cannot be undone."),
                confirmText = McComponent.literal("Delete"),
                onConfirm = {
                    parent.executeDeleteWorld(fileName)
                    minecraft?.setScreen(parent)
                },
                onCancel = { minecraft?.setScreen(parent) },
            ) as Component,
        )
    }
}
