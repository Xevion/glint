package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.SceneSyncManager
import com.xevion.glint.scene.ResolvedScene
import com.xevion.glint.scene.SceneApplicator
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.session.SessionRegistry
import net.minecraft.client.gui.components.Button
import net.minecraft.client.gui.components.toasts.SystemToast
import net.minecraft.client.gui.layouts.HeaderAndFooterLayout
import net.minecraft.client.gui.layouts.LinearLayout
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import net.minecraft.network.chat.Component

class SceneManagerScreen(
    private val lastScreen: Screen?,
) : Screen(Component.literal("Glint Menu")) {
    private val layout = HeaderAndFooterLayout(this)
    private lateinit var sceneList: SceneList

    private val expandedWorlds = mutableSetOf<String>()
    private val expandedScenes = mutableSetOf<String>()

    override fun init() {
        addActionButtons()
        addContents()
        addFooter()
        layout.visitWidgets { addRenderableWidget(it) }
        repositionElements()
    }

    private fun addActionButtons() {
        val buttonRow = LinearLayout.horizontal().spacing(8)

        buttonRow.addChild(
            Button
                .builder(Component.literal("Capture All")) { startOrchestration() }
                .width(100)
                .build(),
        )

        buttonRow.addChild(
            Button
                .builder(Component.literal("Save Scene")) { openSaveSceneDialog() }
                .width(100)
                .build(),
        )

        buttonRow.addChild(
            Button
                .builder(Component.literal("Sync All")) { syncAllScenes() }
                .width(100)
                .build(),
        )

        buttonRow.addChild(
            Button
                .builder(Component.literal("API Config")) { openApiConfig() }
                .width(100)
                .build(),
        )

        layout.addToHeader(buttonRow)
    }

    private fun addContents() {
        sceneList = SceneList(minecraft!!, this)
        layout.addToContents(sceneList)
        refreshSceneList()
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
        sceneList.updateListSize(width, layout)
    }

    override fun onClose() {
        minecraft?.setScreen(lastScreen)
    }

    fun refreshSceneList() {
        val collections = SceneManager.discoverAllCollections()
        sceneList.refreshEntries(collections)
    }

    fun teleportToScene(resolvedScene: ResolvedScene) {
        Glint.LOGGER.info("Teleporting to scene: ${resolvedScene.scene.id}")
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
                        Glint.LOGGER.info("Disabled scene: $sceneId from world: $worldName")
                        refreshSceneList()
                    } else {
                        Glint.LOGGER.error("Failed to disable scene: $sceneId")
                    }
                }
            }
    }

    fun startCaptureScene(sceneId: String) {
        if (SessionRegistry.startCaptureSession(sceneId = sceneId)) {
            Glint.LOGGER.info("Starting capture for scene: $sceneId")
            onClose()
        } else {
            Glint.LOGGER.error("Failed to start capture for scene: $sceneId")
        }
    }

    private fun openSaveSceneDialog() {
        minecraft?.setScreen(SaveSceneDialog(this))
    }

    private fun startOrchestration() {
        if (SessionRegistry.startOrchestration()) {
            Glint.LOGGER.info("Starting orchestration from menu")
            onClose()
        } else {
            Glint.LOGGER.error("Failed to start orchestration")
        }
    }

    fun toggleWorldExpanded(fileName: String) {
        if (expandedWorlds.contains(fileName)) {
            expandedWorlds.remove(fileName)
        } else {
            expandedWorlds.add(fileName)
        }
        refreshSceneList()
    }

    fun toggleSceneExpanded(sceneId: String) {
        if (expandedScenes.contains(sceneId)) {
            expandedScenes.remove(sceneId)
        } else {
            expandedScenes.add(sceneId)
        }
        refreshSceneList()
    }

    fun isWorldExpanded(fileName: String): Boolean = expandedWorlds.contains(fileName)

    fun isSceneExpanded(sceneId: String): Boolean = expandedScenes.contains(sceneId)

    private fun openApiConfig() {
        val config = ApiConfig.load()
        val showConnectionFirst = config.needsValidation()
        minecraft?.setScreen(ApiConfigWizardScreen(this, showConnectionFirst))
    }

    private fun syncAllScenes() {
        val config = ApiConfig.load()

        if (!config.isValid()) {
            Glint.LOGGER.warn("API config not valid - open API Config to set it up")
            SystemToast.add(
                minecraft!!.toastManager,
                SystemToast.SystemToastId.WORLD_ACCESS_FAILURE,
                Component.literal("Cannot sync scenes"),
                Component.literal("API config not set up"),
            )
            return
        }

        val collections = SceneManager.discoverAllCollections()
        var totalScenes = 0

        for ((_, collection) in collections) {
            totalScenes += collection.scenes.size
            SceneSyncManager
                .syncCollection(collection, config)
                .thenAccept { results ->
                    val successes = results.count { it is com.xevion.glint.api.SyncResult.Success }
                    val failures = results.count { it is com.xevion.glint.api.SyncResult.Failure }

                    if (failures > 0) {
                        val failureResults = results.filterIsInstance<com.xevion.glint.api.SyncResult.Failure>()
                        val firstError = failureResults.firstOrNull()?.userMessage ?: "Unknown error"

                        SystemToast.addOrUpdate(
                            minecraft!!.toastManager,
                            SystemToast.SystemToastId.PERIODIC_NOTIFICATION,
                            Component.literal("${collection.world}: $successes/$totalScenes synced"),
                            Component.literal(firstError),
                        )
                    } else {
                        SystemToast.addOrUpdate(
                            minecraft!!.toastManager,
                            SystemToast.SystemToastId.PERIODIC_NOTIFICATION,
                            Component.literal("${collection.world} synced"),
                            Component.literal("$successes scenes updated"),
                        )
                    }
                }.exceptionally { e ->
                    Glint.LOGGER.error("Failed to sync scenes for ${collection.world}", e)
                    SystemToast.add(
                        minecraft!!.toastManager,
                        SystemToast.SystemToastId.WORLD_ACCESS_FAILURE,
                        Component.literal("Sync failed: ${collection.world}"),
                        Component.literal(e.message ?: "Unknown error"),
                    )
                    null
                }
        }

        Glint.LOGGER.info("Started sync for $totalScenes scenes across ${collections.size} worlds")
        SystemToast.add(
            minecraft!!.toastManager,
            SystemToast.SystemToastId.PERIODIC_NOTIFICATION,
            Component.literal("Syncing scenes..."),
            Component.literal("$totalScenes scenes across ${collections.size} worlds"),
        )
    }
}
