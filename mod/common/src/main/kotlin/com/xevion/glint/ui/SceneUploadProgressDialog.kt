package com.xevion.glint.ui

import com.xevion.glint.Loggers
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.SceneSyncManager
import com.xevion.glint.api.SyncStatus
import com.xevion.glint.api.UploadResult
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
import net.minecraft.client.Minecraft
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import java.util.concurrent.CompletableFuture
import net.minecraft.network.chat.Component as McComponent

/**
 * Dialog showing scene upload progress with a progress bar and status.
 * Manages the upload lifecycle: initiating → uploading → finalizing → complete/failed.
 */
class SceneUploadProgressDialog(
    private val parentScreen: Screen,
    private val slug: String,
    private val sceneName: String,
    private val config: ApiConfig,
    private val syncStatus: SyncStatus? = null,
    private val onComplete: () -> Unit,
) : GlintDialogScreen(McComponent.literal("Upload Scene")) {
    private var bytesUploaded: Long = 0
    private var totalBytes: Long = 0
    private var isComplete = false
    private var error: String? = null
    private var cancelled = false
    private var uploadFuture: CompletableFuture<UploadResult>? = null

    private lateinit var statusLabel: LabelComponent
    private lateinit var progressBar: BoxComponent
    private lateinit var progressText: LabelComponent
    private lateinit var bytesLabel: LabelComponent

    override fun buildDialog(dialog: FlowLayout) {
        if (isComplete) {
            buildCompleteView(dialog)
        } else if (error != null) {
            buildErrorView(dialog)
        } else {
            buildProgressView(dialog)
        }
    }

    private fun buildProgressView(dialog: FlowLayout) {
        // Title
        dialog.child(
            Components
                .label(McComponent.literal("Uploading: $sceneName"))
                .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)),
        )

        // Progress container
        val progressContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
        progressContainer.horizontalAlignment(HorizontalAlignment.CENTER)
        progressContainer.gap(GlintTheme.GAP_SM)

        // Progress bar
        val barContainer = Containers.stack(Sizing.fixed(200), Sizing.fixed(20))

        val progressBarBackground = Components.box(Sizing.fill(100), Sizing.fill(100))
        progressBarBackground.color(Color.ofRgb(GlintTheme.PROGRESS_BG))
        progressBarBackground.fill(true)

        progressBar = Components.box(Sizing.fill(0), Sizing.fill(100))
        progressBar.color(Color.ofRgb(GlintTheme.PROGRESS_FILL))
        progressBar.fill(true)

        barContainer.child(progressBarBackground)
        barContainer.child(progressBar)

        progressContainer.child(barContainer)

        // Progress percentage
        progressText = Components.label(McComponent.literal(""))
        progressText.color(Color.ofRgb(GlintTheme.TEXT_PRIMARY))
        progressContainer.child(progressText)

        // Bytes label
        bytesLabel = Components.label(McComponent.literal(""))
        bytesLabel.color(Color.ofRgb(GlintTheme.TEXT_SECONDARY))
        progressContainer.child(bytesLabel)

        dialog.child(progressContainer)

        // Status label
        statusLabel = Components.label(McComponent.literal("Initiating upload..."))
        statusLabel.color(Color.ofRgb(GlintTheme.TEXT_WARNING))
        statusLabel.margins(Insets.top(GlintTheme.GAP_SM))
        dialog.child(statusLabel)

        // Cancel button
        dialog.child(
            GlintComponents.button(CommonComponents.GUI_CANCEL) { cancel() } as Component,
        )

        // Start upload if not already running
        if (uploadFuture == null) {
            startUpload()
        }
    }

    private fun buildCompleteView(dialog: FlowLayout) {
        dialog.child(GlintComponents.title(McComponent.literal("Upload Complete")))
        dialog.child(
            Components
                .label(McComponent.literal("Scene '$sceneName' uploaded successfully"))
                .color(Color.ofRgb(GlintTheme.TEXT_SUCCESS)),
        )
        dialog.child(
            GlintComponents.button(McComponent.literal("Close")) {
                onComplete()
                minecraft?.setScreen(parentScreen)
            } as Component,
        )
    }

    private fun buildErrorView(dialog: FlowLayout) {
        dialog.child(GlintComponents.title(McComponent.literal("Upload Failed")))
        dialog.child(
            Components
                .label(McComponent.literal(error ?: "Unknown error"))
                .color(Color.ofRgb(GlintTheme.TEXT_ERROR)),
        )
        dialog.child(
            GlintComponents.buttonRow(
                GlintComponents.button(McComponent.literal("Retry")) {
                    error = null
                    isComplete = false
                    bytesUploaded = 0
                    totalBytes = 0
                    uploadFuture = null
                    rebuild()
                },
                GlintComponents.cancelButton { minecraft?.setScreen(parentScreen) },
            ),
        )
    }

    private fun startUpload() {
        uploadFuture =
            SceneSyncManager.uploadScene(slug, config, syncStatus) { uploaded, total ->
                minecraft?.execute {
                    bytesUploaded = uploaded
                    totalBytes = total
                }
            }

        uploadFuture?.thenAcceptAsync(
            { result ->
                if (cancelled) return@thenAcceptAsync
                when (result) {
                    is UploadResult.Success -> {
                        isComplete = true
                        log.info("Scene upload complete") {
                            "slug" to slug
                            "sceneId" to result.sceneId
                        }
                        rebuild()
                    }

                    is UploadResult.Failure -> {
                        error = result.error.message
                        log.error("Scene upload failed") {
                            "slug" to slug
                            "error" to result.error.message
                        }
                        rebuild()
                    }
                }
            },
            Minecraft.getInstance(),
        )
    }

    override fun tick() {
        super.tick()
        if (!isComplete && error == null && ::statusLabel.isInitialized) {
            updateProgressDisplay()
        }
    }

    private fun updateProgressDisplay() {
        if (totalBytes > 0) {
            val percent = ((bytesUploaded * 100) / totalBytes).toInt().coerceIn(0, 100)
            progressBar.horizontalSizing(Sizing.fill(percent))
            progressText.text(McComponent.literal("$percent%"))

            val mbUploaded = "%.1f".format(bytesUploaded / (1024.0 * 1024.0))
            val mbTotal = "%.1f".format(totalBytes / (1024.0 * 1024.0))
            bytesLabel.text(McComponent.literal("$mbUploaded MB / $mbTotal MB"))
            statusLabel.text(McComponent.literal("Uploading scene package..."))
        } else {
            statusLabel.text(McComponent.literal("Initiating upload..."))
        }
    }

    private fun cancel() {
        cancelled = true
        uploadFuture?.cancel(true)
        minecraft?.setScreen(parentScreen)
    }

    /** Rebuild the dialog UI to reflect new state (complete, error, or progress). */
    private fun rebuild() {
        uiAdapter?.rootComponent?.let { root ->
            root.clearChildren()
            build(root)
        }
    }

    companion object {
        private val log = Loggers.Ui.get()
    }
}
