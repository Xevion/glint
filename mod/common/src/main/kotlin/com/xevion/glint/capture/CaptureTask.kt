package com.xevion.glint.capture

import com.xevion.glint.Loggers
import com.xevion.glint.mixin.NativeImageAccessor
import net.minecraft.Util
import net.minecraft.client.Minecraft
import net.minecraft.client.Screenshot
import org.lwjgl.system.MemoryUtil
import java.nio.file.Path
import java.util.concurrent.CompletableFuture

/**
 * Manages a single high-resolution screenshot capture across two render frames.
 *
 * Frame sequence:
 * - Frame 0: Optionally hide HUD (framebuffer is already at capture resolution from session start)
 * - Frame 1: Read back framebuffer pixels and write image asynchronously
 *
 * No settle frames are needed because the StabilizationDetector in LinearOrchestrator
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

    /** Future that completes with image analytics when pixel data is analyzed. Created eagerly so callers can hold a reference before capture() runs. */
    val analysisFuture: CompletableFuture<CaptureAnalysis> = CompletableFuture()

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

        // Copy pixel data from native memory while NativeImage is alive, then close it immediately.
        // ABGR→ARGB conversion is done once here; argbPixels is shared by both IO tasks below.
        val imgWidth: Int
        val imgHeight: Int
        val argbPixels: IntArray
        image.use { img ->
            imgWidth = img.width
            imgHeight = img.height
            val pixelCount = imgWidth * imgHeight
            val pixelPtr = (img as NativeImageAccessor).`glint$getPixels`()
            val intBuffer = MemoryUtil.memIntBuffer(pixelPtr, pixelCount)
            argbPixels = IntArray(pixelCount)
            for (i in 0 until pixelCount) {
                val abgr = intBuffer.get(i)
                val a = abgr and 0xFF000000.toInt()
                val b = (abgr shr 16) and 0xFF
                val g = abgr and 0x0000FF00.toInt()
                val r = (abgr and 0xFF) shl 16
                argbPixels[i] = a or r or g or b
            }
        }

        // Start analysis asynchronously — pixel data is now safely copied
        CompletableFuture
            .supplyAsync(
                { CaptureAnalyzer.analyze(argbPixels, imgWidth, imgHeight) },
                Util.ioPool(),
            ).whenComplete { result, error ->
                if (error != null) {
                    analysisFuture.completeExceptionally(error)
                } else {
                    analysisFuture.complete(result)
                }
            }

        Util.ioPool().execute {
            try {
                WebpWriter.write(argbPixels, imgWidth, imgHeight, file)
                future.complete(file)
            } catch (e: Throwable) {
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
