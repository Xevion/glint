package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.scene.ResolvedScene
import com.xevion.glint.scene.SceneApplicator
import com.xevion.glint.scene.SceneManager
import net.minecraft.client.gui.components.Button
import net.minecraft.client.gui.layouts.HeaderAndFooterLayout
import net.minecraft.client.gui.layouts.LinearLayout
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import net.minecraft.network.chat.Component

class SceneManagerScreen(private val lastScreen: Screen?) : Screen(Component.literal("Scene Manager")) {
    private val layout = HeaderAndFooterLayout(this)
    private lateinit var sceneList: SceneList

    private val expandedWorlds = mutableSetOf<String>()
    private val expandedScenes = mutableSetOf<String>()

    override fun init() {
        addTitle()
        addContents()
        addFooter()
        layout.visitWidgets { addRenderableWidget(it) }
        repositionElements()
    }

    private fun addTitle() {
        layout.addTitleHeader(title, font)
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
                .builder(Component.literal("Create Scene")) { createNewScene() }
                .width(100)
                .build(),
        )
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
        minecraft?.setScreen(ConfirmDeleteScreen(this, worldName, sceneId, sceneName))
    }

    fun executeDeleteScene(
        worldName: String,
        sceneId: String,
    ) {
        if (SceneManager.removeScene(worldName, sceneId)) {
            Glint.LOGGER.info("Deleted scene: $sceneId from world: $worldName")
            refreshSceneList()
        } else {
            Glint.LOGGER.error("Failed to delete scene: $sceneId")
        }
    }

    private fun createNewScene() {
        Glint.LOGGER.info("Create scene not yet implemented")
        // TODO: Implement scene creation dialog or use existing saveCurrentStateAsScene
    }

    fun toggleWorldExpanded(worldName: String) {
        if (expandedWorlds.contains(worldName)) {
            expandedWorlds.remove(worldName)
        } else {
            expandedWorlds.add(worldName)
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

    fun isWorldExpanded(worldName: String): Boolean = expandedWorlds.contains(worldName)

    fun isSceneExpanded(sceneId: String): Boolean = expandedScenes.contains(sceneId)
}
