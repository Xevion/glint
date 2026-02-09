package com.xevion.glint.ui

import com.xevion.glint.Loggers
import com.xevion.glint.api.WorldInfo
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintDialogScreen
import com.xevion.glint.ui.base.GlintTheme
import com.xevion.glint.upload.UploadProgress
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
 * Dialog showing world upload progress with staged status.
 */
class WorldUploadProgressDialog(
    private val parent: Screen,
    private val worldName: String,
    private val uploadFuture: CompletableFuture<WorldInfo>,
    private val onComplete: (WorldInfo) -> Unit,
) : GlintDialogScreen(McComponent.literal("Uploading World")) {
    private var currentProgress: UploadProgress = UploadProgress.saving()
    private var cancelled = false

    private lateinit var statusLabel: LabelComponent
    private lateinit var progressBar: BoxComponent
    private lateinit var progressText: LabelComponent
    private lateinit var bytesLabel: LabelComponent

    override fun buildDialog(dialog: FlowLayout) {
        // Title
        dialog.child(
            Components
                .label(McComponent.literal("Uploading: $worldName"))
                .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component,
        )

        // Progress container
        val progressContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
        progressContainer.horizontalAlignment(HorizontalAlignment.CENTER)
        progressContainer.gap(GlintTheme.GAP_SM)

        // Progress bar
        val barContainer = Containers.stack(Sizing.fixed(200), Sizing.fixed(20))

        val progressBarBackground = Components.box(Sizing.fill(100), Sizing.fill(100))
        progressBarBackground.color(Color.ofRgb(0x333333))
        progressBarBackground.fill(true)

        progressBar = Components.box(Sizing.fill(0), Sizing.fill(100))
        progressBar.color(Color.ofRgb(0x5555FF))
        progressBar.fill(true)

        barContainer.child(progressBarBackground as Component)
        barContainer.child(progressBar as Component)

        progressContainer.child(barContainer as Component)

        // Progress percentage
        progressText = Components.label(McComponent.literal(""))
        progressText.color(Color.ofRgb(GlintTheme.TEXT_PRIMARY))
        progressContainer.child(progressText as Component)

        // Bytes label
        bytesLabel = Components.label(McComponent.literal(""))
        bytesLabel.color(Color.ofRgb(GlintTheme.TEXT_SECONDARY))
        progressContainer.child(bytesLabel as Component)

        dialog.child(progressContainer as Component)

        // Status label
        statusLabel = Components.label(McComponent.literal("Saving world..."))
        statusLabel.color(Color.ofRgb(GlintTheme.TEXT_WARNING))
        statusLabel.margins(Insets.top(GlintTheme.GAP_SM))
        dialog.child(statusLabel as Component)

        // Cancel button
        dialog.child(
            GlintComponents.button(CommonComponents.GUI_CANCEL) { cancel() } as Component,
        )

        // Listen for completion
        uploadFuture
            .thenAccept { worldInfo ->
                minecraft?.execute {
                    if (!cancelled) {
                        onUploadComplete(worldInfo)
                    }
                }
            }.exceptionally { e ->
                minecraft?.execute {
                    if (!cancelled) {
                        onUploadFailed(e)
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
            UploadProgress.State.SAVING -> {
                statusLabel.text(McComponent.literal("Saving world to disk..."))
                statusLabel.color(Color.ofRgb(GlintTheme.TEXT_WARNING))
                progressText.text(McComponent.literal(""))
                bytesLabel.text(McComponent.literal(""))
            }

            UploadProgress.State.PACKAGING -> {
                statusLabel.text(McComponent.literal("Packaging world files..."))
                statusLabel.color(Color.ofRgb(GlintTheme.TEXT_WARNING))
                val progress = currentProgress.percentComplete
                (progressBar as Component).horizontalSizing(Sizing.fill(progress.coerceIn(0, 100)))
                progressText.text(McComponent.literal("$progress%"))
                val mbProcessed = currentProgress.bytesProcessed / (1024 * 1024)
                val mbTotal = currentProgress.totalBytes / (1024 * 1024)
                bytesLabel.text(McComponent.literal("$mbProcessed MB / $mbTotal MB"))
            }

            UploadProgress.State.HASHING -> {
                statusLabel.text(McComponent.literal("Computing checksum..."))
                statusLabel.color(Color.ofRgb(GlintTheme.TEXT_WARNING))
            }

            UploadProgress.State.UPLOADING -> {
                statusLabel.text(McComponent.literal("Uploading..."))
                statusLabel.color(Color.ofRgb(GlintTheme.TEXT_WARNING))
                val progress = currentProgress.percentComplete
                (progressBar as Component).horizontalSizing(Sizing.fill(progress.coerceIn(0, 100)))
                progressText.text(McComponent.literal("$progress%"))
                val mbProcessed = currentProgress.bytesProcessed / (1024 * 1024)
                val mbTotal = currentProgress.totalBytes / (1024 * 1024)
                bytesLabel.text(McComponent.literal("$mbProcessed MB / $mbTotal MB"))
            }

            UploadProgress.State.FINALIZING -> {
                statusLabel.text(McComponent.literal("Finalizing..."))
                statusLabel.color(Color.ofRgb(GlintTheme.TEXT_WARNING))
                (progressBar as Component).horizontalSizing(Sizing.fill(100))
                progressText.text(McComponent.literal(""))
                bytesLabel.text(McComponent.literal(""))
            }

            UploadProgress.State.COMPLETE -> {
                statusLabel.text(McComponent.literal("Upload complete!"))
                statusLabel.color(Color.ofRgb(GlintTheme.TEXT_SUCCESS))
                (progressBar as Component).horizontalSizing(Sizing.fill(100))
            }

            UploadProgress.State.FAILED -> {
                statusLabel.text(McComponent.literal(currentProgress.errorMessage ?: "Upload failed"))
                statusLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
            }
        }
    }

    fun updateProgress(progress: UploadProgress) {
        minecraft?.execute {
            currentProgress = progress
        }
    }

    private fun cancel() {
        cancelled = true
        uploadFuture.cancel(true)
        minecraft?.setScreen(parent)
    }

    private fun onUploadComplete(worldInfo: WorldInfo) {
        Loggers.Ui.get().info("World upload complete: {}", worldInfo.slug)
        onComplete(worldInfo)
        minecraft?.setScreen(parent)
    }

    private fun onUploadFailed(error: Throwable) {
        Loggers.Ui.get().error(error, "World upload failed")
        currentProgress = UploadProgress.failed(error.cause?.message ?: error.message)
    }
}
