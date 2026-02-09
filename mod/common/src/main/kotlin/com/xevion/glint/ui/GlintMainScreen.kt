package com.xevion.glint.ui

import com.xevion.glint.Loggers
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.GlintApi
import com.xevion.glint.api.PullResult
import com.xevion.glint.api.PushResult
import com.xevion.glint.api.SceneSyncManager
import com.xevion.glint.api.WorldInfo
import com.xevion.glint.download.WorldDownloader
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
import com.xevion.glint.upload.WorldUploader
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
            GlintComponents.listItemRow(onClick = { openAddWorldPicker() }) {
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
                val apiWorld = world.apiWorld
                if (apiWorld?.latestVersion?.fileUrl != null) {
                    buttonRow.child(
                        GlintComponents.smallButton(
                            McComponent.literal("Download"),
                            width = 65,
                            tooltip = McComponent.literal("Download world from API"),
                        ) { downloadWorld(world) } as Component,
                    )
                } else {
                    val btn =
                        GlintComponents.smallButton(
                            McComponent.literal("Download"),
                            width = 65,
                            tooltip = McComponent.literal("No download available"),
                        ) { }
                    btn.active = false
                    buttonRow.child(btn as Component)
                }
            }

            "local" -> {
                val config = ApiConfig.load()
                val canUpload = config.isValid()
                val uploadBtn =
                    GlintComponents.smallButton(
                        McComponent.literal("Upload"),
                        width = 55,
                        tooltip =
                            McComponent.literal(
                                if (canUpload) "Upload world to API" else "Configure API connection first",
                            ),
                    ) { uploadWorld(world) }
                uploadBtn.active = canUpload
                buttonRow.child(uploadBtn as Component)

                buttonRow.child(
                    GlintComponents.smallButton(
                        McComponent.literal("Delete"),
                        width = 55,
                        tooltip = McComponent.literal("Delete local collection"),
                    ) { confirmDeleteWorld(world) } as Component,
                )
            }

            "synced", "stale" -> {
                val config = ApiConfig.load()
                val canUpload = config.isValid()
                val updateBtn =
                    GlintComponents.smallButton(
                        McComponent.literal("Update"),
                        width = 55,
                        tooltip =
                            McComponent.literal(
                                if (canUpload) "Upload new world version" else "Configure API connection first",
                            ),
                    ) { updateWorldVersion(world) }
                updateBtn.active = canUpload
                buttonRow.child(updateBtn as Component)

                buttonRow.child(
                    GlintComponents.smallButton(
                        McComponent.literal("Pull"),
                        width = 40,
                        tooltip = McComponent.literal("Replace local scenes with API scenes"),
                    ) { pullWorldScenes(world) } as Component,
                )
                buttonRow.child(
                    GlintComponents.smallButton(
                        McComponent.literal("Push"),
                        width = 45,
                        tooltip = McComponent.literal("Push local scenes to API (preview changes first)"),
                    ) { pushWorldScenes(world) } as Component,
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
    // Config Tab
    // ============================================

    private var connectionTestResult: String? = null
    private var connectionTesting = false

    private fun buildConfigMaster(master: FlowLayout) {
        val config = ApiConfig.load()

        master.child(
            GlintComponents.title(McComponent.literal("API Connection")) as Component,
        )

        if (!config.enabled || config.apiUrl.isBlank()) {
            // Not configured state
            master.child(
                Components
                    .label(McComponent.literal("No API connection configured."))
                    .maxWidth(masterTextWidth)
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
            )
            master.child(
                Components
                    .label(McComponent.literal("Set up a connection to sync scenes and download worlds."))
                    .maxWidth(masterTextWidth)
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
            )

            val buttonRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
            buttonRow.gap(GlintTheme.GAP_SM)
            buttonRow.padding(Insets.vertical(GlintTheme.GAP_MD))
            buttonRow.child(
                GlintComponents.smallButton(
                    McComponent.literal("Set Up Connection"),
                    width = 110,
                ) {
                    minecraft?.setScreen(ApiConfigWizardScreen(this, showConnectionFirst = true))
                } as Component,
            )
            master.child(buttonRow as Component)
            return
        }

        // Configured state - show connection info
        val infoContainer = Containers.verticalFlow(Sizing.fill(100), Sizing.content())
        infoContainer.gap(GlintTheme.GAP_SM)
        infoContainer.padding(Insets.vertical(GlintTheme.GAP_SM))

        // Server URL
        infoContainer.child(
            Components
                .label(McComponent.literal("Server"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        infoContainer.child(
            Components
                .label(McComponent.literal(config.apiUrl))
                .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component,
        )

        // Status
        infoContainer.child(
            Components
                .label(McComponent.literal("Status"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )

        val statusColor: Int
        val statusText: String
        when {
            !config.validated -> {
                statusColor = GlintTheme.TEXT_WARNING
                statusText = "Not validated"
            }

            !config.hasValidToken() -> {
                statusColor = GlintTheme.TEXT_ERROR
                statusText = "Token expired"
            }

            config.isTokenExpiringSoon() -> {
                statusColor = GlintTheme.TEXT_WARNING
                val minutesLeft = ((config.tokenExpiresAt - System.currentTimeMillis()) / 60_000).toInt()
                statusText = "Connected (token expires in ${minutesLeft}m)"
            }

            else -> {
                statusColor = GlintTheme.TEXT_SUCCESS
                val hoursLeft = ((config.tokenExpiresAt - System.currentTimeMillis()) / 3_600_000).toInt()
                statusText = "Connected (token expires in ${hoursLeft}h)"
            }
        }
        infoContainer.child(
            Components
                .label(McComponent.literal(statusText))
                .color(Color.ofRgb(statusColor)) as Component,
        )

        master.child(infoContainer as Component)

        // Action buttons
        val row1 = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        row1.gap(GlintTheme.GAP_SM)
        row1.padding(Insets.vertical(GlintTheme.GAP_SM))

        val testBtn =
            GlintComponents.smallButton(
                McComponent.literal(if (connectionTesting) "Testing..." else "Test Connection"),
                width = 100,
                tooltip = McComponent.literal("Test connection to API server"),
            ) {
                if (!connectionTesting) testConnection(config)
            }
        if (connectionTesting) testBtn.active = false
        row1.child(testBtn as Component)

        if (!config.hasValidToken()) {
            row1.child(
                GlintComponents.smallButton(
                    McComponent.literal("Re-authenticate"),
                    width = 100,
                ) {
                    minecraft?.setScreen(ApiConfigWizardScreen(this))
                } as Component,
            )
        }

        master.child(row1 as Component)

        val row2 = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        row2.gap(GlintTheme.GAP_SM)

        row2.child(
            GlintComponents.smallButton(
                McComponent.literal("Change Server"),
                width = 90,
            ) {
                minecraft?.setScreen(ApiConfigWizardScreen(this, showConnectionFirst = true))
            } as Component,
        )

        master.child(row2 as Component)

        // Disconnect button
        val row3 = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        row3.padding(Insets.top(GlintTheme.GAP_MD))
        row3.child(
            GlintComponents.smallButton(
                McComponent.literal("Disconnect"),
                width = 75,
                tooltip = McComponent.literal("Remove API connection"),
            ) {
                disconnectApi()
            } as Component,
        )
        master.child(row3 as Component)
    }

    private fun buildConfigDetail(detail: FlowLayout) {
        detail.horizontalAlignment(HorizontalAlignment.LEFT)
        detail.verticalAlignment(VerticalAlignment.TOP)

        detail.child(
            GlintComponents.title(McComponent.literal("Diagnostics")) as Component,
        )

        val config = ApiConfig.load()

        if (!config.enabled || config.apiUrl.isBlank()) {
            detail.child(
                Components
                    .label(McComponent.literal("Use 'Set Up Connection' to configure the API."))
                    .maxWidth(detailTextWidth)
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
            )
            detail.child(
                Components
                    .label(
                        McComponent.literal(
                            "The wizard will guide you through server URL validation and authentication.",
                        ),
                    ).maxWidth(detailTextWidth)
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
            )
            return
        }

        // Show test result if available
        if (connectionTestResult != null) {
            detail.child(
                Components
                    .label(McComponent.literal("Connection Test"))
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
            )
            val isSuccess = connectionTestResult!!.startsWith("Success")
            detail.child(
                Components
                    .label(McComponent.literal(connectionTestResult!!))
                    .maxWidth(detailTextWidth)
                    .color(Color.ofRgb(if (isSuccess) GlintTheme.TEXT_SUCCESS else GlintTheme.TEXT_ERROR)) as Component,
            )
        }

        // Token info
        if (config.hasValidToken()) {
            val expiresIn = config.tokenExpiresAt - System.currentTimeMillis()
            val hours = (expiresIn / 3_600_000).toInt()
            val minutes = ((expiresIn % 3_600_000) / 60_000).toInt()

            detail.child(
                Components
                    .label(McComponent.literal("Token Expiry"))
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
            )
            detail.child(
                Components
                    .label(McComponent.literal("${hours}h ${minutes}m remaining"))
                    .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component,
            )
        }

        // API endpoint
        detail.child(
            Components
                .label(McComponent.literal("Endpoint"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        detail.child(
            Components
                .label(McComponent.literal("${config.apiUrl}/api"))
                .maxWidth(detailTextWidth)
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
    }

    private fun testConnection(config: ApiConfig) {
        connectionTesting = true
        connectionTestResult = null
        refreshMasterContent()
        refreshDetailPanel()

        CompletableFuture
            .supplyAsync {
                val start = System.currentTimeMillis()
                val result = GlintApi.testConnection(config.apiUrl)
                val latency = System.currentTimeMillis() - start
                Pair(result, latency)
            }.thenAccept { (result, latency) ->
                minecraft?.execute {
                    connectionTesting = false
                    result
                        .onSuccess {
                            connectionTestResult = "Success (${latency}ms)"
                            StatusLog.info("Connection test passed (${latency}ms)")
                        }.onFailure { error ->
                            connectionTestResult = "Failed: ${error.message}"
                            StatusLog.error("Connection test failed: ${error.message}")
                        }
                    refreshMasterContent()
                    refreshDetailPanel()
                    rebuildStatusBar()
                }
            }
    }

    private fun disconnectApi() {
        val disabledConfig = ApiConfig()
        if (ApiConfig.save(disabledConfig)) {
            StatusLog.info("API connection removed")
            connectionTestResult = null
            refreshMasterContent()
            refreshDetailPanel()
            rebuildStatusBar()
            refreshWorlds()
        }
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

        val result = SceneApplicator.apply(resolved)
        if (result != com.xevion.glint.scene.SceneApplyResult.FAILED) {
            StatusLog.info("Applied scene: ${scene.name}")
        } else {
            StatusLog.error("Failed to apply scene: ${scene.name}")
        }
    }

    private fun pullWorldScenes(world: WorldEntry) {
        val config = ApiConfig.load()
        if (!config.isValid()) {
            StatusLog.warn("API config not valid - open API Config to set it up")
            rebuildStatusBar()
            return
        }

        val apiWorldId = world.apiWorld?.id ?: return
        val collectionFileName = world.collectionFileName ?: return

        minecraft?.setScreen(
            ConfirmPullDialog(
                parent = this,
                worldName = world.name,
                localSceneCount = world.sceneCount,
                onConfirm = {
                    StatusLog.info("Pulling scenes for ${world.name}...")
                    rebuildStatusBar()

                    SceneSyncManager
                        .pullScenes(apiWorldId, collectionFileName, config)
                        .thenAccept { result ->
                            minecraft?.execute {
                                when (result) {
                                    is PullResult.Success -> {
                                        StatusLog.info("Pulled ${result.count} scenes for ${world.name}")
                                    }

                                    is PullResult.Failure -> {
                                        StatusLog.error("Pull failed: ${result.error.userMessage}")
                                    }
                                }
                                SceneManager.clearCache()
                                refreshWorlds()
                                rebuildStatusBar()
                            }
                        }
                },
            ),
        )
    }

    private fun pushWorldScenes(world: WorldEntry) {
        val config = ApiConfig.load()
        if (!config.isValid()) {
            StatusLog.warn("API config not valid - open API Config to set it up")
            rebuildStatusBar()
            return
        }

        val apiWorldId = world.apiWorld?.id ?: return

        StatusLog.info("Computing diff for ${world.name}...")
        rebuildStatusBar()

        CompletableFuture
            .supplyAsync {
                GlintApi.fetchScenes(config.apiUrl, apiWorldId, config.accessToken)
            }.thenAccept { fetchResult ->
                minecraft?.execute {
                    fetchResult.fold(
                        onSuccess = { apiScenes ->
                            val diff = SceneSyncManager.computeDiff(world.scenes, apiScenes)
                            if (!diff.hasChanges) {
                                StatusLog.info("${world.name}: already in sync")
                                rebuildStatusBar()
                                return@execute
                            }

                            minecraft?.setScreen(
                                PushDiffDialog(
                                    parent = this,
                                    worldName = world.name,
                                    diff = diff,
                                    onConfirm = {
                                        StatusLog.info("Pushing changes for ${world.name}...")
                                        rebuildStatusBar()

                                        SceneSyncManager
                                            .executePush(diff, apiWorldId, config)
                                            .thenAccept { result ->
                                                minecraft?.execute {
                                                    when (result) {
                                                        is PushResult.Success -> {
                                                            StatusLog.info(
                                                                "${world.name}: ${result.created} created, " +
                                                                    "${result.updated} updated, ${result.removed} removed",
                                                            )
                                                        }

                                                        is PushResult.Failure -> {
                                                            StatusLog.error("Push failed: ${result.error.userMessage}")
                                                        }
                                                    }
                                                    SceneManager.clearCache()
                                                    refreshWorlds()
                                                    rebuildStatusBar()
                                                }
                                            }
                                    },
                                ),
                            )
                        },
                        onFailure = { error ->
                            StatusLog.error("Failed to fetch API scenes: ${error.message}")
                            rebuildStatusBar()
                        },
                    )
                }
            }
    }

    private fun downloadWorld(world: WorldEntry) {
        val apiWorld = world.apiWorld ?: return
        val fileUrl = apiWorld.latestVersion?.fileUrl ?: return

        StatusLog.info("Starting download: ${world.name}...")
        rebuildStatusBar()

        val future =
            WorldDownloader.downloadWorld(
                worldSlug = apiWorld.slug,
                worldId = apiWorld.id,
                fileUrl = fileUrl,
                expectedHash = null,
                progressCallback = { },
            )

        future
            .thenAccept { worldPath ->
                minecraft?.execute {
                    val folderName = worldPath.substringAfterLast("/")
                    SceneManager.addCollectionForApiWorld(
                        worldName = apiWorld.name,
                        folder = folderName,
                        apiWorldId = apiWorld.id,
                    )
                    StatusLog.info("Downloaded: ${world.name}")
                    SceneManager.clearCache()
                    refreshWorlds()
                    rebuildStatusBar()

                    // Auto-pull scenes for the downloaded world
                    val config = ApiConfig.load()
                    if (config.isValid()) {
                        val collectionFileName =
                            apiWorld.name
                                .lowercase()
                                .replace(' ', '_')
                                .replace(Regex("[^a-z0-9_-]"), "")
                        SceneSyncManager
                            .pullScenes(apiWorld.id, collectionFileName, config)
                            .thenAccept { result ->
                                minecraft?.execute {
                                    when (result) {
                                        is PullResult.Success -> {
                                            StatusLog.info("Pulled ${result.count} scenes for ${world.name}")
                                        }

                                        is PullResult.Failure -> {
                                            StatusLog.warn("Could not pull scenes: ${result.error.userMessage}")
                                        }
                                    }
                                    SceneManager.clearCache()
                                    refreshWorlds()
                                    rebuildStatusBar()
                                }
                            }
                    }
                }
            }.exceptionally { e ->
                minecraft?.execute {
                    StatusLog.error("Download failed: ${e.message}")
                    rebuildStatusBar()
                }
                null
            }

        minecraft?.setScreen(WorldDownloadDialog(this, world.name, future))
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
    // World Version Update
    // ============================================

    private fun updateWorldVersion(world: WorldEntry) {
        val config = ApiConfig.load()
        if (!config.isValid()) {
            StatusLog.warn("API config not valid - open API Config to set it up")
            rebuildStatusBar()
            return
        }

        val apiWorld = world.apiWorld ?: return

        val mc =
            net.minecraft.client.Minecraft
                .getInstance()
        val folder = world.collection?.folder ?: world.collectionFileName ?: return
        val worldDir =
            listOf(
                java.io.File(mc.gameDirectory, "glint/worlds/$folder"),
                java.io.File(mc.gameDirectory, "saves/$folder"),
            ).firstOrNull { it.exists() && it.isDirectory }

        if (worldDir == null) {
            StatusLog.error("Could not find world directory for: ${world.name}")
            rebuildStatusBar()
            return
        }

        val currentWorldName = mc.singleplayerServer?.worldData?.levelName
        val isCurrent = currentWorldName != null && worldDir.name == currentWorldName

        StatusLog.info("Updating world version: ${world.name}...")
        rebuildStatusBar()

        val future =
            WorldUploader.uploadWorldVersion(
                worldDir = worldDir,
                worldId = apiWorld.id,
                apiUrl = config.apiUrl,
                token = config.accessToken,
                forceSave = isCurrent,
                progressCallback = { progress ->
                    minecraft?.execute {
                        val screen = minecraft?.screen
                        if (screen is WorldUploadProgressDialog) {
                            screen.updateProgress(progress)
                        }
                    }
                },
            )

        // uploadWorldVersion returns CompletableFuture<String> (version ID), but
        // WorldUploadProgressDialog expects CompletableFuture<WorldInfo>. Adapt by
        // returning the existing apiWorld on success.
        val adaptedFuture = future.thenApply { _ -> apiWorld }

        val progressDialog =
            WorldUploadProgressDialog(
                parent = this,
                worldName = world.name,
                uploadFuture = adaptedFuture,
                onComplete = { _ ->
                    StatusLog.info("Updated version: ${world.name}")
                    SceneManager.clearCache()
                    refreshWorlds()
                    rebuildStatusBar()
                },
            )

        minecraft?.setScreen(progressDialog)
    }

    // ============================================
    // World Upload
    // ============================================

    private fun uploadWorld(world: WorldEntry) {
        val config = ApiConfig.load()
        if (!config.isValid()) {
            StatusLog.warn("API config not valid - open API Config to set it up")
            rebuildStatusBar()
            return
        }

        // Resolve the world directory
        val mc =
            net.minecraft.client.Minecraft
                .getInstance()
        val folder = world.collection?.folder ?: world.collectionFileName ?: return
        val worldDir =
            listOf(
                java.io.File(mc.gameDirectory, "glint/worlds/$folder"),
                java.io.File(mc.gameDirectory, "saves/$folder"),
            ).firstOrNull { it.exists() && it.isDirectory }

        if (worldDir == null) {
            StatusLog.error("Could not find world directory for: ${world.name}")
            rebuildStatusBar()
            return
        }

        // Determine if this is the currently loaded world (needs force-save)
        val currentWorldName = mc.singleplayerServer?.worldData?.levelName
        val isCurrent = currentWorldName != null && worldDir.name == currentWorldName

        minecraft?.setScreen(
            WorldUploadDialog(
                parentScreen = this,
                worldDir = worldDir,
                defaultName = world.name,
            ) { name, slug, description ->
                startUpload(worldDir, name, slug, description, isCurrent, config, world)
            },
        )
    }

    private fun uploadWorldFromDir(
        worldDir: java.io.File,
        worldName: String,
    ) {
        val config = ApiConfig.load()
        if (!config.isValid()) {
            StatusLog.warn("API config not valid - open API Config to set it up")
            rebuildStatusBar()
            return
        }

        val mc =
            net.minecraft.client.Minecraft
                .getInstance()
        val currentWorldName = mc.singleplayerServer?.worldData?.levelName
        val isCurrent = currentWorldName != null && worldDir.name == currentWorldName

        minecraft?.setScreen(
            WorldUploadDialog(
                parentScreen = this,
                worldDir = worldDir,
                defaultName = worldName,
            ) { name, slug, description ->
                startUpload(worldDir, name, slug, description, isCurrent, config, null)
            },
        )
    }

    private fun startUpload(
        worldDir: java.io.File,
        name: String,
        slug: String,
        description: String?,
        forceSave: Boolean,
        config: ApiConfig,
        existingWorld: WorldEntry?,
    ) {
        StatusLog.info("Starting upload: $name...")
        rebuildStatusBar()

        val progressDialog: WorldUploadProgressDialog

        val future =
            WorldUploader.uploadWorld(
                worldDir = worldDir,
                name = name,
                slug = slug,
                description = description,
                minecraftVersion = "1.21.4",
                apiUrl = config.apiUrl,
                token = config.accessToken,
                forceSave = forceSave,
                progressCallback = { progress ->
                    minecraft?.execute {
                        // Update progress dialog if it's the current screen
                        val screen = minecraft?.screen
                        if (screen is WorldUploadProgressDialog) {
                            screen.updateProgress(progress)
                        }
                    }
                },
            )

        progressDialog =
            WorldUploadProgressDialog(
                parent = this,
                worldName = name,
                uploadFuture = future,
                onComplete = { worldInfo ->
                    // Auto-link apiWorldId on the local scene collection
                    val collectionFileName =
                        existingWorld?.collectionFileName
                            ?: name
                                .lowercase()
                                .replace(' ', '_')
                                .replace(Regex("[^a-z0-9_-]"), "")

                    SceneManager.addCollectionForApiWorld(
                        worldName = name,
                        folder = worldDir.name,
                        apiWorldId = worldInfo.id,
                    )

                    StatusLog.info("Uploaded: $name")
                    SceneManager.clearCache()
                    refreshWorlds()
                    rebuildStatusBar()
                },
            )

        minecraft?.setScreen(progressDialog)
    }

    private fun openAddWorldPicker() {
        val localWorlds = WorldUploader.listLocalWorlds()
        if (localWorlds.isEmpty()) {
            StatusLog.warn("No local worlds found in saves/ or glint/worlds/")
            rebuildStatusBar()
            return
        }

        val config = ApiConfig.load()
        if (!config.isValid()) {
            StatusLog.warn("API config not valid - configure it in the Config tab first")
            rebuildStatusBar()
            return
        }

        minecraft?.setScreen(
            WorldPickerDialog(
                parentScreen = this,
                worlds = localWorlds,
            ) { worldDir, worldName ->
                uploadWorldFromDir(worldDir, worldName)
            },
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
            loadMergedWorlds(config)
        } else {
            loadLocalOnlyWorlds()
        }
    }

    private fun loadMergedWorlds(config: ApiConfig) {
        CompletableFuture
            .supplyAsync {
                val localCollections = SceneManager.discoverAllCollections()
                val apiResult = GlintApi.listWorlds(config.apiUrl, config.accessToken)
                Pair(localCollections, apiResult)
            }.thenAccept { (localCollections, apiResult) ->
                minecraft?.execute {
                    loading = false
                    apiResult
                        .onSuccess { apiWorlds ->
                            worldData = mergeWorldSources(apiWorlds, localCollections)
                            val apiCount = worldData.count { it.apiWorld != null }
                            val localCount = worldData.count { it.collection != null }
                            StatusLog.info("Loaded $apiCount API + $localCount local worlds")
                            refreshMasterContent()
                            rebuildStatusBar()
                        }.onFailure { error ->
                            Loggers.Ui.get().warn("Failed to load from API, falling back to local: {}", error.message)
                            StatusLog.warn("API unavailable, using local files")
                            worldData =
                                localCollections.map { (fileName, collection) ->
                                    WorldEntry.fromLocal(fileName, collection)
                                }
                            refreshMasterContent()
                            rebuildStatusBar()
                        }
                }
            }
    }

    private fun loadLocalOnlyWorlds() {
        CompletableFuture
            .supplyAsync {
                SceneManager.discoverAllCollections().map { (fileName, collection) ->
                    WorldEntry.fromLocal(fileName, collection)
                }
            }.thenAccept { localWorlds ->
                minecraft?.execute {
                    loading = false
                    worldData = localWorlds
                    StatusLog.info("Loaded ${localWorlds.size} worlds from local files")
                    refreshMasterContent()
                    rebuildStatusBar()
                }
            }
    }

    private fun mergeWorldSources(
        apiWorlds: List<WorldInfo>,
        localCollections: List<Pair<String, SceneCollection>>,
    ): List<WorldEntry> {
        val result = mutableListOf<WorldEntry>()
        val matchedLocalFiles = mutableSetOf<String>()

        for (apiWorld in apiWorlds) {
            val match =
                localCollections.find { (fileName, collection) ->
                    collection.apiWorldId == apiWorld.id || fileName == apiWorld.slug
                }

            if (match != null) {
                matchedLocalFiles.add(match.first)
                result.add(
                    WorldEntry(
                        id = apiWorld.id,
                        name = apiWorld.name,
                        description = apiWorld.description,
                        scenes = match.second.scenes,
                        collection = match.second,
                        collectionFileName = match.first,
                        apiWorld = apiWorld,
                    ),
                )
            } else {
                result.add(WorldEntry.fromApi(apiWorld))
            }
        }

        for ((fileName, collection) in localCollections) {
            if (fileName !in matchedLocalFiles) {
                result.add(WorldEntry.fromLocal(fileName, collection))
            }
        }

        return result
    }

    override fun onClose() {
        minecraft?.setScreen(lastScreen)
    }
}
