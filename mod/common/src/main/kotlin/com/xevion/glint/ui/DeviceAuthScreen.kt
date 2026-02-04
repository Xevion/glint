package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.api.ApiError
import com.xevion.glint.api.DeviceAuthResponse
import com.xevion.glint.api.DeviceTokenResponse
import com.xevion.glint.api.GlintApi
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.ButtonComponent
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.component.LabelComponent
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.Surface
import net.minecraft.Util
import net.minecraft.client.gui.screens.Screen
import java.util.concurrent.CompletableFuture
import java.util.concurrent.atomic.AtomicBoolean
import net.minecraft.network.chat.Component as McComponent

/**
 * Screen for OAuth 2.0 Device Authorization flow.
 * Displays the user code and URL, polls for authorization completion.
 */
class DeviceAuthScreen(
    private val parent: Screen,
    private val serverUrl: String,
    private val onAuthorized: (DeviceTokenResponse) -> Unit,
    private val onBack: () -> Unit,
) : GlintScreen(McComponent.literal("Authorize Device")) {
    private lateinit var copyUrlButton: ButtonComponent
    private lateinit var openBrowserButton: ButtonComponent
    private lateinit var cancelButton: ButtonComponent
    private lateinit var contentContainer: FlowLayout
    private lateinit var statusLabel: LabelComponent
    private lateinit var instructionsContainer: FlowLayout

    private var authResponse: DeviceAuthResponse? = null
    private var isLoading = true
    private var errorMessage: String? = null
    private var isPolling = AtomicBoolean(false)
    private var pollFuture: CompletableFuture<*>? = null

    private var startTime: Long = 0L
    private var expiresAtTime: Long = 0L

    override fun buildContent(root: FlowLayout) {
        contentContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
        contentContainer.horizontalAlignment(HorizontalAlignment.CENTER)
        contentContainer.gap(GlintTheme.GAP_MD)
        contentContainer.padding(GlintTheme.paddingLg())
        contentContainer.surface(Surface.DARK_PANEL)

        // Title
        contentContainer.child(GlintComponents.title(title) as Component)

        // Status/instructions area
        statusLabel = Components.label(McComponent.literal("Starting authorization..."))
        statusLabel.color(Color.ofRgb(GlintTheme.TEXT_WARNING))
        statusLabel.horizontalTextAlignment(HorizontalAlignment.CENTER)
        contentContainer.child(statusLabel as Component)

        // Instructions container (hidden initially)
        instructionsContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
        instructionsContainer.horizontalAlignment(HorizontalAlignment.CENTER)
        instructionsContainer.gap(GlintTheme.GAP_SM)

        // Button row
        copyUrlButton = GlintComponents.wideButton(McComponent.literal("Copy URL")) { copyUrl() }
        copyUrlButton.active = false
        openBrowserButton = GlintComponents.wideButton(McComponent.literal("Open in Browser")) { openBrowser() }
        openBrowserButton.active = false
        contentContainer.child(
            GlintComponents.buttonRow(copyUrlButton, openBrowserButton) as Component,
        )

        // Cancel button
        cancelButton = GlintComponents.wideButton(McComponent.literal("Cancel")) { cancel() }
        contentContainer.child(cancelButton as Component)

        root.child(contentContainer as Component)

        // Start device auth flow
        startDeviceAuth()
    }

    private fun rebuildInstructions() {
        instructionsContainer.clearChildren()
        val response = authResponse ?: return

        instructionsContainer.child(
            Components
                .label(McComponent.literal("To connect this Minecraft client to Glint:"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )

        // Step 1 - URL
        val step1 = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        step1.gap(GlintTheme.GAP_SM)
        step1.child(Components.label(McComponent.literal("1. Open:")).color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component)
        step1.child(Components.label(McComponent.literal(response.verificationUri)).color(Color.ofRgb(0x55FFFF)) as Component)
        instructionsContainer.child(step1 as Component)

        // Step 2 - Code
        val step2 = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        step2.gap(GlintTheme.GAP_SM)
        step2.child(Components.label(McComponent.literal("2. Enter code:")).color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component)
        step2.child(Components.label(McComponent.literal(response.userCode)).color(Color.ofRgb(0x55FF55)) as Component)
        instructionsContainer.child(step2 as Component)

        // Timer - will be updated in tick
        instructionsContainer.child(
            Components
                .label(McComponent.literal("Waiting for authorization..."))
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED))
                .id("timer-label") as Component,
        )
    }

    override fun tick() {
        super.tick()

        // Update timer display if we have an auth response
        if (authResponse != null && !isLoading && errorMessage == null) {
            val remainingSeconds = ((expiresAtTime - System.currentTimeMillis()) / 1000).coerceAtLeast(0)
            val minutes = remainingSeconds / 60
            val seconds = remainingSeconds % 60
            val timeDisplay = String.format("%d:%02d", minutes, seconds)

            val timerLabel = instructionsContainer.childById(LabelComponent::class.java, "timer-label")
            timerLabel?.text(McComponent.literal("Waiting for authorization... (expires in $timeDisplay)"))
        }
    }

    private fun startDeviceAuth() {
        isLoading = true
        errorMessage = null

        CompletableFuture
            .supplyAsync {
                GlintApi.startDeviceAuth(serverUrl)
            }.thenAccept { result ->
                minecraft?.execute {
                    isLoading = false
                    result
                        .onSuccess { response ->
                            authResponse = response
                            startTime = System.currentTimeMillis()
                            expiresAtTime = startTime + (response.expiresIn * 1000)
                            copyUrlButton.active = true
                            openBrowserButton.active = true

                            // Show instructions
                            rebuildInstructions()
                            statusLabel.text(McComponent.literal(""))

                            // Insert instructions before buttons by rebuilding content
                            val children = contentContainer.children().toMutableList()
                            val buttonIndex =
                                children.indexOfFirst { child ->
                                    child === copyUrlButton || (child is FlowLayout && child.children().any { it === copyUrlButton })
                                }
                            if (buttonIndex >= 0 && !children.contains(instructionsContainer)) {
                                children.add(buttonIndex, instructionsContainer as Component)
                                contentContainer.clearChildren()
                                children.forEach { contentContainer.child(it) }
                            } else if (!children.contains(instructionsContainer)) {
                                contentContainer.child(instructionsContainer as Component)
                            }

                            Glint.LOGGER.info("Device auth started: {}", response.userCode)

                            // Start polling
                            startPolling(response)
                        }.onFailure { error ->
                            errorMessage = error.message ?: "Failed to start device authorization"
                            statusLabel.text(McComponent.literal("Error: $errorMessage"))
                            statusLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
                            Glint.LOGGER.error("Device auth failed: {}", error.message)
                        }
                }
            }
    }

    private fun startPolling(response: DeviceAuthResponse) {
        if (isPolling.getAndSet(true)) return

        pollFuture =
            CompletableFuture.runAsync {
                while (isPolling.get()) {
                    try {
                        Thread.sleep(response.interval * 1000)
                    } catch (_: InterruptedException) {
                        break
                    }

                    if (!isPolling.get()) break

                    val result = GlintApi.pollDeviceToken(serverUrl, response.deviceCode)
                    result
                        .onSuccess { tokenResponse ->
                            isPolling.set(false)
                            minecraft?.execute {
                                Glint.LOGGER.info("Device authorized successfully")
                                onAuthorized(tokenResponse)
                            }
                        }.onFailure { error ->
                            when (error) {
                                is ApiError.AuthorizationPending -> {
                                    // Continue polling
                                }
                                is ApiError.TokenExpired -> {
                                    isPolling.set(false)
                                    minecraft?.execute {
                                        errorMessage = "Authorization code expired. Please try again."
                                        statusLabel.text(McComponent.literal("Error: $errorMessage"))
                                        statusLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
                                        authResponse = null
                                        copyUrlButton.active = false
                                        openBrowserButton.active = false
                                        contentContainer.removeChild(instructionsContainer as Component)
                                    }
                                }
                                is ApiError.InvalidGrant -> {
                                    isPolling.set(false)
                                    minecraft?.execute {
                                        errorMessage = "Invalid authorization code. Please try again."
                                        statusLabel.text(McComponent.literal("Error: $errorMessage"))
                                        statusLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
                                        authResponse = null
                                        copyUrlButton.active = false
                                        openBrowserButton.active = false
                                        contentContainer.removeChild(instructionsContainer as Component)
                                    }
                                }
                                else -> {
                                    isPolling.set(false)
                                    minecraft?.execute {
                                        errorMessage = error.message ?: "Authorization failed"
                                        statusLabel.text(McComponent.literal("Error: $errorMessage"))
                                        statusLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
                                    }
                                }
                            }
                        }
                }
            }
    }

    private fun copyUrl() {
        val response = authResponse ?: return
        minecraft?.keyboardHandler?.clipboard = response.verificationUriComplete
        Glint.LOGGER.info("Copied verification URL to clipboard")
    }

    private fun openBrowser() {
        val response = authResponse ?: return
        Util.getPlatform().openUri(response.verificationUriComplete)
        Glint.LOGGER.info("Opened verification URL in browser")
    }

    private fun cancel() {
        isPolling.set(false)
        pollFuture?.cancel(true)
        onBack()
    }

    override fun removed() {
        super.removed()
        isPolling.set(false)
        pollFuture?.cancel(true)
    }
}
