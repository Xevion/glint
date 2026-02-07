package com.xevion.glint.capture

import com.xevion.glint.Loggers
import net.minecraft.Util
import net.minecraft.client.Minecraft
import net.minecraft.client.Screenshot
import java.nio.file.Path
import java.util.concurrent.CompletableFuture

/**
 * Manages a single high-resolution screenshot capture across multiple render frames.
 *
 * Frame sequence:
 * - Frame 0: Optionally hide HUD, framebuffer is already resized by HighResCapture
 * - Frames 1..SETTLE_FRAMES-1: Wait for GPU to produce fully-rendered frames at new resolution
 * - Frame SETTLE_FRAMES: Read back framebuffer pixels and write PNG asynchronously
 */
class CaptureTask(
    private val file: Path,
    private val hideHud: Boolean,
    private val future: CompletableFuture<Path>,
) {
    private val log = Loggers.Capture.get()
    private var originalHudState = false
    private var frame = 0

    /**
     * Advances the capture by one frame.
     * @return true when capture is complete and the task should be removed
     */
    fun onRenderTick(): Boolean {
        val mc = Minecraft.getInstance()

        when {
            frame == 0 -> {
                originalHudState = mc.options.hideGui
                if (hideHud) {
                    mc.options.hideGui = true
                }
                frame++
                return false
            }

            frame < SETTLE_FRAMES -> {
                frame++
                return false
            }

            else -> {
                capture(mc)
                mc.options.hideGui = originalHudState
                return true
            }
        }
    }

    private fun capture(mc: Minecraft) {
        log.info("Capturing high-res screenshot") {
            "file" to file.fileName.toString()
        }

        val image = Screenshot.takeScreenshot(mc.mainRenderTarget)

        Util.ioPool().execute {
            try {
                image.use { nativeImage ->
                    FramebufferWriter.write(nativeImage, file)
                    future.complete(file)
                }
            } catch (e: Exception) {
                log.error("Failed to save high-res screenshot") {
                    "error" to (e.message ?: "unknown")
                }
                future.completeExceptionally(e)
            }
        }
    }

    companion object {
        /**
         * Number of frames to wait before capturing. The framebuffer needs a few frames
         * after resize for the GPU to produce fully-rendered content at the new resolution.
         */
        private const val SETTLE_FRAMES = 10
    }
}
