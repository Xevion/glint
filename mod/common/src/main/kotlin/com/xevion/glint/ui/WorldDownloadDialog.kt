package com.xevion.glint.ui

import com.xevion.glint.Glint
import com.xevion.glint.download.DownloadProgress
import net.minecraft.client.gui.GuiGraphics
import net.minecraft.client.gui.components.Button
import net.minecraft.client.gui.screens.Screen
import net.minecraft.network.chat.CommonComponents
import net.minecraft.network.chat.Component
import java.util.concurrent.CompletableFuture

/**
 * Dialog showing world download progress.
 */
class WorldDownloadDialog(
    private val parent: Screen,
    private val worldName: String,
    private val downloadFuture: CompletableFuture<String>,
) : Screen(Component.literal("Downloading World")) {
    private var currentProgress: DownloadProgress = DownloadProgress.downloading(0, 0)
    private var cancelled = false

    override fun init() {
        val centerX = width / 2
        val buttonY = height / 2 + 60

        addRenderableWidget(
            Button
                .builder(CommonComponents.GUI_CANCEL) { cancel() }
                .bounds(centerX - 50, buttonY, 100, 20)
                .build(),
        )

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

    override fun render(
        guiGraphics: GuiGraphics,
        mouseX: Int,
        mouseY: Int,
        delta: Float,
    ) {
        renderBackground(guiGraphics, mouseX, mouseY, delta)
        super.render(guiGraphics, mouseX, mouseY, delta)

        val centerX = width / 2
        val centerY = height / 2

        guiGraphics.drawCenteredString(
            font,
            "Downloading: $worldName",
            centerX,
            centerY - 40,
            0xFFFFFF,
        )

        when (currentProgress.state) {
            DownloadProgress.State.DOWNLOADING -> {
                val progress = currentProgress.percentComplete
                val barWidth = 200
                val barHeight = 20
                val barX = centerX - barWidth / 2
                val barY = centerY - 10

                guiGraphics.fill(barX, barY, barX + barWidth, barY + barHeight, 0xFF333333.toInt())

                val fillWidth = (barWidth * progress / 100).coerceIn(0, barWidth)
                guiGraphics.fill(barX, barY, barX + fillWidth, barY + barHeight, 0xFF00AA00.toInt())

                guiGraphics.drawCenteredString(
                    font,
                    "$progress%",
                    centerX,
                    centerY - 5,
                    0xFFFFFF,
                )

                val mbDownloaded = currentProgress.bytesDownloaded / (1024 * 1024)
                val mbTotal = currentProgress.totalBytes / (1024 * 1024)
                guiGraphics.drawCenteredString(
                    font,
                    "$mbDownloaded MB / $mbTotal MB",
                    centerX,
                    centerY + 20,
                    0xAAAAAA,
                )
            }

            DownloadProgress.State.EXTRACTING -> {
                guiGraphics.drawCenteredString(
                    font,
                    "Extracting world files...",
                    centerX,
                    centerY,
                    0xFFAA00,
                )
            }

            DownloadProgress.State.COMPLETE -> {
            }

            DownloadProgress.State.FAILED -> {
                guiGraphics.drawCenteredString(
                    font,
                    "Download failed!",
                    centerX,
                    centerY,
                    0xFF0000,
                )
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
        Glint.LOGGER.info("World download complete: {}", worldPath)
        minecraft?.setScreen(parent)
    }

    private fun onDownloadFailed(error: Throwable) {
        Glint.LOGGER.error("World download failed", error)
        currentProgress = DownloadProgress.failed()
    }
}
