package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.GlintApi
import com.xevion.glint.api.WorldInfo
import net.minecraft.client.gui.GuiGraphics
import net.minecraft.client.gui.components.Button
import net.minecraft.client.gui.components.ObjectSelectionList
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import net.minecraft.network.chat.Component
import java.util.concurrent.CompletableFuture

class WorldSelectionScreen(
    private val parent: Screen,
    private val serverUrl: String,
    private val onBack: () -> Unit,
) : Screen(Component.literal("Select World")) {
    private var worldList: WorldListWidget? = null
    private lateinit var saveButton: Button
    private lateinit var backButton: Button

    private var loadingWorlds = true
    private var worlds: List<WorldInfo> = emptyList()
    private var loadError: String? = null

    override fun init() {
        val centerX = width / 2
        val bottomY = height - 40
        val topY = 55
        val listHeight = bottomY - topY - 10

        // World list widget (fits between title/connection info and buttons)
        // Constructor: (Minecraft, width, height, top_y, itemHeight)
        // Item height: 36px (2 lines of text @ 9px + padding)
        worldList =
            WorldListWidget(
                minecraft!!,
                width,
                listHeight,
                topY,
                36,
            )
        addRenderableWidget(worldList)

        // Save & Enable button
        saveButton =
            Button
                .builder(Component.literal("Save & Enable Sync")) { saveSelection() }
                .bounds(centerX - 155, bottomY, 150, 20)
                .build()
        saveButton.active = false
        addRenderableWidget(saveButton)

        // Back button
        backButton =
            Button
                .builder(CommonComponents.GUI_BACK) { onBack() }
                .bounds(centerX + 5, bottomY, 150, 20)
                .build()
        addRenderableWidget(backButton)

        // Load worlds from server
        loadWorlds()
    }

    private fun loadWorlds() {
        loadingWorlds = true
        loadError = null

        CompletableFuture
            .supplyAsync {
                GlintApi.listWorlds(serverUrl)
            }.thenAccept { result ->
                minecraft?.execute {
                    loadingWorlds = false
                    result
                        .onSuccess { fetchedWorlds ->
                            worlds = fetchedWorlds
                            if (worlds.isEmpty()) {
                                loadError = "No worlds available. Upload a world to the server first."
                            } else {
                                worldList?.setWorlds(worlds)
                            }
                            Glint.LOGGER.info("Loaded {} worlds from server", worlds.size)
                        }.onFailure { error ->
                            loadError = error.message ?: "Failed to load worlds"
                            Glint.LOGGER.error("Failed to load worlds: {}", error.message)
                        }
                }
            }
    }

    private fun saveSelection() {
        val selected = worldList?.selected ?: return

        val config =
            ApiConfig(
                apiUrl = serverUrl,
                worldId = selected.id,
                worldName = selected.name,
                enabled = true,
                validated = true,
            )

        if (ApiConfig.save(config)) {
            Glint.LOGGER.info("API config saved: connected to {} with world '{}'", serverUrl, selected.name)
            minecraft?.setScreen(parent)
        } else {
            Glint.LOGGER.error("Failed to save API config")
        }
    }

    override fun render(
        guiGraphics: GuiGraphics,
        mouseX: Int,
        mouseY: Int,
        delta: Float,
    ) {
        renderBackground(guiGraphics, mouseX, mouseY, delta)

        // Render widgets (includes world list)
        super.render(guiGraphics, mouseX, mouseY, delta)

        // Render text overlays on top
        val centerX = width / 2

        guiGraphics.drawCenteredString(font, title, width / 2, 20, 0xFFFFFF)

        // Connection info at top
        guiGraphics.drawCenteredString(
            font,
            "Connected to: $serverUrl",
            centerX,
            35,
            0x888888,
        )

        // Status messages (rendered on top of everything)
        if (loadingWorlds) {
            guiGraphics.drawCenteredString(
                font,
                "Loading worlds from server...",
                centerX,
                height / 2,
                0xFFAA00,
            )
        } else if (loadError != null) {
            guiGraphics.drawCenteredString(
                font,
                "❌ $loadError",
                centerX,
                height / 2,
                0xFF0000,
            )

            // Show skip option
            guiGraphics.drawCenteredString(
                font,
                "Click Back to change server or skip to disable sync",
                centerX,
                height / 2 + 20,
                0x888888,
            )
        } else if (worlds.isEmpty()) {
            guiGraphics.drawCenteredString(
                font,
                "No worlds available",
                centerX,
                height / 2,
                0x888888,
            )
        }
    }

    inner class WorldListWidget(
        minecraft: net.minecraft.client.Minecraft,
        width: Int,
        height: Int,
        top: Int,
        itemHeight: Int,
    ) : ObjectSelectionList<WorldSelectionScreen.WorldListWidget.WorldEntry>(
        minecraft,
        width,
        height,
        top,
        itemHeight,
    ) {
        var selected: WorldInfo? = null
            private set

        fun setWorlds(worlds: List<WorldInfo>) {
            clearEntries()
            worlds.forEach { world ->
                addEntry(WorldEntry(world))
            }
        }

        override fun setSelected(entry: WorldEntry?) {
            super.setSelected(entry)
            selected = entry?.world
            saveButton.active = entry != null
        }

        override fun mouseScrolled(
            mouseX: Double,
            mouseY: Double,
            scrollX: Double,
            scrollY: Double,
        ): Boolean {
            // Reduce scroll sensitivity by halving the scroll amount
            return super.mouseScrolled(mouseX, mouseY, scrollX, scrollY * 0.5)
        }

        inner class WorldEntry(
            val world: WorldInfo,
        ) : Entry<WorldSelectionScreen.WorldListWidget.WorldEntry>() {
            override fun render(
                guiGraphics: GuiGraphics,
                index: Int,
                top: Int,
                left: Int,
                width: Int,
                height: Int,
                mouseX: Int,
                mouseY: Int,
                hovered: Boolean,
                partialTick: Float,
            ) {
                val minecraft = this@WorldSelectionScreen.minecraft ?: return
                val font = minecraft.font

                val padding = 8
                val textPadding = 4
                val maxWidth = width - (padding * 2)

                // World name - truncate if too long
                val nameText = truncateText(font, world.name, maxWidth)
                guiGraphics.drawString(
                    font,
                    nameText,
                    left + padding,
                    top + textPadding,
                    0xFFFFFF,
                )

                // Description (MC version + slug) - truncate if needed
                val description =
                    world.description?.takeIf { it.isNotBlank() }
                        ?: "MC ${world.minecraftVersion} • ${world.slug}"

                val descText = truncateText(font, description, maxWidth)
                guiGraphics.drawString(
                    font,
                    descText,
                    left + padding,
                    top + textPadding + 11,
                    0x888888,
                )
            }

            private fun truncateText(
                font: net.minecraft.client.gui.Font,
                text: String,
                maxWidth: Int,
            ): String {
                if (font.width(text) <= maxWidth) return text

                val ellipsis = "..."
                val ellipsisWidth = font.width(ellipsis)

                var truncated = text
                while (font.width(truncated) + ellipsisWidth > maxWidth && truncated.isNotEmpty()) {
                    truncated = truncated.dropLast(1)
                }

                return truncated + ellipsis
            }

            override fun getNarration(): Component = Component.literal(world.name)
        }
    }
}
