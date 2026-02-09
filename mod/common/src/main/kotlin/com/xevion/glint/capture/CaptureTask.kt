package com.xevion.glint.capture

import com.xevion.glint.Loggers
import net.minecraft.Util
import net.minecraft.client.Minecraft
import net.minecraft.client.Screenshot
import java.nio.file.Path
import java.util.concurrent.CompletableFuture

/**
 * Manages a single high-resolution screenshot capture across two render frames.
 *
 * Frame sequence:
 * - Frame 0: Optionally hide HUD (framebuffer is already at capture resolution from session start)
 * - Frame 1: Read back framebuffer pixels and write image asynchronously
 *
 * No settle frames are needed because the StabilizationDetector in CaptureSession
 * already ensures chunks, lighting, and rendering are fully stable before capture begins,
 * and the framebuffer has been at 4K since session start (no per-capture resize).
 */
class CaptureTask(
    private val file: Path,
    private val hideHud: Boolean,
    private val future: CompletableFuture<Path>,
) {
    private val log = Loggers.Capture.get()
    private var originalHudState = false
    private var hudHidden = false

    /**
     * Advances the capture by one frame.
     * @return true when capture is complete and the task should be removed
     */
    fun onRenderTick(): Boolean {
        val mc = Minecraft.getInstance()

        if (!hudHidden) {
            originalHudState = mc.options.hideGui
            if (hideHud) {
                mc.options.hideGui = true
            }
            hudHidden = true
            return false
        }

        capture(mc)
        mc.options.hideGui = originalHudState
        return true
    }

    private fun capture(mc: Minecraft) {
        log.info("Capturing high-res screenshot") {
            "file" to file.fileName.toString()
        }

        val image = Screenshot.takeScreenshot(mc.mainRenderTarget)

        Util.ioPool().execute {
            try {
                image.use { nativeImage ->
                    WebpWriter.write(nativeImage, file)
                    future.complete(file)
                }
            } catch (e: Exception) {
                log.error("Failed to save high-res screenshot") {
                    "exception" to e.javaClass.simpleName
                    "error" to (e.message ?: "no message")
                    "stackTrace" to e.stackTraceToString()
                }
                future.completeExceptionally(e)
            }
        }
    }
}
