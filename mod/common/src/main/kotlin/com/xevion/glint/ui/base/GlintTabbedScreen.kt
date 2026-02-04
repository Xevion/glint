package com.xevion.glint.ui.base

import io.wispforest.owo.ui.base.BaseOwoScreen
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.container.ScrollContainer
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.OwoUIAdapter
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.Surface
import io.wispforest.owo.ui.core.VerticalAlignment
import net.minecraft.network.chat.Component as McComponent

/**
 * Base screen with tab bar, master-detail split, and status footer.
 * Master panel (left) 65%, Detail panel (right) 35%.
 */
abstract class GlintTabbedScreen(
    title: McComponent,
) : BaseOwoScreen<FlowLayout>(title) {
    protected lateinit var tabBar: FlowLayout
    protected lateinit var masterPanel: FlowLayout
    protected lateinit var masterScroll: ScrollContainer<FlowLayout>
    protected lateinit var detailPanel: FlowLayout
    protected lateinit var detailScroll: ScrollContainer<FlowLayout>
    protected lateinit var statusBar: FlowLayout

    protected var activeTab: Int = 0

    override fun createAdapter(): OwoUIAdapter<FlowLayout> = OwoUIAdapter.create(this, Containers::verticalFlow)

    override fun build(rootComponent: FlowLayout) {
        rootComponent
            .surface(Surface.VANILLA_TRANSLUCENT)
            .sizing(Sizing.fill(100), Sizing.fill(100))

        // Tab bar (top)
        tabBar = Containers.horizontalFlow(Sizing.fill(100), Sizing.content())
        tabBar.padding(GlintTheme.paddingSm())
        tabBar.gap(GlintTheme.GAP_SM)
        tabBar.verticalAlignment(VerticalAlignment.CENTER)
        buildTabBar(tabBar)
        rootComponent.child(tabBar as Component)

        // Main content area (master-detail split)
        val contentRow = Containers.horizontalFlow(Sizing.fill(100), Sizing.expand())
        contentRow.gap(GlintTheme.GAP_MD)
        contentRow.padding(GlintTheme.paddingSm())

        // Master panel (left, 65%)
        val masterContainer = Containers.verticalFlow(Sizing.fill(65), Sizing.fill(100))
        masterPanel = Containers.verticalFlow(Sizing.fill(100), Sizing.content())
        masterPanel.gap(GlintTheme.GAP_SM)
        masterScroll = Containers.verticalScroll(Sizing.fill(100), Sizing.fill(100), masterPanel)
        masterScroll.surface(Surface.DARK_PANEL)
        masterScroll.padding(GlintTheme.paddingSm())
        masterContainer.child(masterScroll as Component)
        contentRow.child(masterContainer as Component)

        // Detail panel (right, 35%) - wrapped in scroll for overflow
        val detailContainer = Containers.verticalFlow(Sizing.fill(35), Sizing.fill(100))
        detailPanel = Containers.verticalFlow(Sizing.fill(100), Sizing.content())
        detailPanel.padding(GlintTheme.paddingMd())
        detailPanel.gap(GlintTheme.GAP_MD)
        buildDetailPanel(detailPanel)
        detailScroll = Containers.verticalScroll(Sizing.fill(100), Sizing.fill(100), detailPanel)
        detailScroll.surface(Surface.DARK_PANEL)
        detailContainer.child(detailScroll as Component)
        contentRow.child(detailContainer as Component)

        rootComponent.child(contentRow as Component)

        // Status bar (bottom)
        statusBar = Containers.horizontalFlow(Sizing.fill(100), Sizing.content())
        statusBar.padding(GlintTheme.paddingSm())
        statusBar.gap(GlintTheme.GAP_MD)
        statusBar.verticalAlignment(VerticalAlignment.CENTER)
        buildStatusBar(statusBar)
        rootComponent.child(statusBar as Component)

        // Build initial master content
        buildMasterContent(masterPanel)
    }

    protected abstract fun buildTabBar(tabs: FlowLayout)

    protected abstract fun buildMasterContent(master: FlowLayout)

    protected abstract fun buildDetailPanel(detail: FlowLayout)

    protected abstract fun buildStatusBar(status: FlowLayout)

    protected fun refreshMasterContent() {
        masterPanel.clearChildren()
        buildMasterContent(masterPanel)
    }

    protected fun refreshDetailPanel() {
        detailPanel.clearChildren()
        buildDetailPanel(detailPanel)
    }

    /**
     * Handle click on empty space in master panel to deselect.
     * Call this in subclass init after setting up click handler.
     */
    protected fun setupMasterPanelDeselect(onDeselect: () -> Unit) {
        masterScroll.mouseDown().subscribe { _, _, button ->
            if (button == 0) {
                onDeselect()
                true
            } else {
                false
            }
        }
    }
}
