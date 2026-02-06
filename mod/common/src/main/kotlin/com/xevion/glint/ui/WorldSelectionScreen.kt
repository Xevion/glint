package com.xevion.glint.ui

import com.xevion.glint.Loggers
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.GlintApi
import com.xevion.glint.api.WorldInfo
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.ButtonComponent
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.component.LabelComponent
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.container.ScrollContainer
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.Surface
import net.minecraft.client.gui.screens.Screen
import java.util.concurrent.CompletableFuture
import net.minecraft.network.chat.Component as McComponent

/**
 * World selection screen for the device auth flow.
 * Displays available worlds from the API and allows the user to select one.
 */
class WorldSelectionScreen(
    private val parent: Screen,
    private val serverUrl: String,
    private val accessToken: String,
    private val tokenExpiresIn: Long = 0L,
    private val onBack: () -> Unit,
) : GlintScreen(McComponent.literal("Select World")) {
    private lateinit var contentContainer: FlowLayout
    private lateinit var worldListScroll: ScrollContainer<FlowLayout>
    private lateinit var worldListContent: FlowLayout
    private lateinit var statusLabel: LabelComponent
    private lateinit var saveButton: ButtonComponent
    private lateinit var backButton: ButtonComponent

    private var loadingWorlds = true
    private var worlds: List<WorldInfo> = emptyList()
    private var loadError: String? = null
    private var selectedWorld: WorldInfo? = null

    override fun buildContent(root: FlowLayout) {
        contentContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
        contentContainer.horizontalAlignment(HorizontalAlignment.CENTER)
        contentContainer.gap(GlintTheme.GAP_MD)
        contentContainer.padding(GlintTheme.paddingLg())
        contentContainer.surface(Surface.DARK_PANEL)

        // Title
        contentContainer.child(GlintComponents.title(title) as Component)

        // Connection info subtitle
        contentContainer.child(
            GlintComponents.subtitle(McComponent.literal("Connected to: $serverUrl")) as Component,
        )

        // Status label (loading/error)
        statusLabel = Components.label(McComponent.literal("Loading worlds from server..."))
        statusLabel.color(Color.ofRgb(GlintTheme.TEXT_WARNING))
        statusLabel.horizontalTextAlignment(HorizontalAlignment.CENTER)
        contentContainer.child(statusLabel as Component)

        // World list (scrollable)
        worldListContent = Containers.verticalFlow(Sizing.fill(100), Sizing.content())
        worldListContent.gap(GlintTheme.GAP_SM)

        worldListScroll =
            Containers.verticalScroll(
                Sizing.fixed(350),
                Sizing.fixed(200),
                worldListContent,
            )
        worldListScroll.surface(Surface.flat(0x44000000))
        contentContainer.child(worldListScroll as Component)

        // Button row
        saveButton = GlintComponents.wideButton(McComponent.literal("Save & Enable Sync")) { saveSelection() }
        saveButton.active = false

        backButton = GlintComponents.wideButton(McComponent.literal("Back")) { onBack() }

        contentContainer.child(
            GlintComponents.buttonRow(saveButton, backButton) as Component,
        )

        root.child(contentContainer as Component)

        // Load worlds
        loadWorlds()
    }

    private fun loadWorlds() {
        loadingWorlds = true
        loadError = null
        statusLabel.text(McComponent.literal("Loading worlds from server..."))
        statusLabel.color(Color.ofRgb(GlintTheme.TEXT_WARNING))

        CompletableFuture
            .supplyAsync {
                GlintApi.listWorlds(serverUrl, accessToken)
            }.thenAccept { result ->
                minecraft?.execute {
                    loadingWorlds = false
                    result
                        .onSuccess { fetchedWorlds ->
                            worlds = fetchedWorlds
                            if (worlds.isEmpty()) {
                                loadError = "No worlds available. Upload a world to the server first."
                                statusLabel.text(McComponent.literal(loadError!!))
                                statusLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
                            } else {
                                statusLabel.text(McComponent.literal("Select a world:"))
                                statusLabel.color(Color.ofRgb(GlintTheme.TEXT_SECONDARY))
                                rebuildWorldList()
                            }
                            Loggers.Ui.get().info("Loaded {} worlds from server", worlds.size)
                        }.onFailure { error ->
                            loadError = error.message ?: "Failed to load worlds"
                            statusLabel.text(McComponent.literal("Error: $loadError"))
                            statusLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
                            Loggers.Ui.get().error("Failed to load worlds: {}", error.message)
                        }
                }
            }
    }

    private fun rebuildWorldList() {
        worldListContent.clearChildren()

        for (world in worlds) {
            val isSelected = selectedWorld?.id == world.id
            val row =
                GlintComponents.selectableRow(
                    indent = GlintTheme.PADDING_SM,
                    isSelected = isSelected,
                    onSelect = { selectWorld(world) },
                ) {
                    val textContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
                    textContainer.gap(2)

                    // World name
                    textContainer.child(GlintComponents.itemLabel(world.name) as Component)

                    // Description (MC version + slug)
                    val description =
                        world.description?.takeIf { it.isNotBlank() }
                            ?: "MC ${world.minecraftVersion} - ${world.slug}"
                    textContainer.child(GlintComponents.itemDetail(description) as Component)

                    child(textContainer as Component)
                }

            worldListContent.child(row as Component)
        }
    }

    private fun selectWorld(world: WorldInfo) {
        selectedWorld = world
        saveButton.active = true
        rebuildWorldList()
    }

    private fun saveSelection() {
        val selected = selectedWorld ?: return

        // Calculate token expiry time (current time + expires_in seconds)
        val tokenExpiresAt =
            if (accessToken.isNotBlank() && tokenExpiresIn > 0) {
                System.currentTimeMillis() + (tokenExpiresIn * 1000)
            } else {
                0L
            }

        val config =
            ApiConfig(
                apiUrl = serverUrl,
                worldId = selected.id,
                worldName = selected.name,
                enabled = true,
                validated = true,
                accessToken = accessToken,
                tokenExpiresAt = tokenExpiresAt,
            )

        if (ApiConfig.save(config)) {
            Loggers.Ui.get().info("API config saved: connected to {} with world '{}'", serverUrl, selected.name)
            minecraft?.setScreen(parent)
        } else {
            Loggers.Ui.get().error("Failed to save API config")
        }
    }
}
