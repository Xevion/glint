package com.xevion.glint.ui

import com.xevion.glint.Loggers
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.GlintApi
import com.xevion.glint.api.UrlValidationResult
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.ButtonComponent
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.component.LabelComponent
import io.wispforest.owo.ui.component.TextBoxComponent
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.Insets
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.Surface
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import java.util.concurrent.CompletableFuture
import net.minecraft.network.chat.Component as McComponent

class ServerConnectionScreen(
    private val parent: Screen,
    private val onConnectionValidated: (String) -> Unit,
    private val initialUrl: String = "http://localhost:8080",
) : GlintScreen(McComponent.literal("Connect to Glint Server")) {
    companion object {
        private val log = Loggers.Ui.get()
    }

    private lateinit var urlInput: TextBoxComponent
    private lateinit var testButton: ButtonComponent
    private lateinit var skipButton: ButtonComponent
    private lateinit var feedbackLabel: LabelComponent

    private var validationResult: UrlValidationResult = UrlValidationResult.Empty
    private var testingConnection = false
    private var connectionTestResult: String? = null
    private var connectionTestError: String? = null

    override fun buildContent(root: FlowLayout) {
        val content = Containers.verticalFlow(Sizing.content(), Sizing.content())
        content.horizontalAlignment(HorizontalAlignment.CENTER)
        content.gap(GlintTheme.GAP_MD)
        content.padding(GlintTheme.paddingLg())
        content.surface(Surface.DARK_PANEL)

        // Title
        content.child(GlintComponents.title(title) as Component)

        // URL input section
        val inputSection = Containers.verticalFlow(Sizing.content(), Sizing.content())
        inputSection.horizontalAlignment(HorizontalAlignment.LEFT)
        inputSection.gap(GlintTheme.GAP_SM)

        inputSection.child(
            Components
                .label(McComponent.literal("Server URL:"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )

        urlInput = Components.textBox(Sizing.fixed(300))
        urlInput.setMaxLength(256)
        urlInput.setSuggestion("http://localhost:8080")
        urlInput.text(initialUrl)
        urlInput.onChanged().subscribe { validateUrl() }
        inputSection.child(urlInput as Component)

        content.child(inputSection as Component)

        // Feedback label
        feedbackLabel = Components.label(McComponent.literal(""))
        feedbackLabel.horizontalTextAlignment(HorizontalAlignment.CENTER)
        feedbackLabel.margins(Insets.vertical(GlintTheme.GAP_SM))
        content.child(feedbackLabel as Component)

        // Button row
        testButton = GlintComponents.wideButton(McComponent.literal("Test Connection")) { testConnection() }
        skipButton = GlintComponents.wideButton(McComponent.literal("Skip (Disable Sync)")) { skipConnection() }
        content.child(
            GlintComponents.buttonRow(testButton, skipButton) as Component,
        )

        // Cancel button
        content.child(
            GlintComponents.wideButton(CommonComponents.GUI_CANCEL) { minecraft?.setScreen(parent) } as Component,
        )

        // Info text
        content.child(
            Components
                .label(McComponent.literal("Connect to your Glint backend server for scene synchronization"))
                .color(Color.ofRgb(0x666666))
                .margins(Insets.top(GlintTheme.GAP_MD)) as Component,
        )

        root.child(content as Component)

        // Initial validation
        validateUrl()

        // Focus the input
        val input = urlInput
        uiAdapter.rootComponent.focusHandler()?.focus(input as Component, null)
    }

    private fun validateUrl() {
        validationResult = GlintApi.validateApiUrl(urlInput.value)
        connectionTestResult = null
        connectionTestError = null
        updateFeedback()
        updateButtonStates()
    }

    private fun updateButtonStates() {
        testButton.active = !testingConnection && validationResult is UrlValidationResult.Valid
    }

    private fun updateFeedback() {
        when (validationResult) {
            is UrlValidationResult.Empty -> {
                feedbackLabel.text(McComponent.literal("Enter protocol://hostname:port (e.g., http://localhost:8080)"))
                feedbackLabel.color(Color.ofRgb(GlintTheme.TEXT_MUTED))
            }

            is UrlValidationResult.Valid -> {
                when {
                    testingConnection -> {
                        feedbackLabel.text(McComponent.literal("Testing connection..."))
                        feedbackLabel.color(Color.ofRgb(GlintTheme.TEXT_WARNING))
                    }

                    connectionTestResult != null -> {
                        feedbackLabel.text(McComponent.literal("OK: $connectionTestResult"))
                        feedbackLabel.color(Color.ofRgb(GlintTheme.TEXT_SUCCESS))
                    }

                    connectionTestError != null -> {
                        feedbackLabel.text(McComponent.literal("Error: $connectionTestError"))
                        feedbackLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
                    }

                    else -> {
                        feedbackLabel.text(McComponent.literal("URL format valid - click Test Connection"))
                        feedbackLabel.color(Color.ofRgb(GlintTheme.TEXT_MUTED))
                    }
                }
            }

            is UrlValidationResult.Invalid -> {
                val error = (validationResult as UrlValidationResult.Invalid).reason
                feedbackLabel.text(McComponent.literal("Error: $error"))
                feedbackLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
            }
        }
    }

    private fun testConnection() {
        if (validationResult !is UrlValidationResult.Valid) return

        testingConnection = true
        connectionTestResult = null
        connectionTestError = null
        updateFeedback()
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
                            log.info("Connection test successful") { "url" to normalizedUrl }
                            updateFeedback()

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
                            log.warn("Connection test failed") { "error" to error.message }
                            updateFeedback()
                        }
                    updateButtonStates()
                }
            }
    }

    private fun skipConnection() {
        val config =
            ApiConfig(
                apiUrl = "",
                worldId = "",
                worldName = "",
                enabled = false,
                validated = false,
            )
        ApiConfig.save(config)
        log.info("API sync disabled by user")
        minecraft?.setScreen(parent)
    }
}
