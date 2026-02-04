package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.SceneSyncManager
import com.xevion.glint.api.WorldInfo
import com.xevion.glint.scene.ResolvedScene
import com.xevion.glint.scene.Scene
import com.xevion.glint.scene.SceneCollection
import com.xevion.glint.session.SessionRegistry
import net.minecraft.client.Minecraft
import net.minecraft.client.gui.GuiGraphics
import net.minecraft.client.gui.components.Button
import net.minecraft.client.gui.components.ContainerObjectSelectionList
import net.minecraft.client.gui.components.Tooltip
import net.minecraft.client.gui.layouts.HeaderAndFooterLayout
import net.minecraft.network.chat.Component

/**
 * Scrollable list of worlds with expandable scenes.
 * Shows worlds from API or local JSON files.
 */
class WorldListWidget(
    minecraft: Minecraft,
    private val screen: GlintHubScreen,
) : ContainerObjectSelectionList<WorldListWidget.Entry>(
        minecraft,
        screen.width,
        screen.height - 64,
        32,
        ITEM_HEIGHT,
    ) {
    companion object {
        const val ITEM_HEIGHT = 36
    }

    private val expandedWorlds = mutableSetOf<String>()
    private val expandedScenes = mutableSetOf<String>()

    fun updateListSize(
        width: Int,
        layout: HeaderAndFooterLayout,
    ) {
        setSize(width, layout.contentHeight)
    }

    fun refreshEntries(worlds: List<WorldEntry>) {
        clearEntries()

        if (worlds.isEmpty()) {
            return
        }

        for (world in worlds) {
            addEntry(WorldHeaderEntry(world))

            if (expandedWorlds.contains(world.id)) {
                for (scene in world.scenes) {
                    addEntry(SceneItemEntry(scene, world))
                }
            }
        }
    }

    fun toggleWorldExpanded(worldId: String) {
        if (expandedWorlds.contains(worldId)) {
            expandedWorlds.remove(worldId)
        } else {
            expandedWorlds.add(worldId)
        }
        screen.refreshWorlds()
    }

    fun isWorldExpanded(worldId: String): Boolean = expandedWorlds.contains(worldId)

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
                    scenes = emptyList(), // Scenes loaded separately from API
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

    abstract class Entry : ContainerObjectSelectionList.Entry<Entry>()

    /**
     * World header entry - shows world name and expand button.
     */
    inner class WorldHeaderEntry(
        private val world: WorldEntry,
    ) : Entry() {
        private val expandButton =
            Button
                .builder(Component.literal(if (isWorldExpanded(world.id)) "v" else ">")) {
                    toggleWorldExpanded(world.id)
                }.width(20)
                .build()

        private val openButton =
            Button
                .builder(Component.literal("[+]")) {
                    openWorld(world)
                }.width(20)
                .tooltip(Tooltip.create(Component.literal("Open world (download if needed)")))
                .build()

        private val settingsButton =
            Button
                .builder(Component.literal("[*]")) {
                    // TODO: World settings
                }.width(20)
                .tooltip(Tooltip.create(Component.literal("World settings")))
                .build()

        override fun render(
            guiGraphics: GuiGraphics,
            index: Int,
            top: Int,
            left: Int,
            width: Int,
            height: Int,
            mouseX: Int,
            mouseY: Int,
            hovering: Boolean,
            delta: Float,
        ) {
            val indent = 4

            expandButton.setPosition(left + indent, top + 8)
            expandButton.render(guiGraphics, mouseX, mouseY, delta)

            val textX = left + indent + 24
            val textY = top + 12
            guiGraphics.drawString(minecraft.font, world.name, textX, textY, 0xFFFFFF)

            val countText = "(${world.sceneCount} scenes)"
            val countX = textX + minecraft.font.width(world.name) + 8
            guiGraphics.drawString(minecraft.font, countText, countX, textY, 0xAAAAAA)

            // Right-aligned buttons
            settingsButton.setPosition(left + width - 24, top + 8)
            settingsButton.render(guiGraphics, mouseX, mouseY, delta)

            openButton.setPosition(left + width - 48, top + 8)
            openButton.render(guiGraphics, mouseX, mouseY, delta)
        }

        override fun children() = listOf(expandButton, openButton, settingsButton)

        override fun narratables() = listOf(expandButton, openButton, settingsButton)

        private fun openWorld(world: WorldEntry) {
            if (world.isLocal) {
                Glint.LOGGER.info("Opening local world: ${world.name}")
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
                        screen,
                        world.name,
                        com.xevion.glint.download.WorldDownloader.downloadWorld(
                            worldSlug = apiWorld.slug,
                            worldId = apiWorld.id,
                            fileUrl = fileUrl,
                            expectedHash = null,
                        ) { progress ->
                            minecraft.execute {
                                (minecraft.screen as? WorldDownloadDialog)?.updateProgress(progress)
                            }
                        },
                    )

                minecraft.setScreen(downloadDialog)
            }
        }
    }

    /**
     * Scene item entry - individual scene with action buttons.
     */
    inner class SceneItemEntry(
        private val scene: Scene,
        private val world: WorldEntry,
    ) : Entry() {
        private val teleportButton =
            Button
                .builder(Component.literal("[o]")) {
                    teleportToScene()
                }.width(20)
                .tooltip(Tooltip.create(Component.literal("Teleport to scene (must be in world)")))
                .build()
                .apply {
                    active = isInCorrectWorld()
                }

        private val captureButton =
            Button
                .builder(Component.literal("[C]")) {
                    startCapture()
                }.width(20)
                .tooltip(Tooltip.create(Component.literal("Test capture for this scene (must be in world)")))
                .build()

        private val syncButton =
            Button
                .builder(Component.literal("[^]")) {
                    syncScene()
                }.width(20)
                .tooltip(Tooltip.create(Component.literal("Sync scene to API")))
                .build()

        private val disableButton =
            Button
                .builder(Component.literal("[X]")) {
                    disableScene()
                }.width(20)
                .tooltip(Tooltip.create(Component.literal("Disable scene (marks inactive on API)")))
                .build()

        override fun render(
            guiGraphics: GuiGraphics,
            index: Int,
            top: Int,
            left: Int,
            width: Int,
            height: Int,
            mouseX: Int,
            mouseY: Int,
            hovering: Boolean,
            delta: Float,
        ) {
            val indent = 28
            val textX = left + indent
            val nameY = top + 4
            val detailsY = top + 16

            guiGraphics.drawString(minecraft.font, scene.name, textX, nameY, 0xFFFFFF)

            val detailsText = formatSceneDetails(scene)
            guiGraphics.drawString(minecraft.font, detailsText, textX, detailsY, 0x888888)

            // Right-aligned action buttons (spacing: 4px between)
            val buttonY = top + 8
            disableButton.setPosition(left + width - 24, buttonY)
            disableButton.render(guiGraphics, mouseX, mouseY, delta)

            syncButton.setPosition(left + width - 48, buttonY)
            syncButton.render(guiGraphics, mouseX, mouseY, delta)

            captureButton.setPosition(left + width - 72, buttonY)
            captureButton.render(guiGraphics, mouseX, mouseY, delta)

            teleportButton.setPosition(left + width - 96, buttonY)
            teleportButton.render(guiGraphics, mouseX, mouseY, delta)
        }

        override fun children() = listOf(teleportButton, captureButton, syncButton, disableButton)

        override fun narratables() = listOf(teleportButton, captureButton, syncButton, disableButton)

        private fun isInCorrectWorld(): Boolean {
            val mc = Minecraft.getInstance()
            if (mc.level == null || mc.singleplayerServer == null) return false

            val currentWorldName = mc.singleplayerServer?.worldData?.levelName
            return currentWorldName == world.name
        }

        private fun teleportToScene() {
            if (!isInCorrectWorld()) {
                Glint.LOGGER.warn("Cannot teleport - not in correct world")
                return
            }

            val collection = world.collection ?: return
            val fileName = world.collectionFileName ?: return

            val resolved = resolveScene(scene, collection, fileName)
            com.xevion.glint.scene.SceneApplicator
                .apply(resolved)
            screen.onClose()
        }

        private fun startCapture() {
            if (SessionRegistry.startCaptureSession(sceneId = scene.id)) {
                Glint.LOGGER.info("Starting capture for scene: ${scene.id}")
                screen.onClose()
            } else {
                Glint.LOGGER.error("Failed to start capture for scene: ${scene.id}")
            }
        }

        private fun syncScene() {
            val config = ApiConfig.load()
            if (!config.isValid()) {
                Glint.LOGGER.warn("Cannot sync - API not configured")
                return
            }

            SceneSyncManager
                .syncScene(scene, config)
                .thenAccept { result ->
                    when (result) {
                        is com.xevion.glint.api.SyncResult.Success -> {
                            Glint.LOGGER.info("Synced scene: ${scene.id}")
                        }

                        is com.xevion.glint.api.SyncResult.Failure -> {
                            Glint.LOGGER.error("Failed to sync scene: ${result.userMessage}")
                        }
                    }
                }
        }

        private fun disableScene() {
            val fileName = world.collectionFileName ?: return
            minecraft?.setScreen(
                ConfirmDisableScreen(screen, fileName, scene.id, scene.name),
            )
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
            val hour = ((ticks / 1000 + 6) % 24).toInt()
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
                (
                    scene.config ?: com.xevion.glint.scene
                        .SceneConfig()
                ).mergeWith(collection.defaultConfig)
                    .mergeWith(com.xevion.glint.scene.SceneConfig.DEFAULT)

            return ResolvedScene(
                scene = scene,
                collection = collection,
                config = mergedConfig,
                collectionFileName = collectionFileName,
            )
        }
    }
}
