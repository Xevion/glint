package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.GlintApi
import com.xevion.glint.scene.SceneManager
import net.minecraft.client.gui.components.Button
import net.minecraft.client.gui.layouts.HeaderAndFooterLayout
import net.minecraft.client.gui.layouts.LinearLayout
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import net.minecraft.network.chat.Component
import java.util.concurrent.CompletableFuture

/**
 * Main Glint Hub screen - entry point for world and scene management.
 * Shows worlds from API (if configured) or local JSON files as fallback.
 */
class GlintHubScreen(
    private val lastScreen: Screen?,
) : Screen(Component.literal("Glint Hub")) {
    private val layout = HeaderAndFooterLayout(this)
    private lateinit var worldList: WorldListWidget

    private var loading = false
    private var loadError: String? = null
    private var worldData: List<WorldListWidget.WorldEntry> = emptyList()

    override fun init() {
        addHeader()
        addContents()
        addFooter()
        layout.visitWidgets { addRenderableWidget(it) }
        repositionElements()

        // Load worlds on screen open
        refreshWorlds()
    }

    private fun addHeader() {
        val headerRow = LinearLayout.horizontal().spacing(8)

        headerRow.addChild(
            Button
                .builder(Component.literal("🔄 Refresh")) { refreshWorlds() }
                .width(80)
                .build(),
        )

        headerRow.addChild(
            Button
                .builder(Component.literal("📦 New World")) { /* TODO: Stub for now */ }
                .width(90)
                .build(),
        )

        headerRow.addChild(
            Button
                .builder(Component.literal("⚙️ Settings")) { openSettings() }
                .width(80)
                .build(),
        )

        layout.addToHeader(headerRow)
    }

    private fun addContents() {
        worldList = WorldListWidget(minecraft!!, this)
        layout.addToContents(worldList)
    }

    private fun addFooter() {
        val footer = LinearLayout.horizontal().spacing(8)
        footer.addChild(
            Button
                .builder(CommonComponents.GUI_DONE) { onClose() }
                .width(100)
                .build(),
        )
        layout.addToFooter(footer)
    }

    override fun repositionElements() {
        layout.arrangeElements()
        worldList.updateListSize(width, layout)
    }

    override fun onClose() {
        minecraft?.setScreen(lastScreen)
    }

    /**
     * Refreshes world list from API or local files.
     */
    fun refreshWorlds() {
        loading = true
        loadError = null

        val config = ApiConfig.load()

        if (config.isValid()) {
            loadWorldsFromApi(config)
        } else {
            loadWorldsFromLocalFiles()
        }
    }

    private fun loadWorldsFromApi(config: ApiConfig) {
        CompletableFuture
            .supplyAsync {
                GlintApi.listWorlds(config.apiUrl)
            }.thenAccept { result ->
                minecraft?.execute {
                    loading = false
                    result
                        .onSuccess { apiWorlds ->
                            Glint.LOGGER.info("Loaded {} worlds from API", apiWorlds.size)
                            worldData = apiWorlds.map { WorldListWidget.WorldEntry.fromApi(it) }
                            updateWorldList()
                        }.onFailure { error ->
                            Glint.LOGGER.warn("Failed to load worlds from API, falling back to local: {}", error.message)
                            loadWorldsFromLocalFiles()
                        }
                }
            }
    }

    private fun loadWorldsFromLocalFiles() {
        CompletableFuture
            .supplyAsync {
                val collections = SceneManager.discoverAllCollections()
                collections.map { (fileName, collection) ->
                    WorldListWidget.WorldEntry.fromLocal(fileName, collection)
                }
            }.thenAccept { localWorlds ->
                minecraft?.execute {
                    loading = false
                    worldData = localWorlds
                    Glint.LOGGER.info("Loaded {} worlds from local files", localWorlds.size)
                    updateWorldList()
                }
            }
    }

    private fun updateWorldList() {
        worldList.refreshEntries(worldData)
    }

    private fun openSettings() {
        val config = ApiConfig.load()
        val showConnectionFirst = config.needsValidation()
        minecraft?.setScreen(ApiConfigWizardScreen(this, showConnectionFirst))
    }

    fun getWorldData(): List<WorldListWidget.WorldEntry> = worldData

    fun isLoading(): Boolean = loading

    fun getLoadError(): String? = loadError
}
