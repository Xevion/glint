package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.GlintApi
import com.xevion.glint.api.SceneSyncManager
import com.xevion.glint.api.SyncResult
import com.xevion.glint.api.WorldInfo
import com.xevion.glint.download.WorldDownloader
import com.xevion.glint.scene.ResolvedScene
import com.xevion.glint.scene.Scene
import com.xevion.glint.scene.SceneApplicator
import com.xevion.glint.scene.SceneCollection
import com.xevion.glint.scene.SceneConfig
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.session.SessionRegistry
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintListScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.Sizing
import net.minecraft.client.Minecraft
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import java.util.concurrent.CompletableFuture
import net.minecraft.network.chat.Component as McComponent

/**
 * Main Glint Hub screen - entry point for world and scene management.
 * Shows worlds from API (if configured) or local JSON files as fallback.
 */
class GlintHubScreen(
    private val lastScreen: Screen?,
) : GlintListScreen(McComponent.literal("Glint Hub")) {
    private var loading = false
    private var loadError: String? = null
    private var worldData: List<WorldEntry> = emptyList()
    private val expandedWorlds = mutableSetOf<String>()

    /**
     * World entry - can be from API or local JSON file.
     */
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
                    description = "Local scenes (${collection.scenes.size} scenes)",
                    scenes = collection.scenes,
                    collection = collection,
                    collectionFileName = fileName,
                    apiWorld = null,
                )
        }

        val isLocal: Boolean get() = apiWorld == null
        val sceneCount: Int get() = scenes.size
    }

    override fun buildHeader(header: FlowLayout) {
        header.child(
            GlintComponents.button(McComponent.literal("Refresh")) { refreshWorlds() } as Component,
        )
        header.child(
            GlintComponents.button(McComponent.literal("New World")) { /* TODO */ } as Component,
        )
        header.child(
            GlintComponents.button(McComponent.literal("Settings")) { openSettings() } as Component,
        )
    }

    override fun buildFooter(footer: FlowLayout) {
        footer.child(
            GlintComponents.button(CommonComponents.GUI_DONE) { onClose() } as Component,
        )
    }

    override fun buildContent(content: FlowLayout) {
        if (loading) {
            content.child(
                Components
                    .label(McComponent.literal("Loading worlds..."))
                    .color(Color.ofRgb(GlintTheme.TEXT_WARNING)) as Component,
            )
            return
        }

        if (loadError != null) {
            content.child(
                Components
                    .label(McComponent.literal("Error: $loadError"))
                    .color(Color.ofRgb(GlintTheme.TEXT_ERROR)) as Component,
            )
            return
        }

        if (worldData.isEmpty()) {
            content.child(
                Components
                    .label(McComponent.literal("No worlds found"))
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
            )
            return
        }

        for (world in worldData) {
            buildWorldEntry(content, world)

            if (expandedWorlds.contains(world.id)) {
                for (scene in world.scenes) {
                    buildSceneEntry(content, scene, world)
                }
            }
        }
    }

    private fun buildWorldEntry(
        content: FlowLayout,
        world: WorldEntry,
    ) {
        val isExpanded = expandedWorlds.contains(world.id)

        val openBtn =
            GlintComponents.iconButton("+", McComponent.literal("Open world (download if needed)")) {
                openWorld(world)
            }

        val settingsBtn =
            GlintComponents.iconButton("*", McComponent.literal("World settings")) {
                // TODO: World settings
            }

        val row =
            GlintComponents.listItemWithButtons(
                indent = GlintTheme.INDENT_LEVEL_1,
                contentBuilder = {
                    child(GlintComponents.expandToggle(isExpanded) { toggleWorldExpanded(world.id) } as Component)
                    child(GlintComponents.itemLabel(world.name) as Component)
                    child(GlintComponents.itemDetail("(${world.sceneCount} scenes)") as Component)
                },
                buttons = arrayOf(openBtn, settingsBtn),
            )
        content.child(row as Component)
    }

    private fun buildSceneEntry(
        content: FlowLayout,
        scene: Scene,
        world: WorldEntry,
    ) {
        val inCorrectWorld = isInCorrectWorld(world)

        val teleportBtn =
            GlintComponents.iconButton(
                "o",
                McComponent.literal("Teleport to scene (must be in world)"),
            ) {
                teleportToScene(scene, world)
            }
        teleportBtn.active = inCorrectWorld

        val captureBtn =
            GlintComponents.iconButton(
                "C",
                McComponent.literal("Test capture (must be in world)"),
            ) {
                startCapture(scene)
            }
        captureBtn.active = inCorrectWorld

        val syncBtn =
            GlintComponents.iconButton("^", McComponent.literal("Sync scene to API")) {
                syncScene(scene)
            }

        val disableBtn =
            GlintComponents.iconButton("X", McComponent.literal("Disable scene")) {
                disableScene(scene, world)
            }

        val row =
            GlintComponents.listItemWithButtons(
                indent = GlintTheme.INDENT_LEVEL_2,
                contentBuilder = {
                    // Spacer for alignment (no expand toggle for scenes in hub)
                    child(Containers.horizontalFlow(Sizing.fixed(24), Sizing.fixed(1)) as Component)

                    val textContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
                    textContainer.gap(2)
                    textContainer.child(GlintComponents.itemLabel(scene.name) as Component)
                    textContainer.child(GlintComponents.itemDetail(formatSceneDetails(scene)) as Component)
                    child(textContainer as Component)
                },
                buttons = arrayOf(teleportBtn, captureBtn, syncBtn, disableBtn),
            )
        content.child(row as Component)
    }

    private fun formatSceneDetails(scene: Scene): String {
        val time = formatTime(scene.timeOfDay)
        val weather =
            scene.weather.name
                .lowercase()
                .replaceFirstChar { it.uppercase() }
        val dimension = scene.dimension.substringAfter(":")
        return "$time | $weather | $dimension"
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

    override fun onClose() {
        minecraft?.setScreen(lastScreen)
    }

    override fun init() {
        super.init()
        // Load worlds on screen open
        if (worldData.isEmpty()) {
            refreshWorlds()
        }
    }

    fun refreshWorlds() {
        loading = true
        loadError = null
        refreshContent()

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
                            refreshContent()
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
                    WorldEntry.fromLocal(fileName, collection)
                }
            }.thenAccept { localWorlds ->
                minecraft?.execute {
                    loading = false
                    worldData = localWorlds
                    Glint.LOGGER.info("Loaded {} worlds from local files", localWorlds.size)
                    refreshContent()
                }
            }
    }

    private fun toggleWorldExpanded(worldId: String) {
        if (expandedWorlds.contains(worldId)) {
            expandedWorlds.remove(worldId)
        } else {
            expandedWorlds.add(worldId)
        }
        refreshContent()
    }

    private fun openSettings() {
        val config = ApiConfig.load()
        val showConnectionFirst = config.needsValidation()
        minecraft?.setScreen(ApiConfigWizardScreen(this, showConnectionFirst))
    }

    private fun openWorld(world: WorldEntry) {
        if (world.isLocal) {
            val worldFolder = world.collectionFileName ?: return
            Glint.LOGGER.info("Opening local world: ${world.name} (folder: $worldFolder)")
            val mc = Minecraft.getInstance()
            mc.createWorldOpenFlows().openWorld(worldFolder) {
                Glint.LOGGER.debug("World load cancelled for: $worldFolder")
            }
        } else {
            val apiWorld = world.apiWorld ?: return
            val fileUrl = apiWorld.fileUrl

            if (fileUrl == null) {
                Glint.LOGGER.error("World ${world.name} has no download URL")
                return
            }

            Glint.LOGGER.info("Starting download for world: ${world.name}")

            val downloadDialog =
                WorldDownloadDialog(
                    this,
                    world.name,
                    WorldDownloader.downloadWorld(
                        worldSlug = apiWorld.slug,
                        worldId = apiWorld.id,
                        fileUrl = fileUrl,
                        expectedHash = null,
                    ) { progress ->
                        minecraft?.execute {
                            (minecraft?.screen as? WorldDownloadDialog)?.updateProgress(progress)
                        }
                    },
                )

            minecraft?.setScreen(downloadDialog)
        }
    }

    private fun isInCorrectWorld(world: WorldEntry): Boolean {
        val mc = Minecraft.getInstance()
        if (mc.level == null || mc.singleplayerServer == null) return false

        val expectedFolder = world.collectionFileName ?: return false
        val currentWorldName = mc.singleplayerServer?.worldData?.levelName
        return currentWorldName == expectedFolder
    }

    private fun teleportToScene(
        scene: Scene,
        world: WorldEntry,
    ) {
        val collection = world.collection ?: return
        val fileName = world.collectionFileName ?: return

        val resolved = resolveScene(scene, collection, fileName)
        SceneApplicator.apply(resolved)
        onClose()
    }

    private fun startCapture(scene: Scene) {
        if (SessionRegistry.startCaptureSession(sceneId = scene.id)) {
            Glint.LOGGER.info("Starting capture for scene: ${scene.id}")
            onClose()
        } else {
            Glint.LOGGER.error("Failed to start capture for scene: ${scene.id}")
        }
    }

    private fun syncScene(scene: Scene) {
        val config = ApiConfig.load()
        if (!config.isValid()) {
            Glint.LOGGER.warn("Cannot sync - API not configured")
            return
        }

        SceneSyncManager
            .syncScene(scene, config)
            .thenAccept { result ->
                when (result) {
                    is SyncResult.Success -> {
                        Glint.LOGGER.info("Synced scene: ${scene.id}")
                    }
                    is SyncResult.Failure -> {
                        Glint.LOGGER.error("Failed to sync scene: ${result.userMessage}")
                    }
                }
            }
    }

    private fun disableScene(
        scene: Scene,
        world: WorldEntry,
    ) {
        val fileName = world.collectionFileName ?: return
        minecraft?.setScreen(ConfirmDisableScreen(this, fileName, scene.id, scene.name))
    }

    private fun resolveScene(
        scene: Scene,
        collection: SceneCollection,
        collectionFileName: String,
    ): ResolvedScene {
        val mergedConfig =
            (scene.config ?: SceneConfig())
                .mergeWith(collection.defaultConfig)
                .mergeWith(SceneConfig.DEFAULT)

        return ResolvedScene(
            scene = scene,
            collection = collection,
            config = mergedConfig,
            collectionFileName = collectionFileName,
        )
    }
}
