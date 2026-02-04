package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.api.ApiError
import com.xevion.glint.api.DeviceAuthResponse
import com.xevion.glint.api.DeviceTokenResponse
import com.xevion.glint.api.GlintApi
import net.minecraft.Util
import net.minecraft.client.gui.GuiGraphics
import net.minecraft.client.gui.components.Button
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.Component
import java.util.concurrent.CompletableFuture
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Screen for OAuth 2.0 Device Authorization flow.
 * Displays the user code and URL, polls for authorization completion.
 */
class DeviceAuthScreen(
    private val parent: Screen,
    private val serverUrl: String,
    private val onAuthorized: (DeviceTokenResponse) -> Unit,
    private val onBack: () -> Unit,
) : Screen(Component.literal("Authorize Device")) {
    private lateinit var copyUrlButton: Button
    private lateinit var openBrowserButton: Button
    private lateinit var cancelButton: Button

    private var authResponse: DeviceAuthResponse? = null
    private var isLoading = true
    private var errorMessage: String? = null
    private var isPolling = AtomicBoolean(false)
    private var pollFuture: CompletableFuture<*>? = null

    private var startTime: Long = 0L
    private var expiresAtTime: Long = 0L

    override fun init() {
        val centerX = width / 2
        val buttonStartY = height / 2 + 40

        // Copy URL button
        copyUrlButton =
            Button
                .builder(Component.literal("Copy URL")) { copyUrl() }
                .bounds(centerX - 155, buttonStartY, 150, 20)
                .build()
        copyUrlButton.active = false
        addRenderableWidget(copyUrlButton)

        // Open Browser button
        openBrowserButton =
            Button
                .builder(Component.literal("Open in Browser")) { openBrowser() }
                .bounds(centerX + 5, buttonStartY, 150, 20)
                .build()
        openBrowserButton.active = false
        addRenderableWidget(openBrowserButton)

        // Cancel button
        cancelButton =
            Button
                .builder(Component.literal("Cancel")) { cancel() }
                .bounds(centerX - 75, buttonStartY + 30, 150, 20)
                .build()
        addRenderableWidget(cancelButton)

        // Start device auth flow
        startDeviceAuth()
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
                            Glint.LOGGER.info("Device auth started: {}", response.userCode)

                            // Start polling
                            startPolling(response)
                        }.onFailure { error ->
                            errorMessage = error.message ?: "Failed to start device authorization"
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
                    } catch (e: InterruptedException) {
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
                                        authResponse = null
                                        copyUrlButton.active = false
                                        openBrowserButton.active = false
                                    }
                                }

                                is ApiError.InvalidGrant -> {
                                    isPolling.set(false)
                                    minecraft?.execute {
                                        errorMessage = "Invalid authorization code. Please try again."
                                        authResponse = null
                                        copyUrlButton.active = false
                                        openBrowserButton.active = false
                                    }
                                }

                                else -> {
                                    isPolling.set(false)
                                    minecraft?.execute {
                                        errorMessage = error.message ?: "Authorization failed"
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

    override fun render(
        guiGraphics: GuiGraphics,
        mouseX: Int,
        mouseY: Int,
        delta: Float,
    ) {
        renderBackground(guiGraphics, mouseX, mouseY, delta)

        val centerX = width / 2
        val startY = height / 2 - 80

        // Title
        guiGraphics.drawCenteredString(font, title, centerX, startY, 0xFFFFFF)

        if (isLoading) {
            guiGraphics.drawCenteredString(
                font,
                Component.literal("Starting authorization..."),
                centerX,
                startY + 40,
                0xFFAA00,
            )
        } else if (errorMessage != null) {
            guiGraphics.drawCenteredString(
                font,
                Component.literal("Error: $errorMessage"),
                centerX,
                startY + 40,
                0xFF0000,
            )

            guiGraphics.drawCenteredString(
                font,
                Component.literal("Click Cancel to go back and try again"),
                centerX,
                startY + 55,
                0x888888,
            )
        } else if (authResponse != null) {
            val response = authResponse!!

            // Instructions
            guiGraphics.drawCenteredString(
                font,
                Component.literal("To connect this Minecraft client to Glint:"),
                centerX,
                startY + 25,
                0xAAAAAA,
            )

            // Step 1
            guiGraphics.drawString(
                font,
                Component.literal("1. Open:"),
                centerX - 150,
                startY + 45,
                0xFFFFFF,
            )
            guiGraphics.drawString(
                font,
                Component.literal(response.verificationUri),
                centerX - 110,
                startY + 45,
                0x55FFFF,
            )

            // Step 2 with user code
            guiGraphics.drawString(
                font,
                Component.literal("2. Enter code:"),
                centerX - 150,
                startY + 60,
                0xFFFFFF,
            )

            // User code (highlighted)
            guiGraphics.drawString(
                font,
                Component.literal(response.userCode),
                centerX - 70,
                startY + 60,
                0x55FF55,
            )

            // Waiting message with countdown
            val remainingSeconds = ((expiresAtTime - System.currentTimeMillis()) / 1000).coerceAtLeast(0)
            val minutes = remainingSeconds / 60
            val seconds = remainingSeconds % 60
            val timeDisplay = String.format("%d:%02d", minutes, seconds)

            guiGraphics.drawCenteredString(
                font,
                Component.literal("Waiting for authorization... (expires in $timeDisplay)"),
                centerX,
                startY + 85,
                0x888888,
            )
        }

        super.render(guiGraphics, mouseX, mouseY, delta)
    }
}
