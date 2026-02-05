package com.xevion.glint.ui

import com.xevion.glint.Loggers
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.SceneSyncManager
import com.xevion.glint.api.SyncResult
import com.xevion.glint.error
import com.xevion.glint.info
import com.xevion.glint.orchestration.CaptureSpec
import com.xevion.glint.orchestration.ShaderSpec
import com.xevion.glint.scene.ResolvedScene
import com.xevion.glint.scene.Scene
import com.xevion.glint.scene.SceneApplicator
import com.xevion.glint.scene.SceneCollection
import com.xevion.glint.scene.SceneConfig
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.scene.SceneVariant
import com.xevion.glint.session.SessionRegistry
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintListScreen
import com.xevion.glint.ui.base.GlintTheme
import com.xevion.glint.warn
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.Sizing
import net.minecraft.client.gui.components.toasts.SystemToast
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import net.minecraft.network.chat.Component as McComponent

/**
 * Scene manager screen for managing and navigating scenes.
 * Shows a tree of worlds > scenes > variants with action buttons.
 */
class SceneManagerScreen(
    private val lastScreen: Screen?,
) : GlintListScreen(McComponent.literal("Glint Menu")) {
    private var collections: List<Pair<String, SceneCollection>> = emptyList()
    private val expandedWorlds = mutableSetOf<String>()
    private val expandedScenes = mutableSetOf<String>()

    companion object {
        private val log = Loggers.Ui.get()
    }

    override fun buildHeader(header: FlowLayout) {
        header.child(
            GlintComponents.button(McComponent.literal("Capture All")) { startOrchestration() } as Component,
        )
        header.child(
            GlintComponents.button(McComponent.literal("Save Scene")) { openSaveSceneDialog() } as Component,
        )
        header.child(
            GlintComponents.button(McComponent.literal("Sync All")) { syncAllScenes() } as Component,
        )
        header.child(
            GlintComponents.button(McComponent.literal("API Config")) { openApiConfig() } as Component,
        )
    }

    override fun buildFooter(footer: FlowLayout) {
        footer.child(
            GlintComponents.button(CommonComponents.GUI_DONE) { onClose() } as Component,
        )
    }

    override fun buildContent(content: FlowLayout) {
        collections = SceneManager.discoverAllCollections()

        if (collections.isEmpty()) {
            content.child(
                Components
                    .label(McComponent.literal("No scene collections found"))
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
            )
            return
        }

        for ((fileName, collection) in collections) {
            buildWorldEntry(content, fileName, collection)

            if (expandedWorlds.contains(fileName)) {
                for (scene in collection.scenes) {
                    buildSceneEntry(content, scene, collection, fileName)

                    if (expandedScenes.contains(scene.id) && scene.variants.isNotEmpty()) {
                        for (variant in scene.variants) {
                            buildVariantEntry(content, scene, variant, collection, fileName)
                        }
                    }
                }
            }
        }
    }

    private fun buildWorldEntry(
        content: FlowLayout,
        fileName: String,
        collection: SceneCollection,
    ) {
        val isExpanded = expandedWorlds.contains(fileName)
        val row =
            GlintComponents.listItemWithButtons(
                indent = GlintTheme.INDENT_LEVEL_1,
                contentBuilder = {
                    child(GlintComponents.expandToggle(isExpanded) { toggleWorldExpanded(fileName) } as Component)
                    child(GlintComponents.itemLabel(collection.world) as Component)
                    child(GlintComponents.itemDetail("(${collection.scenes.size} scenes)") as Component)
                },
            )
        content.child(row as Component)
    }

    private fun buildSceneEntry(
        content: FlowLayout,
        scene: Scene,
        collection: SceneCollection,
        fileName: String,
    ) {
        val hasVariants = scene.variants.isNotEmpty()
        val isExpanded = expandedScenes.contains(scene.id)

        val teleportBtn =
            GlintComponents.smallButton(
                McComponent.literal("Go"),
                40,
                McComponent.literal("Teleport to scene"),
            ) {
                val resolved = resolveScene(scene, collection, fileName)
                teleportToScene(resolved)
            }

        val captureBtn =
            GlintComponents.smallButton(
                McComponent.literal("Capture"),
                55,
                McComponent.literal("Test capture for this scene"),
            ) {
                startCaptureScene(scene.id)
            }

        val deleteBtn =
            GlintComponents.iconButton("X", McComponent.literal("Disable scene")) {
                confirmDeleteScene(fileName, scene.id, scene.name)
            }

        val row =
            GlintComponents.listItemWithButtons(
                indent = GlintTheme.INDENT_LEVEL_2,
                contentBuilder = {
                    if (hasVariants) {
                        child(GlintComponents.expandToggle(isExpanded) { toggleSceneExpanded(scene.id) } as Component)
                    } else {
                        // Spacer for alignment
                        child(Containers.horizontalFlow(Sizing.fixed(24), Sizing.fixed(1)) as Component)
                    }

                    val textContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
                    textContainer.gap(2)
                    textContainer.child(GlintComponents.itemLabel(scene.name) as Component)
                    textContainer.child(GlintComponents.itemDetail(formatSceneDetails(scene)) as Component)
                    child(textContainer as Component)
                },
                buttons = arrayOf(teleportBtn, captureBtn, deleteBtn),
            )
        content.child(row as Component)
    }

    private fun buildVariantEntry(
        content: FlowLayout,
        baseScene: Scene,
        variant: SceneVariant,
        collection: SceneCollection,
        fileName: String,
    ) {
        val teleportBtn =
            GlintComponents.smallButton(
                McComponent.literal("Go"),
                40,
                McComponent.literal("Teleport to variant"),
            ) {
                val expandedScene = variant.applyTo(baseScene)
                val resolved = resolveScene(expandedScene, collection, fileName)
                teleportToScene(resolved)
            }

        val row =
            GlintComponents.listItemWithButtons(
                indent = GlintTheme.INDENT_LEVEL_3,
                height = GlintTheme.ITEM_HEIGHT_COMPACT,
                contentBuilder = {
                    child(GlintComponents.itemLabel("> ${variant.name}", GlintTheme.TEXT_SECONDARY) as Component)
                },
                buttons = arrayOf(teleportBtn),
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

    override fun onClose() {
        minecraft?.setScreen(lastScreen)
    }

    fun refreshSceneList() {
        refreshContent()
    }

    fun toggleWorldExpanded(fileName: String) {
        if (expandedWorlds.contains(fileName)) {
            expandedWorlds.remove(fileName)
        } else {
            expandedWorlds.add(fileName)
        }
        refreshContent()
    }

    fun toggleSceneExpanded(sceneId: String) {
        if (expandedScenes.contains(sceneId)) {
            expandedScenes.remove(sceneId)
        } else {
            expandedScenes.add(sceneId)
        }
        refreshContent()
    }

    fun isWorldExpanded(fileName: String): Boolean = expandedWorlds.contains(fileName)

    fun isSceneExpanded(sceneId: String): Boolean = expandedScenes.contains(sceneId)

    fun teleportToScene(resolvedScene: ResolvedScene) {
        log.info("Teleporting to scene") {
            "scene_id" to resolvedScene.scene.id
        }
        SceneApplicator.apply(resolvedScene)
        onClose()
    }

    fun confirmDeleteScene(
        worldName: String,
        sceneId: String,
        sceneName: String,
    ) {
        minecraft?.setScreen(ConfirmDisableScreen(this, worldName, sceneId, sceneName))
    }

    fun executeDeleteScene(
        worldName: String,
        sceneId: String,
    ) {
        SceneManager
            .disableScene(worldName, sceneId)
            .thenAccept { success ->
                minecraft?.execute {
                    if (success) {
                        log.info("Disabled scene") {
                            "scene_id" to sceneId
                            "world" to worldName
                        }
                        refreshContent()
                    } else {
                        log.error("Failed to disable scene") {
                            "scene_id" to sceneId
                        }
                    }
                }
            }
    }

    fun startCaptureScene(sceneId: String) {
        val spec =
            CaptureSpec(
                sceneIds = listOf(sceneId),
                shaders = listOf(ShaderSpec(filename = null)),
            )
        if (SessionRegistry.startOrchestration(spec)) {
            log.info("Starting capture for scene") {
                "scene_id" to sceneId
            }
            onClose()
        } else {
            log.error("Failed to start capture for scene") {
                "scene_id" to sceneId
            }
        }
    }

    private fun openSaveSceneDialog() {
        minecraft?.setScreen(SaveSceneDialog(this))
    }

    private fun startOrchestration() {
        val allSceneIds =
            collections.flatMap { (_, collection) ->
                collection.scenes.flatMap { scene ->
                    listOf(scene.id) + scene.variants.map { it.id }
                }
            }

        if (allSceneIds.isEmpty()) {
            log.warn("No scenes to capture")
            return
        }

        val spec =
            CaptureSpec(
                sceneIds = allSceneIds,
                shaders = listOf(ShaderSpec(filename = null)),
            )

        if (SessionRegistry.startOrchestration(spec)) {
            log.info("Starting orchestration from menu")
            onClose()
        } else {
            log.error("Failed to start orchestration")
        }
    }

    private fun openApiConfig() {
        val config = ApiConfig.load()
        val showConnectionFirst = config.needsValidation()
        minecraft?.setScreen(ApiConfigWizardScreen(this, showConnectionFirst))
    }

    private fun syncAllScenes() {
        val config = ApiConfig.load()

        if (!config.isValid()) {
            log.warn("API config not valid - open API Config to set it up")
            SystemToast.add(
                minecraft!!.toastManager,
                SystemToast.SystemToastId.WORLD_ACCESS_FAILURE,
                McComponent.literal("Cannot sync scenes"),
                McComponent.literal("API config not set up"),
            )
            return
        }

        val allCollections = SceneManager.discoverAllCollections()
        var totalScenes = 0

        for ((_, collection) in allCollections) {
            totalScenes += collection.scenes.size
            SceneSyncManager
                .syncCollection(collection, config)
                .thenAccept { results ->
                    val successes = results.count { it is SyncResult.Success }
                    val failures = results.count { it is SyncResult.Failure }

                    if (failures > 0) {
                        val failureResults = results.filterIsInstance<SyncResult.Failure>()
                        val firstError = failureResults.firstOrNull()?.userMessage ?: "Unknown error"

                        SystemToast.addOrUpdate(
                            minecraft!!.toastManager,
                            SystemToast.SystemToastId.PERIODIC_NOTIFICATION,
                            McComponent.literal("${collection.world}: $successes/$totalScenes synced"),
                            McComponent.literal(firstError),
                        )
                    } else {
                        SystemToast.addOrUpdate(
                            minecraft!!.toastManager,
                            SystemToast.SystemToastId.PERIODIC_NOTIFICATION,
                            McComponent.literal("${collection.world} synced"),
                            McComponent.literal("$successes scenes updated"),
                        )
                    }
                }.exceptionally { e ->
                    log.error(e, "Failed to sync scenes") {
                        "world" to collection.world
                    }
                    SystemToast.add(
                        minecraft!!.toastManager,
                        SystemToast.SystemToastId.WORLD_ACCESS_FAILURE,
                        McComponent.literal("Sync failed: ${collection.world}"),
                        McComponent.literal(e.message ?: "Unknown error"),
                    )
                    null
                }
        }

        log.info("Started sync") {
            "scene_count" to totalScenes
            "world_count" to allCollections.size
        }
        SystemToast.add(
            minecraft!!.toastManager,
            SystemToast.SystemToastId.PERIODIC_NOTIFICATION,
            McComponent.literal("Syncing scenes..."),
            McComponent.literal("$totalScenes scenes across ${allCollections.size} worlds"),
        )
    }
}
