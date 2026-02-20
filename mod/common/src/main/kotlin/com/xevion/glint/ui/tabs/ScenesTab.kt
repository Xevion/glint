package com.xevion.glint.ui.tabs

import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.ReconcileResult
import com.xevion.glint.api.ReconcileSceneStatus
import com.xevion.glint.api.SceneSyncManager
import com.xevion.glint.api.SyncStatus
import com.xevion.glint.orchestration.PreviewCapture
import com.xevion.glint.scene.LocalPreset
import com.xevion.glint.scene.LocalSceneMetadata
import com.xevion.glint.scene.LocalSceneStore
import com.xevion.glint.scene.SceneApplicator
import com.xevion.glint.scene.SceneApplyResult
import com.xevion.glint.scene.SceneFormatting
import com.xevion.glint.scene.Weather
import com.xevion.glint.ui.GlintMainScreen
import com.xevion.glint.ui.SceneSetupScreen
import com.xevion.glint.ui.SceneUploadProgressDialog
import com.xevion.glint.ui.StatusLog
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintDialogScreen
import com.xevion.glint.ui.base.GlintListComponents
import com.xevion.glint.ui.base.GlintTheme
import com.xevion.glint.ui.components.PresetComponents
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.Insets
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.VerticalAlignment
import net.minecraft.client.Minecraft
import net.minecraft.network.chat.Component as McComponent

