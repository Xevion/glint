package com.xevion.glint.ui

import com.xevion.glint.scene.ResolvedScene
import com.xevion.glint.scene.Scene
import com.xevion.glint.scene.SceneCollection
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.scene.SceneVariant
import net.minecraft.client.Minecraft
import net.minecraft.client.gui.GuiGraphics
import net.minecraft.client.gui.components.Button
import net.minecraft.client.gui.components.ContainerObjectSelectionList
import net.minecraft.client.gui.layouts.HeaderAndFooterLayout
import net.minecraft.client.gui.narration.NarratableEntry
import net.minecraft.client.gui.navigation.CommonInputs
import net.minecraft.network.chat.Component

class SceneList(
    minecraft: Minecraft,
    private val screen: SceneManagerScreen,
) : ContainerObjectSelectionList<SceneList.Entry>(
        minecraft,
        screen.width,
        screen.height - 64,
        32,
        ITEM_HEIGHT,
    ) {
    companion object {
        const val ITEM_HEIGHT = 36
    }

    fun updateListSize(
        width: Int,
        layout: HeaderAndFooterLayout,
    ) {
        setSize(width, layout.contentHeight)
    }

    fun refreshEntries(collections: List<Pair<String, SceneCollection>>) {
        clearEntries()

        if (collections.isEmpty()) {
            return
        }

        for ((fileName, collection) in collections) {
            addEntry(WorldEntry(collection.world, fileName, collection.scenes.size))

            if (screen.isWorldExpanded(fileName)) {
                for (scene in collection.scenes) {
                    addEntry(SceneEntry(scene, collection, fileName))

                    if (screen.isSceneExpanded(scene.id) && scene.variants.isNotEmpty()) {
                        for (variant in scene.variants) {
                            addEntry(VariantEntry(scene, variant, collection, fileName))
                        }
                    }
                }
            }
        }
    }

    abstract class Entry : ContainerObjectSelectionList.Entry<Entry>()

    inner class WorldEntry(
        private val worldName: String,
        private val fileName: String,
        private val sceneCount: Int,
    ) : Entry() {
        private val expandButton =
            Button
                .builder(Component.literal(if (screen.isWorldExpanded(fileName)) "▼" else "▶")) {
                    screen.toggleWorldExpanded(fileName)
                }.width(20)
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
            guiGraphics.drawString(minecraft.font, worldName, textX, textY, 0xFFFFFF)

            val countText = "($sceneCount scenes)"
            val countX = textX + minecraft.font.width(worldName) + 8
            guiGraphics.drawString(minecraft.font, countText, countX, textY, 0xAAAAAA)
        }

        override fun children() = listOf(expandButton)

        override fun narratables() = listOf(expandButton)
    }

    inner class SceneEntry(
        private val scene: Scene,
        private val collection: SceneCollection,
        private val fileName: String,
    ) : Entry() {
        private val teleportButton =
            Button
                .builder(Component.literal("Go")) {
                    val resolved = resolveScene(scene, collection, fileName)
                    screen.teleportToScene(resolved)
                }.width(40)
                .build()

        private val captureButton =
            Button
                .builder(Component.literal("Capture")) {
                    screen.startCaptureScene(scene.id)
                }.width(50)
                .build()

        private val deleteButton =
            Button
                .builder(Component.literal("X")) {
                    screen.confirmDeleteScene(fileName, scene.id, scene.name)
                }.width(20)
                .build()

        private val expandButton: Button? =
            if (scene.variants.isNotEmpty()) {
                Button
                    .builder(Component.literal(if (screen.isSceneExpanded(scene.id)) "▼" else "▶")) {
                        screen.toggleSceneExpanded(scene.id)
                    }.width(16)
                    .build()
            } else {
                null
            }

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
            val indent = 20
            var x = left + indent

            expandButton?.let {
                it.setPosition(x, top + 8)
                it.render(guiGraphics, mouseX, mouseY, delta)
                x += 20
            } ?: run { x += 4 }

            val nameY = top + 4
            guiGraphics.drawString(minecraft.font, scene.name, x, nameY, 0xFFFFFF)

            val detailsY = top + 16
            val detailsText = formatSceneDetails(scene)
            guiGraphics.drawString(minecraft.font, detailsText, x, detailsY, 0x888888)

            val buttonY = top + 8
            deleteButton.setPosition(left + width - 24, buttonY)
            deleteButton.render(guiGraphics, mouseX, mouseY, delta)

            captureButton.setPosition(left + width - 78, buttonY)
            captureButton.render(guiGraphics, mouseX, mouseY, delta)

            teleportButton.setPosition(left + width - 122, buttonY)
            teleportButton.render(guiGraphics, mouseX, mouseY, delta)
        }

        override fun children() = listOfNotNull(expandButton, teleportButton, captureButton, deleteButton)

        override fun narratables() = listOfNotNull(expandButton, teleportButton, captureButton, deleteButton)

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
    }

    inner class VariantEntry(
        private val baseScene: Scene,
        private val variant: SceneVariant,
        private val collection: SceneCollection,
        private val fileName: String,
    ) : Entry() {
        private val teleportButton =
            Button
                .builder(Component.literal("Go")) {
                    val expandedScene = variant.applyTo(baseScene)
                    val resolved = resolveScene(expandedScene, collection, fileName)
                    screen.teleportToScene(resolved)
                }.width(40)
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
            val indent = 40
            val textY = top + 12

            guiGraphics.drawString(minecraft.font, "  ↳ ${variant.name}", left + indent, textY, 0xCCCCCC)

            teleportButton.setPosition(left + width - 44, top + 8)
            teleportButton.render(guiGraphics, mouseX, mouseY, delta)
        }

        override fun children() = listOf(teleportButton)

        override fun narratables() = listOf(teleportButton)
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
