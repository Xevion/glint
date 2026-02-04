package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.GlintApi
import com.xevion.glint.api.WorldInfo
import com.xevion.glint.scene.Scene
import com.xevion.glint.scene.SceneCollection
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintTabbedScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.Surface
import io.wispforest.owo.ui.core.VerticalAlignment
import net.minecraft.client.gui.screens.Screen
import java.util.concurrent.CompletableFuture
import net.minecraft.network.chat.Component as McComponent

/**
 * Main Glint screen with tabbed master-detail layout.
 */
class GlintMainScreen(
    private val lastScreen: Screen?,
) : GlintTabbedScreen(McComponent.literal("Glint")) {
    // World data
    private var loading = false
    private var loadError: String? = null
    private var worldData: List<WorldEntry> = emptyList()
    private var selectedWorldId: String? = null

    data class WorldEntry(
        val id: String,
        val name: String,
        val description: String?,
        val scenes: List<Scene>,
        val collection: SceneCollection?,
        val collectionFileName: String?,
        val apiWorld: WorldInfo?,
    ) {
        companion object {
            fun fromApi(worldInfo: WorldInfo): WorldEntry =
                WorldEntry(
                    id = worldInfo.id,
                    name = worldInfo.name,
                    description = worldInfo.description,
                    scenes = emptyList(),
                    collection = null,
                    collectionFileName = null,
                    apiWorld = worldInfo,
                )

            fun fromLocal(
                fileName: String,
                collection: SceneCollection,
            ): WorldEntry =
                WorldEntry(
                    id = fileName,
                    name = collection.world,
                    description = "Local scenes",
                    scenes = collection.scenes,
                    collection = collection,
                    collectionFileName = fileName,
                    apiWorld = null,
                )
        }

        val isLocal: Boolean get() = apiWorld == null
        val sceneCount: Int get() = scenes.size

        val status: String
            get() =
                when {
                    apiWorld != null && collection != null -> "synced"
                    apiWorld != null -> "remote"
                    collection != null -> "local"
                    else -> "unknown"
                }
    }

    // Grid column tracking for responsive layout
    private var columnCount = 2
    private var lastCalculatedColumns = 0

    override fun init() {
        super.init()
        setupMasterPanelDeselect { deselectWorld() }

        // Calculate column count based on available width
        // Master panel is 65% of screen, minus padding
        val masterWidth = (width * 0.65 - GlintTheme.PADDING_SM * 4).toInt()
        val newColumnCount = maxOf(1, masterWidth / GlintTheme.CARD_MIN_WIDTH)

        if (newColumnCount != lastCalculatedColumns) {
            columnCount = newColumnCount
            lastCalculatedColumns = newColumnCount
            // Refresh grid if column count changed and we have data
            if (worldData.isNotEmpty()) {
                refreshMasterContent()
            }
        }

        if (worldData.isEmpty()) {
            refreshWorlds()
        }
    }

    override fun buildTabBar(tabs: FlowLayout) {
        // Single "Worlds" tab for now
        tabs.child(
            GlintComponents.tabButton(
                McComponent.literal("Worlds"),
                isActive = true,
                onClick = { /* Already on this tab */ },
            ) as Component,
        )

        // Placeholder tabs (disabled for now)
        tabs.child(
            GlintComponents.tabButton(
                McComponent.literal("Shaders"),
                isActive = false,
                onClick = { /* TODO */ },
            ) as Component,
        )

        tabs.child(
            GlintComponents.tabButton(
                McComponent.literal("Config"),
                isActive = false,
                onClick = { /* TODO */ },
            ) as Component,
        )

        // Spacer to push close button right
        val spacer = Containers.horizontalFlow(Sizing.expand(), Sizing.fixed(1))
        tabs.child(spacer as Component)

        // Close button
        tabs.child(
            GlintComponents.smallButton(McComponent.literal("X"), width = 24) { onClose() } as Component,
        )
    }

    override fun buildMasterContent(master: FlowLayout) {
        if (loading) {
            master.child(
                Components
                    .label(McComponent.literal("Loading..."))
                    .color(Color.ofRgb(GlintTheme.TEXT_WARNING)) as Component,
            )
            return
        }

        if (loadError != null) {
            master.child(
                Components
                    .label(McComponent.literal("Error: $loadError"))
                    .color(Color.ofRgb(GlintTheme.TEXT_ERROR)) as Component,
            )
            return
        }

        if (worldData.isEmpty()) {
            master.child(
                Components
                    .label(McComponent.literal("No worlds found"))
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
            )
            return
        }

        // Build grid with dynamic column count
        val fillPercent = 100 / columnCount
        val chunked = worldData.chunked(columnCount)

        for (row in chunked) {
            val rowLayout = Containers.horizontalFlow(Sizing.fill(100), Sizing.content())
            rowLayout.gap(GlintTheme.GAP_MD)

            for (world in row) {
                val card =
                    GlintComponents.worldCard(
                        name = world.name,
                        sceneCount = world.sceneCount,
                        status = world.status,
                        isSelected = world.id == selectedWorldId,
                        onClick = { selectWorld(world.id) },
                    )
                card.sizing(Sizing.fill(fillPercent), Sizing.fixed(GlintTheme.CARD_HEIGHT))
                rowLayout.child(card as Component)
            }

            // Add spacers for incomplete rows
            repeat(columnCount - row.size) {
                val spacer = Containers.horizontalFlow(Sizing.fill(fillPercent), Sizing.fixed(GlintTheme.CARD_HEIGHT))
                rowLayout.child(spacer as Component)
            }

            master.child(rowLayout as Component)
        }

        // Add World button at bottom
        val addButton =
            GlintComponents.listItemRow(onClick = { /* TODO */ }) {
                horizontalAlignment(HorizontalAlignment.CENTER)
                child(
                    Components
                        .label(McComponent.literal("+ Add World"))
                        .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
                )
            }
        master.child(addButton as Component)
    }

    override fun buildDetailPanel(detail: FlowLayout) {
        val selectedWorld = worldData.find { it.id == selectedWorldId }

        if (selectedWorld == null) {
            // Empty state
            detail.horizontalAlignment(HorizontalAlignment.CENTER)
            detail.verticalAlignment(VerticalAlignment.CENTER)
            detail.child(
                Components
                    .label(McComponent.literal("No world selected"))
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
            )
            detail.child(
                Components
                    .label(McComponent.literal("Select a world from the list"))
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
            )
            return
        }

        // World header
        detail.horizontalAlignment(HorizontalAlignment.LEFT)
        detail.verticalAlignment(VerticalAlignment.TOP)

        detail.child(GlintComponents.title(McComponent.literal(selectedWorld.name)) as Component)

        if (selectedWorld.description != null) {
            detail.child(GlintComponents.subtitle(McComponent.literal(selectedWorld.description)) as Component)
        }

        // Metadata
        val metaContainer = Containers.verticalFlow(Sizing.fill(100), Sizing.content())
        metaContainer.gap(GlintTheme.GAP_SM)
        metaContainer.child(GlintComponents.itemDetail("Status: ${selectedWorld.status}") as Component)
        metaContainer.child(GlintComponents.itemDetail("Scenes: ${selectedWorld.sceneCount}") as Component)
        detail.child(metaContainer as Component)

        // Scene list header
        detail.child(
            Components
                .label(McComponent.literal("Scenes"))
                .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component,
        )

        // Scene cards (read-only for now)
        if (selectedWorld.scenes.isEmpty()) {
            detail.child(
                Components
                    .label(McComponent.literal("No scenes"))
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
            )
        } else {
            val sceneScroll =
                Containers.verticalScroll(
                    Sizing.fill(100),
                    Sizing.expand(),
                    Containers.verticalFlow(Sizing.fill(100), Sizing.content()).also { sceneList ->
                        sceneList.gap(GlintTheme.GAP_SM)
                        for (scene in selectedWorld.scenes) {
                            sceneList.child(buildSceneCard(scene) as Component)
                        }
                    },
                )
            detail.child(sceneScroll as Component)
        }
    }

    private fun buildSceneCard(scene: Scene): FlowLayout {
        val card = Containers.verticalFlow(Sizing.fill(100), Sizing.content())
        card.padding(GlintTheme.paddingSm())
        card.gap(GlintTheme.GAP_SM)
        card.surface(Surface.flat(0x22FFFFFF))

        card.child(GlintComponents.itemLabel(scene.name) as Component)
        card.child(GlintComponents.itemDetail(formatSceneDetails(scene)) as Component)

        // Action buttons placeholder (read-only for now)
        val buttonRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        buttonRow.gap(GlintTheme.GAP_SM)
        buttonRow.child(
            GlintComponents.smallButton(
                McComponent.literal("Go"),
                tooltip = McComponent.literal("Teleport"),
            ) { } as Component,
        )
        buttonRow.child(
            GlintComponents.smallButton(
                McComponent.literal("Edit"),
                tooltip = McComponent.literal("Edit scene"),
            ) { } as Component,
        )
        buttonRow.child(
            GlintComponents.smallButton(
                McComponent.literal("Cap"),
                tooltip = McComponent.literal("Capture"),
                width = 36,
            ) { } as Component,
        )
        card.child(buttonRow as Component)

        return card
    }

    private fun formatSceneDetails(scene: Scene): String {
        val time = formatTime(scene.timeOfDay)
        val weather =
            scene.weather.name
                .lowercase()
                .replaceFirstChar { it.uppercase() }
        val dimension = scene.dimension.substringAfter(":")
        return "$time - $weather - $dimension"
    }

    private fun formatTime(ticks: Int): String {
        val hour = ((ticks / 1000 + 6) % 24)
        return when (hour) {
            in 0..5 -> "Night"
            in 6..11 -> "Morning"
            in 12..17 -> "Afternoon"
            else -> "Evening"
        }
    }

    override fun buildStatusBar(status: FlowLayout) {
        // Simple status text for now
        status.child(
            Components
                .label(McComponent.literal("Status: Ready"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )

        val spacer = Containers.horizontalFlow(Sizing.expand(), Sizing.fixed(1))
        status.child(spacer as Component)

        status.child(
            GlintComponents.smallButton(
                McComponent.literal("View Log"),
                width = 60,
            ) { /* TODO */ } as Component,
        )
    }

    private fun selectWorld(worldId: String) {
        selectedWorldId = worldId
        refreshMasterContent()
        refreshDetailPanel()
    }

    private fun deselectWorld() {
        selectedWorldId = null
        refreshMasterContent()
        refreshDetailPanel()
    }

    fun refreshWorlds() {
        loading = true
        loadError = null
        refreshMasterContent()

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
                            worldData = apiWorlds.map { WorldEntry.fromApi(it) }
                            refreshMasterContent()
                        }.onFailure { error ->
                            Glint.LOGGER.warn("Failed to load from API, falling back to local: {}", error.message)
                            loadWorldsFromLocalFiles()
                        }
                }
            }
    }

    private fun loadWorldsFromLocalFiles() {
        CompletableFuture
            .supplyAsync {
                SceneManager.discoverAllCollections().map { (fileName, collection) ->
                    WorldEntry.fromLocal(fileName, collection)
                }
            }.thenAccept { localWorlds ->
                minecraft?.execute {
                    loading = false
                    worldData = localWorlds
                    Glint.LOGGER.info("Loaded {} worlds from local files", localWorlds.size)
                    refreshMasterContent()
                }
            }
    }

    override fun onClose() {
        minecraft?.setScreen(lastScreen)
    }
}
