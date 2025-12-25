package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.api.ApiConfig
import net.minecraft.client.gui.GuiGraphics
import net.minecraft.client.gui.components.Button
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import net.minecraft.network.chat.Component

/**
 * Entry point for API configuration.
 * Shows edit screen if already configured, or starts wizard flow if not.
 */
class ApiConfigWizardScreen(
    private val parent: Screen,
    private val showConnectionFirst: Boolean = false,
) : Screen(Component.literal("API Configuration")) {
    private val config: ApiConfig = ApiConfig.load()

    override fun init() {
        // If no config or forced to show connection first, start wizard flow
        if (showConnectionFirst || !config.validated) {
            showConnectionScreen()
        } else {
            // Show edit screen for existing valid config
            showEditScreen()
        }
    }

    private fun showConnectionScreen() {
        minecraft?.setScreen(
            ServerConnectionScreen(
                parent = parent,
                onConnectionValidated = { validatedUrl ->
                    showWorldSelection(validatedUrl)
                },
                initialUrl = config.apiUrl.ifBlank { "http://localhost:8080" },
            ),
        )
    }

    private fun showWorldSelection(serverUrl: String) {
        minecraft?.setScreen(
            WorldSelectionScreen(
                parent = parent,
                serverUrl = serverUrl,
                onBack = { showConnectionScreen() },
            ),
        )
    }

    private fun showEditScreen() {
        minecraft?.setScreen(ConfigEditScreen(parent, config))
    }

    override fun render(
        guiGraphics: GuiGraphics,
        mouseX: Int,
        mouseY: Int,
        delta: Float,
    ) {
        // This screen immediately forwards to another screen, so no rendering needed
        super.render(guiGraphics, mouseX, mouseY, delta)
    }

    /**
     * Edit screen for already-configured API connection.
     */
    private inner class ConfigEditScreen(
        private val parent: Screen,
        private val config: ApiConfig,
    ) : Screen(Component.literal("API Configuration")) {
        private lateinit var changeServerButton: Button
        private lateinit var disableButton: Button
        private lateinit var doneButton: Button

        override fun init() {
            val centerX = width / 2
            val startY = height / 2 - 40

            // Change Server button (starts wizard flow)
            changeServerButton =
                Button
                    .builder(Component.literal("Change Server")) { showConnectionScreen() }
                    .bounds(centerX - 155, startY + 60, 310, 20)
                    .build()
            addRenderableWidget(changeServerButton)

            // Disable Sync button
            disableButton =
                Button
                    .builder(Component.literal("Disable Sync")) { disableSync() }
                    .bounds(centerX - 155, startY + 85, 310, 20)
                    .build()
            addRenderableWidget(disableButton)

            // Done button
            doneButton =
                Button
                    .builder(CommonComponents.GUI_DONE) { minecraft?.setScreen(parent) }
                    .bounds(centerX - 75, startY + 120, 150, 20)
                    .build()
            addRenderableWidget(doneButton)
        }

        private fun disableSync() {
            val disabledConfig =
                ApiConfig(
                    apiUrl = "",
                    worldId = "",
                    worldName = "",
                    enabled = false,
                    validated = false,
                )
            if (ApiConfig.save(disabledConfig)) {
                Glint.LOGGER.info("API sync disabled")
                minecraft?.setScreen(parent)
            }
        }

        override fun render(
            guiGraphics: GuiGraphics,
            mouseX: Int,
            mouseY: Int,
            delta: Float,
        ) {
            renderBackground(guiGraphics, mouseX, mouseY, delta)
            guiGraphics.drawCenteredString(font, title, width / 2, height / 2 - 70, 0xFFFFFF)

            val centerX = width / 2
            val startY = height / 2 - 40

            // Current configuration
            guiGraphics.drawCenteredString(
                font,
                "✓ Connected to server",
                centerX,
                startY,
                0x00FF00,
            )

            guiGraphics.drawString(
                font,
                "Server URL:",
                centerX - 150,
                startY + 20,
                0xAAAAAA,
            )
            guiGraphics.drawString(
                font,
                config.apiUrl,
                centerX - 150,
                startY + 32,
                0xFFFFFF,
            )

            guiGraphics.drawString(
                font,
                "Selected World:",
                centerX - 150,
                startY + 44,
                0xAAAAAA,
            )
            val worldDisplay =
                if (config.worldName.isNotEmpty()) {
                    "${config.worldName} (${config.worldId.take(8)}...)"
                } else {
                    config.worldId.take(16) + "..."
                }
            guiGraphics.drawString(
                font,
                worldDisplay,
                centerX - 150,
                startY + 56,
                0xFFFFFF,
            )

            super.render(guiGraphics, mouseX, mouseY, delta)
        }
    }
}
