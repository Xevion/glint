package com.xevion.glint.ui.base

import io.wispforest.owo.ui.base.BaseOwoScreen
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.container.ScrollContainer
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.OwoUIAdapter
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.Surface
import io.wispforest.owo.ui.core.VerticalAlignment
import net.minecraft.network.chat.Component as McComponent

/**
 * Base screen class for list-based UI screens with header/content/footer layout.
 * Provides a consistent structure for screens showing scrollable lists.
 */
abstract class GlintListScreen(
    title: McComponent,
) : BaseOwoScreen<FlowLayout>(title) {
    protected lateinit var headerRow: FlowLayout
    protected lateinit var contentScroll: ScrollContainer<FlowLayout>
    protected lateinit var contentList: FlowLayout
    protected lateinit var footerRow: FlowLayout

    override fun createAdapter(): OwoUIAdapter<FlowLayout> = OwoUIAdapter.create(this, Containers::verticalFlow)

    override fun build(rootComponent: FlowLayout) {
        rootComponent
            .horizontalAlignment(HorizontalAlignment.CENTER)
            .verticalAlignment(VerticalAlignment.TOP)
            .surface(Surface.VANILLA_TRANSLUCENT)
            .padding(GlintTheme.paddingSm())

        // Header row (fixed, horizontal)
        headerRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        headerRow.horizontalAlignment(HorizontalAlignment.CENTER)
        headerRow.gap(GlintTheme.GAP_MD)
        headerRow.padding(GlintTheme.paddingSm())
        buildHeader(headerRow)
        rootComponent.child(headerRow as Component)

        // Content area (scrollable, fills remaining space)
        contentList = Containers.verticalFlow(Sizing.fill(100), Sizing.content())
        contentList.gap(GlintTheme.GAP_SM)
        contentList.padding(GlintTheme.paddingSm())

        contentScroll =
            Containers.verticalScroll(
                Sizing.fill(100),
                Sizing.fill(100),
                contentList,
            )
        contentScroll.surface(Surface.DARK_PANEL)

        buildContent(contentList)
        rootComponent.child(contentScroll as Component)

        // Footer row (fixed, horizontal)
        footerRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        footerRow.horizontalAlignment(HorizontalAlignment.CENTER)
        footerRow.gap(GlintTheme.GAP_MD)
        footerRow.padding(GlintTheme.paddingSm())
        buildFooter(footerRow)
        rootComponent.child(footerRow as Component)
    }

    /**
     * Build header components (buttons, title, etc.)
     */
    protected abstract fun buildHeader(header: FlowLayout)

    /**
     * Build footer components (navigation buttons, etc.)
     */
    protected abstract fun buildFooter(footer: FlowLayout)

    /**
     * Build content list items.
     */
    protected abstract fun buildContent(content: FlowLayout)

    /**
     * Clears and rebuilds the content list.
     */
    protected fun refreshContent() {
        contentList.clearChildren()
        buildContent(contentList)
    }
}
