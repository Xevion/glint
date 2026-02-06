package com.xevion.glint.ui

import com.xevion.glint.Loggers
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.DeviceTokenResponse
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.ButtonComponent
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.Surface
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import net.minecraft.network.chat.Component as McComponent

/**
 * Entry point for API configuration.
 * Shows edit screen if already configured, or starts wizard flow if not.
 *
 * Wizard flow:
 * 1. ServerConnectionScreen - validate URL
 * 2. DeviceAuthScreen - authenticate user
 * 3. WorldSelectionScreen - select world
 */
class ApiConfigWizardScreen(
    private val parent: Screen,
    private val showConnectionFirst: Boolean = false,
) : GlintScreen(McComponent.literal("API Configuration")) {
    private val config: ApiConfig = ApiConfig.load()

    // Store token response from device auth for world selection
    private var pendingToken: DeviceTokenResponse? = null
    private var pendingServerUrl: String? = null

    override fun buildContent(root: FlowLayout) {
        // This screen immediately forwards to another screen
        // But we still need to build something for the brief moment it's shown
        root.child(
            Components
                .label(McComponent.literal("Loading..."))
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
    }

    override fun init() {
        super.init()

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
                    showDeviceAuth(validatedUrl)
                },
                initialUrl = config.apiUrl.ifBlank { "http://localhost:8080" },
            ),
        )
    }

    private fun showDeviceAuth(serverUrl: String) {
        minecraft?.setScreen(
            DeviceAuthScreen(
                parent = parent,
                serverUrl = serverUrl,
                onAuthorized = { tokenResponse ->
                    pendingToken = tokenResponse
                    pendingServerUrl = serverUrl
                    showWorldSelection(serverUrl, tokenResponse)
                },
                onBack = { showConnectionScreen() },
            ),
        )
    }

    private fun showWorldSelection(
        serverUrl: String,
        tokenResponse: DeviceTokenResponse,
    ) {
        minecraft?.setScreen(
            WorldSelectionScreen(
                parent = parent,
                serverUrl = serverUrl,
                accessToken = tokenResponse.accessToken,
                tokenExpiresIn = tokenResponse.expiresIn,
                onBack = { showDeviceAuth(serverUrl) },
            ),
        )
    }

    private fun showEditScreen() {
        minecraft?.setScreen(ConfigEditScreen(parent, config))
    }

    /**
     * Edit screen for already-configured API connection.
     */
    private inner class ConfigEditScreen(
        private val parentScreen: Screen,
        private val apiConfig: ApiConfig,
    ) : GlintScreen(McComponent.literal("API Configuration")) {
        private lateinit var changeServerButton: ButtonComponent
        private lateinit var disableButton: ButtonComponent
        private lateinit var doneButton: ButtonComponent

        override fun buildContent(root: FlowLayout) {
            val content = Containers.verticalFlow(Sizing.content(), Sizing.content())
            content.horizontalAlignment(HorizontalAlignment.CENTER)
            content.gap(GlintTheme.GAP_MD)
            content.padding(GlintTheme.paddingLg())
            content.surface(Surface.DARK_PANEL)

            // Title
            content.child(GlintComponents.title(title) as Component)

            // Status
            content.child(
                Components
                    .label(McComponent.literal("Connected to server"))
                    .color(Color.ofRgb(GlintTheme.TEXT_SUCCESS)) as Component,
            )

            // Config info
            val infoSection = Containers.verticalFlow(Sizing.content(), Sizing.content())
            infoSection.horizontalAlignment(HorizontalAlignment.LEFT)
            infoSection.gap(GlintTheme.GAP_SM)

            // Server URL
            infoSection.child(
                Components
                    .label(McComponent.literal("Server URL:"))
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
            )
            infoSection.child(
                Components
                    .label(McComponent.literal(apiConfig.apiUrl))
                    .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component,
            )

            // World
            infoSection.child(
                Components
                    .label(McComponent.literal("Selected World:"))
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
            )
            val worldDisplay =
                if (apiConfig.worldName.isNotEmpty()) {
                    "${apiConfig.worldName} (${apiConfig.worldId.take(8)}...)"
                } else {
                    apiConfig.worldId.take(16) + "..."
                }
            infoSection.child(
                Components
                    .label(McComponent.literal(worldDisplay))
                    .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component,
            )

            content.child(infoSection as Component)

            // Buttons
            changeServerButton = Components.button(McComponent.literal("Change Server")) { showConnectionScreen() }
            (changeServerButton as Component).horizontalSizing(Sizing.fixed(310))
            content.child(changeServerButton as Component)

            disableButton = Components.button(McComponent.literal("Disable Sync")) { disableSync() }
            (disableButton as Component).horizontalSizing(Sizing.fixed(310))
            content.child(disableButton as Component)

            doneButton = GlintComponents.wideButton(CommonComponents.GUI_DONE) { minecraft?.setScreen(parentScreen) }
            content.child(doneButton as Component)

            root.child(content as Component)
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
                Loggers.Ui.get().info("API sync disabled")
                minecraft?.setScreen(parentScreen)
            }
        }
    }
}
