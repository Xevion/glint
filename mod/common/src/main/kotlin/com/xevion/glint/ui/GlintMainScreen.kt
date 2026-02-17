package com.xevion.glint.ui

import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.UrlValidation
import com.xevion.glint.scene.LocalPreset
import com.xevion.glint.scene.LocalSceneMetadata
import com.xevion.glint.scene.LocalSceneStore
import com.xevion.glint.scene.SceneApplicator
import com.xevion.glint.scene.SceneApplyResult
import com.xevion.glint.scene.SceneState
import com.xevion.glint.scene.Weather
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintDialogScreen
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
    enum class Tab { SCENES, SHADERS, CONFIG }

    private var currentTab = Tab.SCENES

    // Scene data
    private var sceneEntries: List<SceneListEntry> = emptyList()
    private var selectedSceneSlug: String? = null

    data class SceneListEntry(
        val slug: String,
        val name: String,
        val dimension: String,
        val state: SceneState,
        val presetCount: Int,
        val needsPush: Boolean,
        val isLoaded: Boolean,
    )

    // Preset editing state
    private var editingPresetSlug: String? = null
    private var addingPreset = false

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
            if (currentTab == Tab.SCENES) deselectScene()
        }

        // Rebuild panels with recalculated widths (owo-lib reuses the
        // component tree on resize rather than calling build() again)
        refreshMasterContent()
        refreshDetailPanel()

        if (sceneEntries.isEmpty() && currentTab == Tab.SCENES) {
            refreshScenes()
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

    override fun buildMasterContent(master: FlowLayout) {
        when (currentTab) {
            Tab.SCENES -> buildScenesMaster(master)
            Tab.SHADERS -> buildShadersMaster(master)
            Tab.CONFIG -> buildConfigMaster(master)
        }
    }

    override fun buildDetailPanel(detail: FlowLayout) {
        when (currentTab) {
            Tab.SCENES -> buildScenesDetail(detail)
            Tab.SHADERS -> buildShadersDetail(detail)
            Tab.CONFIG -> buildConfigDetail(detail)
        }
    }

    private fun buildScenesMaster(master: FlowLayout) {
        val headerRow = Containers.horizontalFlow(Sizing.fill(100), Sizing.content())
        headerRow.gap(GlintTheme.GAP_SM)
        headerRow.verticalAlignment(VerticalAlignment.CENTER)

        headerRow.child(
            GlintComponents.title(McComponent.literal("Scenes")) as Component,
        )

        val spacer = Containers.horizontalFlow(Sizing.expand(), Sizing.fixed(1))
        headerRow.child(spacer as Component)

        val inSingleplayer = minecraft?.singleplayerServer != null
        val exportBtn =
            GlintComponents.smallButton(
                McComponent.literal("+ New Scene"),
                width = 80,
                tooltip =
                    McComponent.literal(
                        if (inSingleplayer) {
                            "Export current position as a new scene"
                        } else {
                            "Only available in singleplayer"
                        },
                    ),
            ) {
                minecraft?.setScreen(
                    ExportSceneDialog(parentScreen = this) {
                        refreshScenes()
                    },
                )
            }
        exportBtn.active = inSingleplayer
        headerRow.child(exportBtn as Component)

        master.child(headerRow as Component)

        if (sceneEntries.isEmpty()) {
            master.child(
                Components
                    .label(McComponent.literal("No scenes yet"))
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
            )
            master.child(
                Components
                    .label(
                        McComponent.literal(
                            "Open a singleplayer world and use '+ New Scene' to export your first scene.",
                        ),
                    ).maxWidth(masterTextWidth)
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
            )
            return
        }

        val localScenes = sceneEntries.filter { it.state == SceneState.LOCAL }
        val syncedScenes = sceneEntries.filter { it.state == SceneState.SYNCED }

        if (localScenes.isNotEmpty()) {
            master.child(
                Components
                    .label(McComponent.literal("Local"))
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
            )
            val sorted = localScenes.sortedByDescending { it.isLoaded }
            for (entry in sorted) {
                master.child(
                    GlintComponents.sceneCard(
                        name = entry.name,
                        dimension = entry.dimension,
                        state = entry.state,
                        presetCount = entry.presetCount,
                        isSelected = entry.slug == selectedSceneSlug,
                        isLoaded = entry.isLoaded,
                        needsPush = entry.needsPush,
                        onClick = { selectScene(entry.slug) },
                    ) as Component,
                )
            }
        }

        if (syncedScenes.isNotEmpty()) {
            master.child(
                Components
                    .label(McComponent.literal("Synced"))
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
            )
            val sorted = syncedScenes.sortedBy { it.name }
            for (entry in sorted) {
                master.child(
                    GlintComponents.sceneCard(
                        name = entry.name,
                        dimension = entry.dimension,
                        state = entry.state,
                        presetCount = entry.presetCount,
                        isSelected = entry.slug == selectedSceneSlug,
                        isLoaded = entry.isLoaded,
                        needsPush = entry.needsPush,
                        onClick = { selectScene(entry.slug) },
                    ) as Component,
                )
            }
        }
    }

    private fun buildScenesDetail(detail: FlowLayout) {
        val slug = selectedSceneSlug
        if (slug == null) {
            detail.horizontalAlignment(HorizontalAlignment.CENTER)
            detail.verticalAlignment(VerticalAlignment.CENTER)
            detail.child(
                Components
                    .label(McComponent.literal("No scene selected"))
                    .maxWidth(detailTextWidth)
                    .horizontalTextAlignment(HorizontalAlignment.CENTER)
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
            )
            detail.child(
                Components
                    .label(McComponent.literal("Select a scene from the list"))
                    .maxWidth(detailTextWidth)
                    .horizontalTextAlignment(HorizontalAlignment.CENTER)
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
            )
            return
        }

        val metadata = LocalSceneStore.loadMetadata(slug)
        if (metadata == null) {
            detail.child(
                Components
                    .label(McComponent.literal("Failed to load scene metadata"))
                    .color(Color.ofRgb(GlintTheme.TEXT_ERROR)) as Component,
            )
            return
        }

        detail.horizontalAlignment(HorizontalAlignment.LEFT)
        detail.verticalAlignment(VerticalAlignment.TOP)

        detail.child(
            GlintComponents
                .title(McComponent.literal(metadata.name))
                .maxWidth(detailTextWidth) as Component,
        )
        if (metadata.description != null) {
            detail.child(
                GlintComponents
                    .subtitle(McComponent.literal(metadata.description))
                    .maxWidth(detailTextWidth) as Component,
            )
        }

        detail.child(
            GlintComponents.collapsibleSection("Metadata") {
                child(GlintComponents.itemDetail("Slug: ${metadata.slug}") as Component)
                child(GlintComponents.itemDetail("Dimension: ${metadata.dimension}") as Component)
                child(GlintComponents.itemDetail("Minecraft: ${metadata.minecraftVersion}") as Component)
                child(GlintComponents.itemDetail("State: ${metadata.state}") as Component)
                child(GlintComponents.itemDetail("Exported: ${metadata.exportedAt}") as Component)
                val sizeMb = "%.1f".format(metadata.packageSizeBytes / (1024.0 * 1024.0))
                child(GlintComponents.itemDetail("Package: $sizeMb MB") as Component)
            } as Component,
        )

        detail.child(
            GlintComponents.collapsibleSection("Camera") {
                val cam = metadata.camera
                child(
                    GlintComponents.itemDetail(
                        "Position: %.1f, %.1f, %.1f".format(cam.x, cam.y, cam.z),
                    ) as Component,
                )
                child(
                    GlintComponents.itemDetail(
                        "Rotation: Yaw %.1f, Pitch %.1f".format(cam.yaw, cam.pitch),
                    ) as Component,
                )
                child(GlintComponents.itemDetail("FOV: ${metadata.fov}") as Component)
                val chunks = (metadata.renderDistance * 2 + 1) * (metadata.renderDistance * 2 + 1)
                child(
                    GlintComponents.itemDetail(
                        "Render Distance: ${metadata.renderDistance} (~$chunks chunks)",
                    ) as Component,
                )
            } as Component,
        )

        detail.child(
            GlintComponents.collapsibleSection("Environment") {
                val env = metadata.environment
                val timeName =
                    when {
                        env.time < 1000L -> "Night"
                        env.time < 6000L -> "Morning"
                        env.time < 9000L -> "Noon"
                        env.time < 13000L -> "Afternoon"
                        env.time < 18000L -> "Evening"
                        else -> "Night"
                    }
                child(GlintComponents.itemDetail("Time: ${env.time} ($timeName)") as Component)
                val weatherDisplay = env.weather.replaceFirstChar { it.uppercase() }
                child(GlintComponents.itemDetail("Weather: $weatherDisplay") as Component)
                if (env.weatherIntensity > 0f) {
                    child(GlintComponents.itemDetail("Intensity: %.1f".format(env.weatherIntensity)) as Component)
                }
                child(GlintComponents.itemDetail("Moon Phase: ${env.moonPhase}") as Component)
            } as Component,
        )

        detail.child(
            buildPresetsSection(slug, metadata) as Component,
        )

        detail.child(
            GlintComponents.collapsibleSection("Versions", defaultExpanded = false) {
                if (metadata.versions.isEmpty()) {
                    child(
                        Components
                            .label(McComponent.literal("No version history"))
                            .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
                    )
                } else {
                    for ((i, version) in metadata.versions.withIndex()) {
                        val latest = if (i == 0) " (latest)" else ""
                        val sizeMb = "%.1f".format(version.sizeBytes / (1024.0 * 1024.0))
                        child(
                            GlintComponents.itemDetail(
                                "v${version.version}$latest — ${version.exportedAt} — $sizeMb MB",
                            ) as Component,
                        )
                    }
                }
            } as Component,
        )

        buildSceneActionBar(detail, metadata)
    }

    private fun buildSceneActionBar(
        detail: FlowLayout,
        metadata: LocalSceneMetadata,
    ) {
        val buttonRow = Containers.horizontalFlow(Sizing.fill(100), Sizing.content())
        buttonRow.gap(GlintTheme.GAP_SM)
        buttonRow.padding(Insets.vertical(GlintTheme.GAP_SM))

        val config = ApiConfig.load()
        val canUpload = config.isValid()
        val isNewUpload = metadata.backend == null
        val uploadLabel = if (isNewUpload) "Upload" else "Push Update"
        val uploadBtn =
            GlintComponents.smallButton(
                McComponent.literal(uploadLabel),
                width = if (isNewUpload) 55 else 80,
                tooltip =
                    McComponent.literal(
                        if (canUpload) "Upload scene to backend" else "Configure API connection first",
                    ),
            ) {
                minecraft?.setScreen(
                    SceneUploadProgressDialog(
                        parentScreen = this,
                        slug = metadata.slug,
                        sceneName = metadata.name,
                        config = config,
                        onComplete = { refreshScenes() },
                    ),
                )
            }
        uploadBtn.active = canUpload
        buttonRow.child(uploadBtn as Component)

        val inSingleplayer = minecraft?.singleplayerServer != null
        if (inSingleplayer) {
            buttonRow.child(
                GlintComponents.smallButton(
                    McComponent.literal("Re-export"),
                    width = 65,
                    tooltip = McComponent.literal("Re-export scene from current world state"),
                ) {
                    minecraft?.setScreen(
                        ExportSceneDialog(parentScreen = this) { refreshScenes() },
                    )
                } as Component,
            )
        }

        buttonRow.child(
            GlintComponents.smallButton(
                McComponent.literal("Delete"),
                width = 50,
                tooltip = McComponent.literal("Delete this scene"),
            ) {
                confirmDeleteScene(metadata.slug, metadata.name)
            } as Component,
        )

        detail.child(buttonRow as Component)
    }

    private fun buildPresetsSection(
        slug: String,
        metadata: LocalSceneMetadata,
    ): FlowLayout {
        val inSingleplayer = minecraft?.singleplayerServer != null

        return GlintComponents.collapsibleSection("Presets") {
            if (metadata.presets.isEmpty() && !addingPreset) {
                child(
                    Components
                        .label(McComponent.literal("No presets defined"))
                        .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
                )
            } else {
                for ((index, preset) in metadata.presets.withIndex()) {
                    if (editingPresetSlug == preset.slug) {
                        child(
                            GlintComponents.presetEditForm(
                                initial = preset,
                                isSceneLoaded = inSingleplayer,
                                onSave = { updated -> savePreset(slug, metadata, updated, replacing = preset.slug) },
                                onCancel = {
                                    editingPresetSlug = null
                                    refreshDetailPanel()
                                },
                            ) as Component,
                        )
                    } else {
                        child(
                            GlintComponents.presetRow(
                                preset = preset,
                                isDefault = index == 0,
                                isSceneLoaded = inSingleplayer,
                                onEdit = {
                                    editingPresetSlug = preset.slug
                                    addingPreset = false
                                    refreshDetailPanel()
                                },
                                onDelete = { deletePreset(slug, metadata, preset.slug) },
                                onApply = { applyPreset(preset) },
                            ) as Component,
                        )
                    }
                }
            }

            if (addingPreset) {
                val env = metadata.environment
                val prefilled =
                    LocalPreset(
                        name = "",
                        slug = "",
                        timeOfDayTicks = env.time.toInt(),
                        weather = env.weather,
                        weatherIntensity = env.weatherIntensity.toDouble(),
                        moonPhase = env.moonPhase,
                    )
                child(
                    GlintComponents.presetEditForm(
                        initial = prefilled,
                        isSceneLoaded = inSingleplayer,
                        onSave = { newPreset -> savePreset(slug, metadata, newPreset, replacing = null) },
                        onCancel = {
                            addingPreset = false
                            refreshDetailPanel()
                        },
                    ) as Component,
                )
            } else {
                child(
                    GlintComponents.smallButton(
                        McComponent.literal("+ Add Preset"),
                        width = 80,
                    ) {
                        addingPreset = true
                        editingPresetSlug = null
                        refreshDetailPanel()
                    } as Component,
                )
            }
        }
    }

    private fun savePreset(
        slug: String,
        metadata: LocalSceneMetadata,
        preset: LocalPreset,
        replacing: String?,
    ) {
        val updatedPresets =
            if (replacing != null) {
                metadata.presets.map { if (it.slug == replacing) preset else it }
            } else {
                metadata.presets + preset
            }
        val updatedMetadata = metadata.copy(presets = updatedPresets)
        LocalSceneStore.saveMetadata(slug, updatedMetadata)
        editingPresetSlug = null
        addingPreset = false
        refreshDetailPanel()
    }

    private fun deletePreset(
        slug: String,
        metadata: LocalSceneMetadata,
        presetSlug: String,
    ) {
        val presetName = metadata.presets.find { it.slug == presetSlug }?.name ?: presetSlug
        minecraft?.setScreen(
            object : GlintDialogScreen(McComponent.literal("Delete Preset")) {
                override fun buildDialog(dialog: FlowLayout) {
                    dialog.child(
                        GlintComponents.confirmationContent(
                            title = McComponent.literal("Delete Preset"),
                            description = McComponent.literal("Delete preset '$presetName'?"),
                            confirmText = McComponent.literal("Delete"),
                            onConfirm = {
                                val updatedPresets = metadata.presets.filter { it.slug != presetSlug }
                                val updatedMetadata = metadata.copy(presets = updatedPresets)
                                LocalSceneStore.saveMetadata(slug, updatedMetadata)
                                minecraft?.setScreen(this@GlintMainScreen)
                                refreshDetailPanel()
                            },
                            onCancel = { minecraft?.setScreen(this@GlintMainScreen) },
                        ) as Component,
                    )
                }
            },
        )
    }

    private fun applyPreset(preset: LocalPreset) {
        val weather = Weather.fromString(preset.weather)
        val result =
            SceneApplicator.applyEnvironment(
                timeOfDay = preset.timeOfDayTicks,
                weather = weather,
                weatherIntensity = preset.weatherIntensity.toFloat(),
                moonPhase = preset.moonPhase ?: 0,
            )
        if (result == SceneApplyResult.FAILED) {
            StatusLog.error("Failed to apply preset: ${preset.name}")
        } else {
            StatusLog.info("Applied preset: ${preset.name}")
        }
        rebuildStatusBar()
    }

    private fun confirmDeleteScene(
        slug: String,
        name: String,
    ) {
        minecraft?.setScreen(
            object : GlintDialogScreen(McComponent.literal("Delete Scene")) {
                override fun buildDialog(dialog: FlowLayout) {
                    dialog.child(
                        GlintComponents.confirmationContent(
                            title = McComponent.literal("Delete Scene"),
                            description = McComponent.literal("Delete '$name'? This cannot be undone."),
                            confirmText = McComponent.literal("Delete"),
                            onConfirm = {
                                LocalSceneStore.deleteScene(slug)
                                selectedSceneSlug = null
                                refreshScenes()
                                minecraft?.setScreen(this@GlintMainScreen)
                            },
                            onCancel = { minecraft?.setScreen(this@GlintMainScreen) },
                        ) as Component,
                    )
                }
            },
        )
    }

    private fun selectScene(slug: String) {
        selectedSceneSlug = slug
        editingPresetSlug = null
        addingPreset = false
        refreshMasterContent()
        refreshDetailPanel()
    }

    private fun deselectScene() {
        selectedSceneSlug = null
        editingPresetSlug = null
        addingPreset = false
        refreshMasterContent()
        refreshDetailPanel()
    }

    fun refreshScenes() {
        LocalSceneStore.clearCache()
        val index = LocalSceneStore.loadIndex()
        sceneEntries =
            index.scenes.map { (slug, entry) ->
                SceneListEntry(
                    slug = slug,
                    name = entry.name,
                    dimension = entry.dimension,
                    state = entry.state,
                    presetCount = 0,
                    needsPush = LocalSceneStore.needsPush(slug),
                    isLoaded = false,
                )
            }
        refreshMasterContent()
        refreshDetailPanel()
    }

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
                val result = UrlValidation.testConnection(config.apiUrl, token = config.accessToken)
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
        }
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

    override fun onClose() {
        minecraft?.setScreen(lastScreen)
    }
}
