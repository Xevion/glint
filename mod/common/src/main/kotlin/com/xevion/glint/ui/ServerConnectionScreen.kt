package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.GlintApi
import com.xevion.glint.api.UrlValidationResult
import net.minecraft.client.gui.GuiGraphics
import net.minecraft.client.gui.components.Button
import net.minecraft.client.gui.components.EditBox
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import net.minecraft.network.chat.Component
import java.util.concurrent.CompletableFuture

class ServerConnectionScreen(
    private val parent: Screen,
    private val onConnectionValidated: (String) -> Unit,
    private val initialUrl: String = "http://localhost:8080",
) : Screen(Component.literal("Connect to Glint Server")) {
    private lateinit var urlInput: EditBox
    private lateinit var testButton: Button
    private lateinit var skipButton: Button

    private var validationResult: UrlValidationResult = UrlValidationResult.Empty
    private var testingConnection = false
    private var connectionTestResult: String? = null
    private var connectionTestError: String? = null

    override fun init() {
        val centerX = width / 2
        val startY = height / 2 - 60

        // URL input with validation feedback
        urlInput = EditBox(font, centerX - 150, startY, 300, 20, Component.literal("Server URL"))
        urlInput.setHint(Component.literal("http://localhost:8080"))
        urlInput.setMaxLength(256)
        urlInput.value = initialUrl
        urlInput.setResponder { value ->
            validationResult = GlintApi.validateApiUrl(value)
            connectionTestResult = null
            connectionTestError = null
            updateButtonStates()
        }
        addRenderableWidget(urlInput)

        // Test Connection button
        testButton =
            Button
                .builder(Component.literal("Test Connection")) { testConnection() }
                .bounds(centerX - 155, startY + 40, 150, 20)
                .build()
        addRenderableWidget(testButton)

        // Skip button
        skipButton =
            Button
                .builder(Component.literal("Skip (Disable Sync)")) { skipConnection() }
                .bounds(centerX + 5, startY + 40, 150, 20)
                .build()
        addRenderableWidget(skipButton)

        // Back button
        addRenderableWidget(
            Button
                .builder(CommonComponents.GUI_CANCEL) { minecraft?.setScreen(parent) }
                .bounds(centerX - 75, startY + 80, 150, 20)
                .build(),
        )

        // Initial validation
        validationResult = GlintApi.validateApiUrl(urlInput.value)
        updateButtonStates()
        setFocused(urlInput)
    }

    private fun updateButtonStates() {
        testButton.active = !testingConnection && validationResult is UrlValidationResult.Valid
    }

    private fun testConnection() {
        if (validationResult !is UrlValidationResult.Valid) return

        testingConnection = true
        connectionTestResult = null
        connectionTestError = null
        updateButtonStates()

        val normalizedUrl = (validationResult as UrlValidationResult.Valid).normalizedUrl

        CompletableFuture
            .supplyAsync {
                GlintApi.testConnection(normalizedUrl)
            }.thenAccept { result ->
                minecraft?.execute {
                    testingConnection = false
                    result
                        .onSuccess { message ->
                            connectionTestResult = message
                            connectionTestError = null
                            Glint.LOGGER.info("Connection test successful: {}", normalizedUrl)

                            // Automatically proceed to world selection after 1 second
                            minecraft?.execute {
                                Thread.sleep(1000)
                                minecraft?.execute {
                                    onConnectionValidated(normalizedUrl)
                                }
                            }
                        }.onFailure { error ->
                            connectionTestResult = null
                            connectionTestError = error.message ?: "Connection failed"
                            Glint.LOGGER.warn("Connection test failed: {}", error.message)
                        }
                    updateButtonStates()
                }
            }
    }

    private fun skipConnection() {
        // Save disabled config
        val config =
            ApiConfig(
                apiUrl = "",
                worldId = "",
                worldName = "",
                enabled = false,
                validated = false,
            )
        ApiConfig.save(config)
        Glint.LOGGER.info("API sync disabled by user")
        minecraft?.setScreen(parent)
    }

    override fun render(
        guiGraphics: GuiGraphics,
        mouseX: Int,
        mouseY: Int,
        delta: Float,
    ) {
        renderBackground(guiGraphics, mouseX, mouseY, delta)
        guiGraphics.drawCenteredString(font, title, width / 2, height / 2 - 90, 0xFFFFFF)

        val centerX = width / 2
        val startY = height / 2 - 60

        // URL label
        guiGraphics.drawString(font, "Server URL:", centerX - 150, startY - 12, 0xAAAAAA)

        // Validation feedback
        val feedbackY = startY + 22
        when (validationResult) {
            is UrlValidationResult.Empty -> {
                guiGraphics.drawCenteredString(
                    font,
                    "Enter protocol://hostname:port (e.g., http://localhost:8080)",
                    centerX,
                    feedbackY,
                    0x888888,
                )
            }

            is UrlValidationResult.Valid -> {
                if (testingConnection) {
                    guiGraphics.drawCenteredString(
                        font,
                        "Testing connection...",
                        centerX,
                        feedbackY,
                        0xFFAA00,
                    )
                } else if (connectionTestResult != null) {
                    guiGraphics.drawCenteredString(
                        font,
                        "OK: $connectionTestResult",
                        centerX,
                        feedbackY,
                        0x00FF00,
                    )
                } else if (connectionTestError != null) {
                    guiGraphics.drawCenteredString(
                        font,
                        "Error: $connectionTestError",
                        centerX,
                        feedbackY,
                        0xFF0000,
                    )
                } else {
                    guiGraphics.drawCenteredString(
                        font,
                        "URL format valid - click Test Connection",
                        centerX,
                        feedbackY,
                        0x888888,
                    )
                }
            }

            is UrlValidationResult.Invalid -> {
                val error = (validationResult as UrlValidationResult.Invalid).reason
                guiGraphics.drawCenteredString(
                    font,
                    "Error: $error",
                    centerX,
                    feedbackY,
                    0xFF0000,
                )
            }
        }

        // Info text
        val infoY = startY + 110
        guiGraphics.drawCenteredString(
            font,
            "Connect to your Glint backend server for scene synchronization",
            centerX,
            infoY,
            0x666666,
        )

        super.render(guiGraphics, mouseX, mouseY, delta)
    }
}
