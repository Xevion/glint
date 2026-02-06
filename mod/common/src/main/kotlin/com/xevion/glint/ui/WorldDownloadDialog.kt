package com.xevion.glint.ui

import com.xevion.glint.Loggers
import com.xevion.glint.download.DownloadProgress
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintDialogScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.BoxComponent
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.component.LabelComponent
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.Insets
import io.wispforest.owo.ui.core.Sizing
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import java.util.concurrent.CompletableFuture
import net.minecraft.network.chat.Component as McComponent

/**
 * Dialog showing world download progress.
 */
class WorldDownloadDialog(
    private val parent: Screen,
    private val worldName: String,
    private val downloadFuture: CompletableFuture<String>,
) : GlintDialogScreen(McComponent.literal("Downloading World")) {
    private var currentProgress: DownloadProgress = DownloadProgress.downloading(0, 0)
    private var cancelled = false

    private lateinit var statusLabel: LabelComponent
    private lateinit var progressBar: BoxComponent
    private lateinit var progressBarBackground: BoxComponent
    private lateinit var progressText: LabelComponent
    private lateinit var bytesLabel: LabelComponent
    private lateinit var progressContainer: FlowLayout

    override fun buildDialog(dialog: FlowLayout) {
        // Title
        dialog.child(
            Components
                .label(McComponent.literal("Downloading: $worldName"))
                .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component,
        )

        // Progress container
        progressContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
        progressContainer.horizontalAlignment(HorizontalAlignment.CENTER)
        progressContainer.gap(GlintTheme.GAP_SM)

        // Progress bar (stacked layers)
        val barContainer = Containers.stack(Sizing.fixed(200), Sizing.fixed(20))

        progressBarBackground = Components.box(Sizing.fill(100), Sizing.fill(100))
        progressBarBackground.color(Color.ofRgb(0x333333))
        progressBarBackground.fill(true)

        progressBar = Components.box(Sizing.fill(0), Sizing.fill(100))
        progressBar.color(Color.ofRgb(0x00AA00))
        progressBar.fill(true)

        barContainer.child(progressBarBackground as Component)
        barContainer.child(progressBar as Component)

        progressContainer.child(barContainer as Component)

        // Progress percentage
        progressText = Components.label(McComponent.literal("0%"))
        progressText.color(Color.ofRgb(GlintTheme.TEXT_PRIMARY))
        progressContainer.child(progressText as Component)

        // Bytes label
        bytesLabel = Components.label(McComponent.literal("0 MB / 0 MB"))
        bytesLabel.color(Color.ofRgb(GlintTheme.TEXT_SECONDARY))
        progressContainer.child(bytesLabel as Component)

        dialog.child(progressContainer as Component)

        // Status label (for extracting/failed states)
        statusLabel = Components.label(McComponent.literal(""))
        statusLabel.color(Color.ofRgb(GlintTheme.TEXT_WARNING))
        statusLabel.margins(Insets.top(GlintTheme.GAP_SM))
        dialog.child(statusLabel as Component)

        // Cancel button
        dialog.child(
            GlintComponents.button(CommonComponents.GUI_CANCEL) { cancel() } as Component,
        )

        // Start listening for download completion
        downloadFuture
            .thenAccept { worldPath ->
                minecraft?.execute {
                    if (!cancelled) {
                        onDownloadComplete(worldPath)
                    }
                }
            }.exceptionally { e ->
                minecraft?.execute {
                    if (!cancelled) {
                        onDownloadFailed(e)
                    }
                }
                null
            }
    }

    override fun tick() {
        super.tick()
        updateProgressDisplay()
    }

    private fun updateProgressDisplay() {
        when (currentProgress.state) {
            DownloadProgress.State.DOWNLOADING -> {
                val progress = currentProgress.percentComplete
                (progressBar as Component).horizontalSizing(Sizing.fill(progress.coerceIn(0, 100)))
                progressText.text(McComponent.literal("$progress%"))

                val mbDownloaded = currentProgress.bytesDownloaded / (1024 * 1024)
                val mbTotal = currentProgress.totalBytes / (1024 * 1024)
                bytesLabel.text(McComponent.literal("$mbDownloaded MB / $mbTotal MB"))

                statusLabel.text(McComponent.literal(""))
            }

            DownloadProgress.State.EXTRACTING -> {
                statusLabel.text(McComponent.literal("Extracting world files..."))
                statusLabel.color(Color.ofRgb(GlintTheme.TEXT_WARNING))
            }

            DownloadProgress.State.COMPLETE -> {
                statusLabel.text(McComponent.literal(""))
            }

            DownloadProgress.State.FAILED -> {
                statusLabel.text(McComponent.literal("Download failed!"))
                statusLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
            }
        }
    }

    fun updateProgress(progress: DownloadProgress) {
        minecraft?.execute {
            currentProgress = progress
        }
    }

    private fun cancel() {
        cancelled = true
        downloadFuture.cancel(true)
        minecraft?.setScreen(parent)
    }

    private fun onDownloadComplete(worldPath: String) {
        Loggers.Ui.get().info("World download complete: {}", worldPath)
        minecraft?.setScreen(parent)
    }

    private fun onDownloadFailed(error: Throwable) {
        Loggers.Ui.get().error(error, "World download failed")
        currentProgress = DownloadProgress.failed()
    }
}
