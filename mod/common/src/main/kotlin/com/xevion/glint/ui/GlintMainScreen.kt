package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.GlintApi
import com.xevion.glint.api.SceneSyncManager
import com.xevion.glint.api.SyncResult
import com.xevion.glint.api.WorldInfo
import com.xevion.glint.orchestration.CaptureSpec
import com.xevion.glint.orchestration.ShaderSpec
import com.xevion.glint.scene.ResolvedScene
import com.xevion.glint.scene.Scene
import com.xevion.glint.scene.SceneApplicator
import com.xevion.glint.scene.SceneCollection
import com.xevion.glint.scene.SceneConfig
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.session.SessionRegistry
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintTabbedScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.Insets
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
    enum class Tab { WORLDS, SHADERS, CONFIG }

    private var currentTab = Tab.WORLDS

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

    // Responsive layout tracking
    private var columnCount = 2
    private var detailTextWidth = 200
    private var masterTextWidth = 400

    override fun init() {
        // Calculate layout widths before super.init() triggers build()
        val masterWidth = (width * 0.65 - GlintTheme.PADDING_SM * 4).toInt()
        detailTextWidth = (width * 0.35 - GlintTheme.PADDING_MD * 2 - GlintTheme.PADDING_SM * 2).toInt()
        masterTextWidth = (masterWidth - GlintTheme.PADDING_SM * 2).toInt()
        columnCount = maxOf(1, masterWidth / GlintTheme.CARD_MIN_WIDTH)

        super.init()
        setupMasterPanelDeselect {
            if (currentTab == Tab.WORLDS) deselectWorld()
        }

        // Rebuild panels with recalculated widths (owo-lib reuses the
        // component tree on resize rather than calling build() again)
        refreshMasterContent()
        refreshDetailPanel()

        if (worldData.isEmpty() && currentTab == Tab.WORLDS) {
            refreshWorlds()
        }
    }

    // ============================================
    // Tab Bar
    // ============================================

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
                McComponent.literal("Worlds"),
                isActive = currentTab == Tab.WORLDS,
                onClick = { switchTab(Tab.WORLDS) },
            ) as Component,
        )
        tabs.child(
            GlintComponents.tabButton(
                McComponent.literal("Shaders"),
                isActive = currentTab == Tab.SHADERS,
                onClick = { switchTab(Tab.SHADERS) },
            ) as Component,
        )
        tabs.child(
            GlintComponents.tabButton(
                McComponent.literal("Config"),
                isActive = currentTab == Tab.CONFIG,
                onClick = { switchTab(Tab.CONFIG) },
            ) as Component,
        )

        val spacer = Containers.horizontalFlow(Sizing.expand(), Sizing.fixed(1))
        tabs.child(spacer as Component)

        tabs.child(
            GlintComponents.smallButton(McComponent.literal("X"), width = 24) { onClose() } as Component,
        )
    }

    // ============================================
    // Master Content (dispatches by tab)
    // ============================================

    override fun buildMasterContent(master: FlowLayout) {
        when (currentTab) {
            Tab.WORLDS -> buildWorldsMaster(master)
            Tab.SHADERS -> buildShadersMaster(master)
            Tab.CONFIG -> buildConfigMaster(master)
        }
    }

    override fun buildDetailPanel(detail: FlowLayout) {
        when (currentTab) {
            Tab.WORLDS -> buildWorldsDetail(detail)
            Tab.SHADERS -> buildShadersDetail(detail)
            Tab.CONFIG -> buildConfigDetail(detail)
        }
    }

    // ============================================
    // Worlds Tab — Master
    // ============================================

    private fun buildWorldsMaster(master: FlowLayout) {
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

            repeat(columnCount - row.size) {
                val spacer = Containers.horizontalFlow(Sizing.fill(fillPercent), Sizing.fixed(GlintTheme.CARD_HEIGHT))
                rowLayout.child(spacer as Component)
            }

            master.child(rowLayout as Component)
        }

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

    // ============================================
    // Worlds Tab — Detail
    // ============================================

    private fun buildWorldsDetail(detail: FlowLayout) {
        val selectedWorld = worldData.find { it.id == selectedWorldId }

        if (selectedWorld == null) {
            detail.horizontalAlignment(HorizontalAlignment.CENTER)
            detail.verticalAlignment(VerticalAlignment.CENTER)
            detail.child(
                Components
                    .label(McComponent.literal("No world selected"))
                    .maxWidth(detailTextWidth)
                    .horizontalTextAlignment(HorizontalAlignment.CENTER)
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
            )
            detail.child(
                Components
                    .label(McComponent.literal("Select a world from the list"))
                    .maxWidth(detailTextWidth)
                    .horizontalTextAlignment(HorizontalAlignment.CENTER)
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
            )
            return
        }

        detail.horizontalAlignment(HorizontalAlignment.LEFT)
        detail.verticalAlignment(VerticalAlignment.TOP)

        detail.child(
            GlintComponents
                .title(McComponent.literal(selectedWorld.name))
                .maxWidth(detailTextWidth) as Component,
        )

        if (selectedWorld.description != null) {
            detail.child(
                GlintComponents
                    .subtitle(McComponent.literal(selectedWorld.description))
                    .maxWidth(detailTextWidth) as Component,
            )
        }

        // Metadata
        val metaContainer = Containers.verticalFlow(Sizing.fill(100), Sizing.content())
        metaContainer.gap(GlintTheme.GAP_SM)
        metaContainer.child(GlintComponents.itemDetail("Status: ${selectedWorld.status}") as Component)
        metaContainer.child(GlintComponents.itemDetail("Scenes: ${selectedWorld.sceneCount}") as Component)
        detail.child(metaContainer as Component)

        // World action buttons
        buildWorldActionButtons(detail, selectedWorld)

        // Scene list header
        detail.child(
            Components
                .label(McComponent.literal("Scenes"))
                .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component,
        )

        if (selectedWorld.scenes.isEmpty()) {
            detail.child(
                Components
                    .label(McComponent.literal("No scenes defined. Click [+ New Scene] to create one."))
                    .maxWidth(detailTextWidth)
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
            )
        } else {
            val sceneScroll =
                Containers.verticalScroll(
                    Sizing.fill(100),
                    Sizing.expand(),
                    Containers.verticalFlow(Sizing.fill(100), Sizing.content()).also { sceneList ->
                        sceneList.gap(GlintTheme.GAP_SM)
                        // Right padding so scene cards don't render under the scrollbar
                        sceneList.padding(Insets.right(GlintTheme.GAP_MD))
                        for (scene in selectedWorld.scenes) {
                            sceneList.child(buildSceneCard(scene, selectedWorld) as Component)
                        }
                    },
                )
            detail.child(sceneScroll as Component)
        }

        // New Scene button (only in singleplayer)
        if (minecraft?.singleplayerServer != null) {
            detail.child(
                GlintComponents.smallButton(
                    McComponent.literal("+ New Scene"),
                    width = 80,
                    tooltip = McComponent.literal("Capture current position as a new scene"),
                ) {
                    openNewSceneDialog(selectedWorld)
                } as Component,
            )
        }
    }

    private fun buildWorldActionButtons(
        detail: FlowLayout,
        world: WorldEntry,
    ) {
        val buttonRow = Containers.horizontalFlow(Sizing.fill(100), Sizing.content())
        buttonRow.gap(GlintTheme.GAP_SM)
        buttonRow.padding(Insets.vertical(GlintTheme.GAP_SM))

        when (world.status) {
            "remote" -> {
                val btn =
                    GlintComponents.smallButton(
                        McComponent.literal("Download"),
                        width = 65,
                        tooltip = McComponent.literal("Not yet implemented"),
                    ) { }
                btn.active = false
                buttonRow.child(btn as Component)
            }

            "local" -> {
                val uploadBtn =
                    GlintComponents.smallButton(
                        McComponent.literal("Upload"),
                        width = 55,
                        tooltip = McComponent.literal("Not yet implemented"),
                    ) { }
                uploadBtn.active = false
                buttonRow.child(uploadBtn as Component)

                buttonRow.child(
                    GlintComponents.smallButton(
                        McComponent.literal("Delete"),
                        width = 55,
                        tooltip = McComponent.literal("Delete local collection"),
                    ) { confirmDeleteWorld(world) } as Component,
                )
            }

            "synced" -> {
                buttonRow.child(
                    GlintComponents.smallButton(
                        McComponent.literal("Sync"),
                        width = 45,
                        tooltip = McComponent.literal("Sync scenes to API"),
                    ) { syncWorld(world) } as Component,
                )
                buttonRow.child(
                    GlintComponents.smallButton(
                        McComponent.literal("Delete"),
                        width = 55,
                        tooltip = McComponent.literal("Delete local collection"),
                    ) { confirmDeleteWorld(world) } as Component,
                )
            }

            "stale" -> {
                buttonRow.child(
                    GlintComponents.smallButton(
                        McComponent.literal("Sync"),
                        width = 45,
                        tooltip = McComponent.literal("Sync scenes to API (out of date)"),
                    ) { syncWorld(world) } as Component,
                )
                buttonRow.child(
                    GlintComponents.smallButton(
                        McComponent.literal("Delete"),
                        width = 55,
                        tooltip = McComponent.literal("Delete local collection"),
                    ) { confirmDeleteWorld(world) } as Component,
                )
            }
        }

        if (buttonRow.children().isNotEmpty()) {
            detail.child(buttonRow as Component)
        }
    }

    // ============================================
    // Shaders Tab (Stub)
    // ============================================

    private fun buildShadersMaster(master: FlowLayout) {
        master.horizontalAlignment(HorizontalAlignment.CENTER)
        master.verticalAlignment(VerticalAlignment.CENTER)
        master.child(
            Components
                .label(McComponent.literal("Shader management coming soon"))
                .maxWidth(masterTextWidth)
                .horizontalTextAlignment(HorizontalAlignment.CENTER)
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
    }

    private fun buildShadersDetail(detail: FlowLayout) {
        detail.horizontalAlignment(HorizontalAlignment.CENTER)
        detail.verticalAlignment(VerticalAlignment.CENTER)
        detail.child(
            Components
                .label(McComponent.literal("No shader selected"))
                .maxWidth(detailTextWidth)
                .horizontalTextAlignment(HorizontalAlignment.CENTER)
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
        detail.child(
            Components
                .label(McComponent.literal("Select from the list to view details."))
                .maxWidth(detailTextWidth)
                .horizontalTextAlignment(HorizontalAlignment.CENTER)
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
    }

    // ============================================
    // Config Tab (Stub)
    // ============================================

    private fun buildConfigMaster(master: FlowLayout) {
        master.horizontalAlignment(HorizontalAlignment.CENTER)
        master.verticalAlignment(VerticalAlignment.CENTER)
        master.child(
            Components
                .label(McComponent.literal("Configuration coming soon"))
                .maxWidth(masterTextWidth)
                .horizontalTextAlignment(HorizontalAlignment.CENTER)
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
    }

    private fun buildConfigDetail(detail: FlowLayout) {
        detail.horizontalAlignment(HorizontalAlignment.CENTER)
        detail.verticalAlignment(VerticalAlignment.CENTER)
        detail.child(
            Components
                .label(McComponent.literal("Settings will appear here."))
                .maxWidth(detailTextWidth)
                .horizontalTextAlignment(HorizontalAlignment.CENTER)
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
    }

    // ============================================
    // Scene Cards
    // ============================================

    private fun buildSceneCard(
        scene: Scene,
        world: WorldEntry,
    ): FlowLayout {
        val card = Containers.verticalFlow(Sizing.fill(100), Sizing.content())
        card.padding(GlintTheme.paddingSm())
        card.gap(GlintTheme.GAP_SM)
        card.surface(Surface.flat(0x22FFFFFF))

        // Tooltip with coordinates + dimension
        val pos = scene.position
        val tooltipText = "%.0f, %.0f, %.0f - %s".format(pos.x, pos.y, pos.z, scene.dimension.substringAfter(":"))
        (card as Component).tooltip(McComponent.literal(tooltipText))

        card.child(GlintComponents.itemLabel(scene.name) as Component)
        card.child(GlintComponents.itemDetail(formatSceneDetails(scene)) as Component)

        val buttonRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        buttonRow.gap(GlintTheme.GAP_SM)

        // Go button
        val inSingleplayer = minecraft?.singleplayerServer != null
        val goBtn =
            GlintComponents.smallButton(
                McComponent.literal("Go"),
                tooltip =
                    if (inSingleplayer) {
                        McComponent.literal(
                            "Teleport to scene",
                        )
                    } else {
                        McComponent.literal("Only available in singleplayer")
                    },
            ) {
                teleportToScene(scene, world)
            }
        if (!inSingleplayer) goBtn.active = false
        buttonRow.child(goBtn as Component)

        // Edit button
        buttonRow.child(
            GlintComponents.smallButton(
                McComponent.literal("Edit"),
                tooltip = McComponent.literal("Edit scene"),
            ) {
                openEditSceneDialog(scene, world)
            } as Component,
        )

        // Capture button
        buttonRow.child(
            GlintComponents.smallButton(
                McComponent.literal("Capture"),
                tooltip = McComponent.literal("Capture screenshot"),
                width = 55,
            ) {
                val spec =
                    CaptureSpec(
                        sceneIds = listOf(scene.id),
                        shaders = listOf(ShaderSpec(filename = null)),
                    )
                if (SessionRegistry.startOrchestration(spec)) {
                    StatusLog.info("Started capture for ${scene.name}")
                    onClose()
                } else {
                    StatusLog.error("Failed to start capture")
                }
                rebuildStatusBar()
            } as Component,
        )
        card.child(buttonRow as Component)

        // Invisible spacer after buttons to absorb scissor bleed from last rendered button
        val clipFix = Containers.horizontalFlow(Sizing.fixed(1), Sizing.fixed(1))
        card.child(clipFix as Component)

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

    // ============================================
    // Status Bar
    // ============================================

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
                    .color(Color.ofRgb(color)) as Component,
            )
        } else {
            status.child(
                Components
                    .label(McComponent.literal("Status: Ready"))
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
            )
        }

        val spacer = Containers.horizontalFlow(Sizing.expand(), Sizing.fixed(1))
        status.child(spacer as Component)

        status.child(
            GlintComponents.smallButton(
                McComponent.literal("View Log"),
                width = 60,
            ) {
                minecraft?.setScreen(StatusLogScreen(this))
            } as Component,
        )
    }

    // ============================================
    // Actions
    // ============================================

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

    private fun teleportToScene(
        scene: Scene,
        world: WorldEntry,
    ) {
        val collection = world.collection
        val fileName = world.collectionFileName

        if (collection == null || fileName == null) {
            StatusLog.error("Cannot teleport: no local collection for ${world.name}")
            rebuildStatusBar()
            return
        }

        val mergedConfig =
            (scene.config ?: SceneConfig())
                .mergeWith(collection.defaultConfig)
                .mergeWith(SceneConfig.DEFAULT)

        val resolved =
            ResolvedScene(
                scene = scene,
                collection = collection,
                config = mergedConfig,
                collectionFileName = fileName,
            )

        StatusLog.info("Teleporting to ${scene.name}...")
        onClose()

        val success = SceneApplicator.apply(resolved)
        if (success) {
            StatusLog.info("Applied scene: ${scene.name}")
        } else {
            StatusLog.error("Failed to apply scene: ${scene.name}")
        }
    }

    private fun syncWorld(world: WorldEntry) {
        val collection = world.collection ?: return
        val config = ApiConfig.load()

        if (!config.isValid()) {
            StatusLog.warn("API config not valid - open API Config to set it up")
            rebuildStatusBar()
            return
        }

        StatusLog.info("Syncing ${world.name}...")
        rebuildStatusBar()

        SceneSyncManager
            .syncCollection(collection, config)
            .thenAccept { results ->
                minecraft?.execute {
                    val successes = results.count { it is SyncResult.Success }
                    val failures = results.count { it is SyncResult.Failure }

                    if (failures > 0) {
                        StatusLog.warn("${world.name}: $successes/${results.size} synced ($failures failed)")
                    } else {
                        StatusLog.info("${world.name}: all $successes scenes synced")
                    }
                    rebuildStatusBar()
                }
            }.exceptionally { e ->
                minecraft?.execute {
                    StatusLog.error("Sync failed for ${world.name}: ${e.message}")
                    rebuildStatusBar()
                }
                null
            }
    }

    private fun confirmDeleteWorld(world: WorldEntry) {
        val fileName = world.collectionFileName ?: return
        minecraft?.setScreen(
            ConfirmDeleteWorldScreen(this, fileName, world.name),
        )
    }

    fun executeDeleteWorld(fileName: String) {
        val mc =
            net.minecraft.client.Minecraft
                .getInstance()
        val sceneFile = java.io.File(mc.gameDirectory, "glint/scenes/$fileName.json")

        if (sceneFile.exists() && sceneFile.delete()) {
            StatusLog.info("Deleted collection: $fileName")
            SceneManager.clearCache()
            selectedWorldId = null
            refreshWorlds()
        } else {
            StatusLog.error("Failed to delete collection: $fileName")
        }
        rebuildStatusBar()
    }

    private fun openNewSceneDialog(world: WorldEntry) {
        minecraft?.setScreen(
            SaveSceneDialog(
                parentScreen = this,
                worldName = world.name,
                onSave = {
                    StatusLog.info("Scene saved to ${world.name}")
                    SceneManager.clearCache()
                    refreshWorlds()
                    rebuildStatusBar()
                },
            ),
        )
    }

    private fun openEditSceneDialog(
        scene: Scene,
        world: WorldEntry,
    ) {
        minecraft?.setScreen(
            SaveSceneDialog(
                parentScreen = this,
                worldName = world.name,
                existingScene = scene,
                onSave = {
                    StatusLog.info("Scene '${scene.name}' updated")
                    SceneManager.clearCache()
                    refreshWorlds()
                    rebuildStatusBar()
                },
            ),
        )
    }

    // ============================================
    // World Data Loading
    // ============================================

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
                            StatusLog.info("Loaded ${apiWorlds.size} worlds from API")
                            worldData = apiWorlds.map { WorldEntry.fromApi(it) }
                            refreshMasterContent()
                            rebuildStatusBar()
                        }.onFailure { error ->
                            Glint.LOGGER.warn("Failed to load from API, falling back to local: {}", error.message)
                            StatusLog.warn("API unavailable, using local files")
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
                    StatusLog.info("Loaded ${localWorlds.size} worlds from local files")
                    refreshMasterContent()
                    rebuildStatusBar()
                }
            }
    }

    override fun onClose() {
        minecraft?.setScreen(lastScreen)
    }
}
