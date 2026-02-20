package com.xevion.glint.ui.tabs

import com.xevion.glint.ui.GlintMainScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.VerticalAlignment
import net.minecraft.network.chat.Component as McComponent

class ShadersTab(
    private val host: GlintMainScreen,
) : MainScreenTab {
    override fun buildMaster(master: FlowLayout) {
        master.horizontalAlignment(HorizontalAlignment.CENTER)
        master.verticalAlignment(VerticalAlignment.CENTER)
        master.child(
            Components
                .label(McComponent.literal("Shader management coming soon"))
                .maxWidth(host.masterTextWidth)
                .horizontalTextAlignment(HorizontalAlignment.CENTER)
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)),
        )
    }

    override fun buildDetail(detail: FlowLayout) {
        detail.horizontalAlignment(HorizontalAlignment.CENTER)
        detail.verticalAlignment(VerticalAlignment.CENTER)
        detail.child(
            Components
                .label(McComponent.literal("No shader selected"))
                .maxWidth(host.detailTextWidth)
                .horizontalTextAlignment(HorizontalAlignment.CENTER)
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)),
        )
        detail.child(
            Components
                .label(McComponent.literal("Select from the list to view details."))
                .maxWidth(host.detailTextWidth)
                .horizontalTextAlignment(HorizontalAlignment.CENTER)
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)),
        )
    }
}
