package com.xevion.glint.ui.tabs

import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.UrlValidation
import com.xevion.glint.ui.ApiConfigWizardScreen
import com.xevion.glint.ui.GlintMainScreen
import com.xevion.glint.ui.StatusLog
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.Insets
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.VerticalAlignment
import java.util.concurrent.CompletableFuture
import net.minecraft.network.chat.Component as McComponent

class ConfigTab(
    private val host: GlintMainScreen,
) : MainScreenTab {
    private var connectionTestResult: String? = null
    private var connectionTesting = false

    override fun buildMaster(master: FlowLayout) {
        val config = ApiConfig.load()

        master.child(
            GlintComponents.title(McComponent.literal("API Connection")),
        )

        if (!config.enabled || config.apiUrl.isBlank()) {
            master.child(
                Components
                    .label(McComponent.literal("No API connection configured."))
                    .maxWidth(host.masterTextWidth)
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)),
            )
            master.child(
                Components
                    .label(McComponent.literal("Set up a connection to sync scenes and download worlds."))
                    .maxWidth(host.masterTextWidth)
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)),
            )

            val buttonRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
            buttonRow.gap(GlintTheme.GAP_SM)
            buttonRow.padding(Insets.vertical(GlintTheme.GAP_MD))
            buttonRow.child(
                GlintComponents.smallButton(
                    McComponent.literal("Set Up Connection"),
                    width = 110,
                ) {
                    host.client?.setScreen(ApiConfigWizardScreen(host, showConnectionFirst = true))
                } as Component,
            )
            master.child(buttonRow)
            return
        }

        val infoContainer = Containers.verticalFlow(Sizing.fill(100), Sizing.content())
        infoContainer.gap(GlintTheme.GAP_SM)
        infoContainer.padding(Insets.vertical(GlintTheme.GAP_SM))

        infoContainer.child(
            Components
                .label(McComponent.literal("Server"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)),
        )
        infoContainer.child(
            Components
                .label(McComponent.literal(config.apiUrl))
                .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)),
        )

        infoContainer.child(
            Components
                .label(McComponent.literal("Status"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)),
        )

        val statusColor: Int
        val statusText: String
        when {
            !config.validated -> {
                statusColor = GlintTheme.TEXT_WARNING
                statusText = "Not validated"
            }

            !config.hasValidToken() -> {
                statusColor = GlintTheme.TEXT_ERROR
                statusText = "Token expired"
            }

            config.isTokenExpiringSoon() -> {
                statusColor = GlintTheme.TEXT_WARNING
                val minutesLeft = ((config.tokenExpiresAt - System.currentTimeMillis()) / 60_000).toInt()
                statusText = "Connected (token expires in ${minutesLeft}m)"
            }

            else -> {
                statusColor = GlintTheme.TEXT_SUCCESS
                val hoursLeft = ((config.tokenExpiresAt - System.currentTimeMillis()) / 3_600_000).toInt()
                statusText = "Connected (token expires in ${hoursLeft}h)"
            }
        }
        infoContainer.child(
            Components
                .label(McComponent.literal(statusText))
                .color(Color.ofRgb(statusColor)),
        )

        master.child(infoContainer)

        val row1 = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        row1.gap(GlintTheme.GAP_SM)
        row1.padding(Insets.vertical(GlintTheme.GAP_SM))

        val testBtn =
            GlintComponents.smallButton(
                McComponent.literal(if (connectionTesting) "Testing..." else "Test Connection"),
                width = 100,
                tooltip = McComponent.literal("Test connection to API server"),
            ) {
                if (!connectionTesting) testConnection(config)
            }
        if (connectionTesting) testBtn.active = false
        row1.child(testBtn as Component)

        if (!config.hasValidToken()) {
            row1.child(
                GlintComponents.smallButton(
                    McComponent.literal("Re-authenticate"),
                    width = 100,
                ) {
                    host.client?.setScreen(ApiConfigWizardScreen(host))
                } as Component,
            )
        }

        master.child(row1)

        val row2 = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        row2.gap(GlintTheme.GAP_SM)

        row2.child(
            GlintComponents.smallButton(
                McComponent.literal("Change Server"),
                width = 90,
            ) {
                host.client?.setScreen(ApiConfigWizardScreen(host, showConnectionFirst = true))
            } as Component,
        )

        master.child(row2)

        val row3 = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        row3.padding(Insets.top(GlintTheme.GAP_MD))
        row3.child(
            GlintComponents.smallButton(
                McComponent.literal("Disconnect"),
                width = 75,
                tooltip = McComponent.literal("Remove API connection"),
            ) {
                disconnectApi()
            } as Component,
        )
        master.child(row3)
    }

    override fun buildDetail(detail: FlowLayout) {
        detail.horizontalAlignment(HorizontalAlignment.LEFT)
        detail.verticalAlignment(VerticalAlignment.TOP)

        detail.child(
            GlintComponents.title(McComponent.literal("Diagnostics")),
        )

        val config = ApiConfig.load()

        if (!config.enabled || config.apiUrl.isBlank()) {
            detail.child(
                Components
                    .label(McComponent.literal("Use 'Set Up Connection' to configure the API."))
                    .maxWidth(host.detailTextWidth)
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)),
            )
            detail.child(
                Components
                    .label(
                        McComponent.literal(
                            "The wizard will guide you through server URL validation and authentication.",
                        ),
                    ).maxWidth(host.detailTextWidth)
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)),
            )
            return
        }

        if (connectionTestResult != null) {
            detail.child(
                Components
                    .label(McComponent.literal("Connection Test"))
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)),
            )
            val isSuccess = connectionTestResult!!.startsWith("Success")
            detail.child(
                Components
                    .label(McComponent.literal(connectionTestResult!!))
                    .maxWidth(host.detailTextWidth)
                    .color(Color.ofRgb(if (isSuccess) GlintTheme.TEXT_SUCCESS else GlintTheme.TEXT_ERROR)),
            )
        }

        if (config.hasValidToken()) {
            val expiresIn = config.tokenExpiresAt - System.currentTimeMillis()
            val hours = (expiresIn / 3_600_000).toInt()
            val minutes = ((expiresIn % 3_600_000) / 60_000).toInt()

            detail.child(
                Components
                    .label(McComponent.literal("Token Expiry"))
                    .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)),
            )
            detail.child(
                Components
                    .label(McComponent.literal("${hours}h ${minutes}m remaining"))
                    .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)),
            )
        }

        detail.child(
            Components
                .label(McComponent.literal("Endpoint"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)),
        )
        detail.child(
            Components
                .label(McComponent.literal("${config.apiUrl}/api"))
                .maxWidth(host.detailTextWidth)
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)),
        )
    }

    private fun testConnection(config: ApiConfig) {
        connectionTesting = true
        connectionTestResult = null
        host.triggerRefreshMaster()
        host.triggerRefreshDetail()

        CompletableFuture
            .supplyAsync {
                val start = System.currentTimeMillis()
                val result = UrlValidation.testConnection(config.apiUrl, token = config.accessToken)
                val latency = System.currentTimeMillis() - start
                Pair(result, latency)
            }.thenAccept { (result, latency) ->
                host.client?.execute {
                    connectionTesting = false
                    result
                        .onSuccess { testResult ->
                            connectionTestResult = "Success (${latency}ms)"
                            StatusLog.info("Connection test passed (${latency}ms) → ${testResult.resolvedUrl}")
                        }.onFailure { error ->
                            connectionTestResult = "Failed: ${error.message}"
                            StatusLog.error("Connection test failed: ${error.message}")
                        }
                    host.triggerRefreshMaster()
                    host.triggerRefreshDetail()
                    host.triggerRebuildStatusBar()
                }
            }
    }

    private fun disconnectApi() {
        val disabledConfig = ApiConfig()
        if (ApiConfig.save(disabledConfig)) {
            StatusLog.info("API connection removed")
            connectionTestResult = null
            host.triggerRefreshMaster()
            host.triggerRefreshDetail()
            host.triggerRebuildStatusBar()
        }
    }
}