class ScenesTab(
    private val host: GlintMainScreen,
) : MainScreenTab {
    data class SceneListEntry(
        val slug: String,
        val name: String,
        val dimension: String,
        val syncStatus: SyncStatus,
        val presetCount: Int,
        val isLoaded: Boolean,
        /** True if this scene only exists on the server (no local package) */
        val isRemoteOnly: Boolean,
    )

    private var sceneEntries: List<SceneListEntry> = emptyList()

    /** Live sync status from the last reconcile call, keyed by slug */
    private var syncStatuses: Map<String, ReconcileSceneStatus> = emptyMap()
    var selectedSceneSlug: String? = null
        private set
    private var editingPresetSlug: String? = null
    private var addingPreset = false

    override fun buildMaster(master: FlowLayout) {
        val headerRow = Containers.horizontalFlow(Sizing.fill(100), Sizing.content())
        headerRow.gap(GlintTheme.GAP_SM)
        headerRow.verticalAlignment(VerticalAlignment.CENTER)

        headerRow.child(
            GlintComponents.title(McComponent.literal("Scenes")),
        )

        val spacer = Containers.horizontalFlow(Sizing.expand(), Sizing.fixed(1))
        headerRow.child(spacer)

        val inSingleplayer = host.client?.singleplayerServer != null
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
                host.client?.setScreen(SceneSetupScreen())
            }
        exportBtn.active = inSingleplayer
        headerRow.child(exportBtn as Component)

        master.child(headerRow)

        if (sceneEntries.isEmpty()) {
            master.child(
                Components
                    .label(McComponent.literal("No scenes yet"))
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)),
            )
            master.child(
                Components
                    .label(
                        McComponent.literal(
                            "Open a singleplayer world and use '+ New Scene' to export your first scene.",
                        ),
                    ).maxWidth(host.masterTextWidth)
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)),
            )
            return
        }

        // Group by sync status
        val localScenes =
            sceneEntries.filter {
                it.syncStatus == SyncStatus.LOCAL_ONLY || it.syncStatus == SyncStatus.UNKNOWN
            }
        val syncedScenes = sceneEntries.filter { it.syncStatus == SyncStatus.SYNCED }
        val needsAttention =
            sceneEntries.filter {
                it.syncStatus == SyncStatus.LOCAL_AHEAD || it.syncStatus == SyncStatus.REMOTE_AHEAD
            }
        val remoteOnly = sceneEntries.filter { it.syncStatus == SyncStatus.REMOTE_ONLY }

        if (needsAttention.isNotEmpty()) {
            master.child(
                Components
                    .label(McComponent.literal("Needs Sync"))
                    .color(Color.ofRgb(GlintTheme.TEXT_WARNING)),
            )
            for (entry in needsAttention) {
                master.child(buildSceneCard(entry))
            }
        }

        if (localScenes.isNotEmpty()) {
            master.child(
                Components
                    .label(McComponent.literal("Local"))
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)),
            )
            val sorted = localScenes.sortedByDescending { it.isLoaded }
            for (entry in sorted) {
                master.child(buildSceneCard(entry))
            }
        }

        if (syncedScenes.isNotEmpty()) {
            master.child(
                Components
                    .label(McComponent.literal("Synced"))
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)),
            )
            val sorted = syncedScenes.sortedBy { it.name }
            for (entry in sorted) {
                master.child(buildSceneCard(entry))
            }
        }

        if (remoteOnly.isNotEmpty()) {
            master.child(
                Components
                    .label(McComponent.literal("Available to Download"))
                    .color(Color.ofRgb(GlintTheme.TEXT_INFO)),
            )
            for (entry in remoteOnly.sortedBy { it.name }) {
                master.child(buildSceneCard(entry))
            }
        }
    }

    private fun buildSceneCard(entry: SceneListEntry): FlowLayout =
        GlintListComponents.sceneCard(
            name = entry.name,
            dimension = entry.dimension,
            syncStatus = entry.syncStatus,
            presetCount = entry.presetCount,
            isSelected = entry.slug == selectedSceneSlug,
            isLoaded = entry.isLoaded,
            onClick = { selectScene(entry.slug) },
        )

    override fun buildDetail(detail: FlowLayout) {
        val slug = selectedSceneSlug
        if (slug == null) {
            detail.horizontalAlignment(HorizontalAlignment.CENTER)
            detail.verticalAlignment(VerticalAlignment.CENTER)
            detail.child(
                Components
                    .label(McComponent.literal("No scene selected"))
                    .maxWidth(host.detailTextWidth)
                    .horizontalTextAlignment(HorizontalAlignment.CENTER)
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)),
            )
            detail.child(
                Components
                    .label(McComponent.literal("Select a scene from the list"))
                    .maxWidth(host.detailTextWidth)
                    .horizontalTextAlignment(HorizontalAlignment.CENTER)
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)),
            )
            return
        }

        val metadata = LocalSceneStore.loadMetadata(slug)
        if (metadata == null) {
            detail.child(
                Components
                    .label(McComponent.literal("Failed to load scene metadata"))
                    .color(Color.ofRgb(GlintTheme.TEXT_ERROR)),
            )
            return
        }

        detail.horizontalAlignment(HorizontalAlignment.LEFT)
        detail.verticalAlignment(VerticalAlignment.TOP)

        detail.child(
            GlintComponents
                .title(McComponent.literal(metadata.name))
                .maxWidth(host.detailTextWidth),
        )
        if (metadata.description != null) {
            detail.child(
                GlintComponents
                    .subtitle(McComponent.literal(metadata.description))
                    .maxWidth(host.detailTextWidth),
            )
        }

        val syncStatus = syncStatuses[slug]?.status ?: SyncStatus.UNKNOWN
        detail.child(
            GlintListComponents.collapsibleSection("Metadata") {
                child(GlintListComponents.itemDetail("Slug: ${metadata.slug}"))
                child(GlintListComponents.itemDetail("Dimension: ${metadata.dimension}"))
                child(GlintListComponents.itemDetail("Minecraft: ${metadata.minecraftVersion}"))
                child(GlintListComponents.itemDetail("Sync: ${syncStatus.name.lowercase().replace('_', ' ')}"))
                child(GlintListComponents.itemDetail("Exported: ${metadata.exportedAt}"))
                val sizeMb = "%.1f".format(metadata.packageSizeBytes / (1024.0 * 1024.0))
                child(GlintListComponents.itemDetail("Package: $sizeMb MB"))
            },
        )

        detail.child(
            GlintListComponents.collapsibleSection("Camera") {
                val cam = metadata.camera
                child(
                    GlintListComponents.itemDetail(
                        "Position: %.1f, %.1f, %.1f".format(cam.x, cam.y, cam.z),
                    ),
                )
                child(
                    GlintListComponents.itemDetail(
                        "Rotation: Yaw %.1f, Pitch %.1f".format(cam.yaw, cam.pitch),
                    ),
                )
                child(GlintListComponents.itemDetail("FOV: ${metadata.fov}"))
                val chunks = (metadata.renderDistance * 2 + 1) * (metadata.renderDistance * 2 + 1)
                child(
                    GlintListComponents.itemDetail(
                        "Render Distance: ${metadata.renderDistance} (~$chunks chunks)",
                    ),
                )
            },
        )

        detail.child(
            GlintListComponents.collapsibleSection("Environment") {
                val env = metadata.environment
                child(GlintListComponents.itemDetail("Time: ${SceneFormatting.formatTime(env.time)}"))
                val weatherDisplay = env.weather.replaceFirstChar { it.uppercase() }
                child(GlintListComponents.itemDetail("Weather: $weatherDisplay"))
                if (env.weatherIntensity > 0f) {
                    child(GlintListComponents.itemDetail("Intensity: %.1f".format(env.weatherIntensity)))
                }
                child(GlintListComponents.itemDetail("Moon: ${SceneFormatting.moonPhaseName(env.moonPhase)}"))
            },
        )

        detail.child(
            buildPresetsSection(slug, metadata),
        )

        detail.child(
            GlintListComponents.collapsibleSection("Versions", defaultExpanded = false) {
                if (metadata.versions.isEmpty()) {
                    child(
                        Components
                            .label(McComponent.literal("No version history"))
                            .color(Color.ofRgb(GlintTheme.TEXT_MUTED)),
                    )
                } else {
                    for ((i, version) in metadata.versions.withIndex()) {
                        val latest = if (i == 0) " (latest)" else ""
                        val sizeMb = "%.1f".format(version.sizeBytes / (1024.0 * 1024.0))
                        child(
                            GlintListComponents.itemDetail(
                                "v${version.version}$latest — ${version.exportedAt} — $sizeMb MB",
                            ),
                        )
                    }
                }
            },
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
        val syncStatus = syncStatuses[metadata.slug]?.status
        val isNewUpload = syncStatus == null || syncStatus == SyncStatus.LOCAL_ONLY || syncStatus == SyncStatus.UNKNOWN
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
                host.client?.setScreen(
                    SceneUploadProgressDialog(
                        parentScreen = host,
                        slug = metadata.slug,
                        sceneName = metadata.name,
                        config = config,
                        syncStatus = syncStatus,
                        onComplete = { refreshScenes() },
                    ),
                )
            }
        uploadBtn.active = canUpload
        buttonRow.child(uploadBtn as Component)

        val inSingleplayer = host.client?.singleplayerServer != null
        if (inSingleplayer) {
            buttonRow.child(
                GlintComponents.smallButton(
                    McComponent.literal("Re-export"),
                    width = 65,
                    tooltip = McComponent.literal("Re-export scene from current world state"),
                ) {
                    host.client?.setScreen(SceneSetupScreen())
                } as Component,
            )
        }

        buttonRow.child(
            GlintComponents.smallButton(
                McComponent.literal("Preview"),
                width = 55,
                tooltip = McComponent.literal("Take a vanilla capture of this scene"),
            ) {
                if (PreviewCapture.start(metadata.slug)) {
                    host.onClose()
                } else {
                    StatusLog.error("Failed to start preview capture")
                    host.triggerRebuildStatusBar()
                }
            } as Component,
        )

        buttonRow.child(
            GlintComponents.smallButton(
                McComponent.literal("Delete"),
                width = 50,
                tooltip = McComponent.literal("Delete this scene"),
            ) {
                confirmDeleteScene(metadata.slug, metadata.name)
            } as Component,
        )

        detail.child(buttonRow)
    }

    private fun buildPresetsSection(
        slug: String,
        metadata: LocalSceneMetadata,
    ): FlowLayout {
        val inSingleplayer = host.client?.singleplayerServer != null

        return GlintListComponents.collapsibleSection("Presets") {
            if (metadata.presets.isEmpty() && !addingPreset) {
                child(
                    Components
                        .label(McComponent.literal("No presets defined"))
                        .color(Color.ofRgb(GlintTheme.TEXT_MUTED)),
                )
            } else {
                for ((index, preset) in metadata.presets.withIndex()) {
                    if (editingPresetSlug == preset.slug) {
                        child(
                            PresetComponents.presetEditForm(
                                initial = preset,
                                isSceneLoaded = inSingleplayer,
                                onSave = { updated -> savePreset(slug, metadata, updated, replacing = preset.slug) },
                                onCancel = {
                                    editingPresetSlug = null
                                    host.triggerRefreshDetail()
                                },
                            ),
                        )
                    } else {
                        child(
                            PresetComponents.presetRow(
                                preset = preset,
                                isDefault = index == 0,
                                isSceneLoaded = inSingleplayer,
                                onEdit = {
                                    editingPresetSlug = preset.slug
                                    addingPreset = false
                                    host.triggerRefreshDetail()
                                },
                                onDelete = { deletePreset(slug, metadata, preset.slug) },
                                onApply = { applyPreset(preset) },
                            ),
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
                        timeOfDayTicks = env.time,
                        weather = env.weather,
                        weatherIntensity = env.weatherIntensity.toDouble(),
                        moonPhase = env.moonPhase,
                    )
                child(
                    PresetComponents.presetEditForm(
                        initial = prefilled,
                        isSceneLoaded = inSingleplayer,
                        onSave = { newPreset -> savePreset(slug, metadata, newPreset, replacing = null) },
                        onCancel = {
                            addingPreset = false
                            host.triggerRefreshDetail()
                        },
                    ),
                )
            } else {
                child(
                    GlintComponents.smallButton(
                        McComponent.literal("+ Add Preset"),
                        width = 80,
                    ) {
                        addingPreset = true
                        editingPresetSlug = null
                        host.triggerRefreshDetail()
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
        host.triggerRefreshDetail()
    }

    private fun deletePreset(
        slug: String,
        metadata: LocalSceneMetadata,
        presetSlug: String,
    ) {
        val presetName = metadata.presets.find { it.slug == presetSlug }?.name ?: presetSlug
        host.client?.setScreen(
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
                                host.client?.setScreen(host)
                                host.triggerRefreshDetail()
                            },
                            onCancel = { host.client?.setScreen(host) },
                        ),
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
        host.triggerRebuildStatusBar()
    }

    private fun confirmDeleteScene(
        slug: String,
        name: String,
    ) {
        host.client?.setScreen(
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
                                host.client?.setScreen(host)
                            },
                            onCancel = { host.client?.setScreen(host) },
                        ),
                    )
                }
            },
        )
    }

    fun selectScene(slug: String) {
        selectedSceneSlug = slug
        editingPresetSlug = null
        addingPreset = false
        host.triggerRefreshMaster()
        host.triggerRefreshDetail()
    }

    fun deselectScene() {
        selectedSceneSlug = null
        editingPresetSlug = null
        addingPreset = false
        host.triggerRefreshMaster()
        host.triggerRefreshDetail()
    }

    fun refreshScenes() {
        LocalSceneStore.clearCache()
        val index = LocalSceneStore.loadIndex()

        // Build entries from local scenes first, all with UNKNOWN status
        val localEntries =
            index.scenes.map { (slug, entry) ->
                SceneListEntry(
                    slug = slug,
                    name = entry.name,
                    dimension = entry.dimension,
                    syncStatus = syncStatuses[slug]?.status ?: SyncStatus.UNKNOWN,
                    presetCount = 0,
                    isLoaded = false,
                    isRemoteOnly = false,
                )
            }

        // Add remote-only scenes from the last reconcile
        val remoteOnlyEntries =
            syncStatuses
                .filter { (slug, status) ->
                    status.status == SyncStatus.REMOTE_ONLY && !index.scenes.containsKey(slug)
                }.mapNotNull { (slug, status) ->
                    val sceneInfo = status.scene ?: return@mapNotNull null
                    SceneListEntry(
                        slug = slug,
                        name = sceneInfo.name,
                        dimension = sceneInfo.dimension,
                        syncStatus = SyncStatus.REMOTE_ONLY,
                        presetCount = 0,
                        isLoaded = false,
                        isRemoteOnly = true,
                    )
                }

        sceneEntries = localEntries + remoteOnlyEntries

        // Kick off async reconciliation against the backend
        val config = ApiConfig.load()
        if (config.isValid()) {
            SceneSyncManager.reconcile(config).thenAccept { result ->
                when (result) {
                    is ReconcileResult.Success -> {
                        syncStatuses = result.scenes
                        // Re-build entries with updated sync statuses
                        val updatedLocal =
                            index.scenes.map { (slug, entry) ->
                                SceneListEntry(
                                    slug = slug,
                                    name = entry.name,
                                    dimension = entry.dimension,
                                    syncStatus = syncStatuses[slug]?.status ?: SyncStatus.UNKNOWN,
                                    presetCount = 0,
                                    isLoaded = false,
                                    isRemoteOnly = false,
                                )
                            }
                        val updatedRemote =
                            syncStatuses
                                .filter { (slug, status) ->
                                    status.status == SyncStatus.REMOTE_ONLY && !index.scenes.containsKey(slug)
                                }.mapNotNull { (slug, status) ->
                                    val sceneInfo = status.scene ?: return@mapNotNull null
                                    SceneListEntry(
                                        slug = slug,
                                        name = sceneInfo.name,
                                        dimension = sceneInfo.dimension,
                                        syncStatus = SyncStatus.REMOTE_ONLY,
                                        presetCount = 0,
                                        isLoaded = false,
                                        isRemoteOnly = true,
                                    )
                                }
                        sceneEntries = updatedLocal + updatedRemote
                        // Schedule UI rebuild on the render thread — owo-lib requires it
                        Minecraft.getInstance().execute { host.triggerRefreshMaster() }
                    }

                    is ReconcileResult.Failure -> {
                        // Keep UNKNOWN status — already set
                    }
                }
            }
        }

        host.triggerRefreshMaster()
        host.triggerRefreshDetail()
    }

    /** Get the current sync status for a scene slug. */
    fun syncStatusFor(slug: String): SyncStatus? = syncStatuses[slug]?.status

    fun hasScenes(): Boolean = sceneEntries.isNotEmpty()
}
