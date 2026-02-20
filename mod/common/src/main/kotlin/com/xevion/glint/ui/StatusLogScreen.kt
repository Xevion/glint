package com.xevion.glint.ui

import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintListScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.Component as McComponent

/**
 * Full-page scrollable view of the status log.
 */
class StatusLogScreen(
    private val parent: Screen,
) : GlintListScreen(McComponent.literal("Status Log")) {
    override fun buildHeader(header: FlowLayout) {
        header.child(GlintComponents.title(McComponent.literal("Status Log")))
    }

    override fun buildContent(content: FlowLayout) {
        val entries = StatusLog.entries()

        if (entries.isEmpty()) {
            content.child(
                Components
                    .label(McComponent.literal("No log entries"))
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)),
            )
            return
        }

        for (entry in entries) {
            val color =
                when (entry.level) {
                    StatusLog.Level.INFO -> GlintTheme.TEXT_SUCCESS
                    StatusLog.Level.WARNING -> GlintTheme.TEXT_WARNING
                    StatusLog.Level.ERROR -> GlintTheme.TEXT_ERROR
                }
            content.child(
                Components
                    .label(McComponent.literal("[${entry.formattedTime()}] ${entry.message}"))
                    .color(Color.ofRgb(color)),
            )
        }
    }

    override fun buildFooter(footer: FlowLayout) {
        footer.child(
            GlintComponents.button(McComponent.literal("Clear Log")) {
                StatusLog.clear()
                refreshContent()
            } as Component,
        )
        footer.child(
            GlintComponents.button(McComponent.literal("Close")) { onClose() } as Component,
        )
    }

    override fun onClose() {
        minecraft?.setScreen(parent)
    }
}
