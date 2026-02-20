package com.xevion.glint.ui.base

import io.wispforest.owo.ui.base.BaseOwoScreen
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.OwoUIAdapter
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.Surface
import io.wispforest.owo.ui.core.VerticalAlignment
import net.minecraft.network.chat.Component

/**
 * Base screen class for dialog-style screens (confirmation dialogs, etc.)
 * Provides a centered dialog panel with standard styling.
 */
abstract class GlintDialogScreen(
    title: Component,
) : BaseOwoScreen<FlowLayout>(title) {
    override fun createAdapter(): OwoUIAdapter<FlowLayout> = OwoUIAdapter.create(this, Containers::verticalFlow)

    override fun build(rootComponent: FlowLayout) {
        rootComponent
            .horizontalAlignment(HorizontalAlignment.CENTER)
            .verticalAlignment(VerticalAlignment.CENTER)
            .surface(Surface.VANILLA_TRANSLUCENT)

        // Create centered dialog container
        val dialog = Containers.verticalFlow(Sizing.content(), Sizing.content())
        dialog.horizontalAlignment(HorizontalAlignment.CENTER)
        dialog.padding(GlintTheme.paddingLg())
        dialog.gap(GlintTheme.GAP_MD)
        dialog.surface(Surface.DARK_PANEL)

        buildDialog(dialog)

        rootComponent.child(dialog)
    }

    /**
     * Build the dialog content. Override this to add components.
     * The dialog is already configured with centered alignment and panel background.
     */
    protected abstract fun buildDialog(dialog: FlowLayout)
}
