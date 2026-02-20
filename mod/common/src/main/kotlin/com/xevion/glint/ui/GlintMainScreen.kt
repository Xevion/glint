package com.xevion.glint.ui

import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintTabbedScreen
import com.xevion.glint.ui.base.GlintTheme
import com.xevion.glint.ui.tabs.ConfigTab
import com.xevion.glint.ui.tabs.MainScreenTab
import com.xevion.glint.ui.tabs.ScenesTab
import com.xevion.glint.ui.tabs.ShadersTab
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.component.TextBoxComponent
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.Sizing
import net.minecraft.client.Minecraft
import net.minecraft.client.gui.screens.Screen
import org.lwjgl.glfw.GLFW
import net.minecraft.network.chat.Component as McComponent

/**
 * Main Glint screen with tabbed master-detail layout.
 */
class GlintMainScreen(
    private val lastScreen: Screen?,
) : GlintTabbedScreen(McComponent.literal("Glint")) {
    enum class Tab { SCENES, SHADERS, CONFIG }

    private var currentTab = Tab.SCENES

    private val scenesTab = ScenesTab(this)
    private val tabs: Map<Tab, MainScreenTab> =
        mapOf(
            Tab.SCENES to scenesTab,
            Tab.SHADERS to ShadersTab(this),
            Tab.CONFIG to ConfigTab(this),
        )

    // Responsive layout dimensions (exposed for tabs)
    private var columnCount = 2
    var detailTextWidth = 200
        private set
    var masterTextWidth = 400
        private set

    override fun init() {
        // Calculate layout widths before super.init() triggers build()
        val masterWidth = (width * 0.65 - GlintTheme.PADDING_SM * 4).toInt()
        detailTextWidth = (width * 0.35 - GlintTheme.PADDING_MD * 2 - GlintTheme.PADDING_SM * 2).toInt()
        masterTextWidth = (masterWidth - GlintTheme.PADDING_SM * 2).toInt()
        columnCount = maxOf(1, masterWidth / GlintTheme.CARD_MIN_WIDTH)

        super.init()
        setupMasterPanelDeselect {
            if (currentTab == Tab.SCENES) scenesTab.deselectScene()
        }

        // Rebuild panels with recalculated widths (owo-lib reuses the
        // component tree on resize rather than calling build() again)
        refreshMasterContent()
        refreshDetailPanel()

        if (!scenesTab.hasScenes() && currentTab == Tab.SCENES) {
            scenesTab.refreshScenes()
        }
    }

    private fun switchTab(tab: Tab) {
        if (currentTab == tab) return
        currentTab = tab
        rebuildTabBar()
        refreshMasterContent()
        refreshDetailPanel()
    }

    override fun buildTabBar(tabs: FlowLayout) {
        tabs.child(
            GlintComponents.tabButton(
                McComponent.literal("Scenes"),
                isActive = currentTab == Tab.SCENES,
                onClick = { switchTab(Tab.SCENES) },
            ),
        )
        tabs.child(
            GlintComponents.tabButton(
                McComponent.literal("Shaders"),
                isActive = currentTab == Tab.SHADERS,
                onClick = { switchTab(Tab.SHADERS) },
            ),
        )
        tabs.child(
            GlintComponents.tabButton(
                McComponent.literal("Config"),
                isActive = currentTab == Tab.CONFIG,
                onClick = { switchTab(Tab.CONFIG) },
            ),
        )

        val spacer = Containers.horizontalFlow(Sizing.expand(), Sizing.fixed(1))
        tabs.child(spacer)

        tabs.child(
            GlintComponents.smallButton(McComponent.literal("X"), width = 24) { onClose() } as Component,
        )
    }

    override fun buildMasterContent(master: FlowLayout) {
        this.tabs[currentTab]?.buildMaster(master)
    }

    override fun buildDetailPanel(detail: FlowLayout) {
        this.tabs[currentTab]?.buildDetail(detail)
    }

    override fun buildStatusBar(status: FlowLayout) {
        val latestEntry = StatusLog.recent(1).firstOrNull()

        if (latestEntry != null) {
            val color =
                when (latestEntry.level) {
                    StatusLog.Level.INFO -> GlintTheme.TEXT_SUCCESS
                    StatusLog.Level.WARNING -> GlintTheme.TEXT_WARNING
                    StatusLog.Level.ERROR -> GlintTheme.TEXT_ERROR
                }
            status.child(
                Components
                    .label(McComponent.literal("[${latestEntry.formattedTime()}] ${latestEntry.message}"))
                    .color(Color.ofRgb(color)),
            )
        } else {
            status.child(
                Components
                    .label(McComponent.literal("Status: Ready"))
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)),
            )
        }

        val spacer = Containers.horizontalFlow(Sizing.expand(), Sizing.fixed(1))
        status.child(spacer)

        status.child(
            GlintComponents.smallButton(
                McComponent.literal("View Log"),
                width = 60,
            ) {
                minecraft?.setScreen(StatusLogScreen(this))
            } as Component,
        )
    }

    override fun keyPressed(
        keyCode: Int,
        scanCode: Int,
        modifiers: Int,
    ): Boolean {
        // K opens Scene Setup unless a text input is focused (e.g., preset edit form)
        if (keyCode == GLFW.GLFW_KEY_K && focused !is TextBoxComponent) {
            val inSingleplayer = minecraft?.singleplayerServer != null
            if (inSingleplayer) {
                minecraft?.setScreen(SceneSetupScreen())
            }
            return true
        }
        return super.keyPressed(keyCode, scanCode, modifiers)
    }

    override fun onClose() {
        minecraft?.setScreen(lastScreen)
    }

    /** Exposes the Minecraft client instance for tab classes. */
    val client: Minecraft? get() = minecraft

    fun triggerRefreshMaster() = refreshMasterContent()

    fun triggerRefreshDetail() = refreshDetailPanel()

    fun triggerRebuildStatusBar() = rebuildStatusBar()
}
