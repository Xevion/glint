package com.xevion.glint.capture

import com.xevion.glint.Loggers
import net.minecraft.client.GraphicsStatus
import net.minecraft.client.Minecraft
import java.nio.file.Path
import java.util.concurrent.CompletableFuture

/**
 * Orchestrates high-resolution screenshot capture by faking a window resize.
 *
 * Resolution management is session-scoped: dimensions are set to 4K once at session
 * start and restored at session end. This avoids per-capture resize cascades in Iris
 * (which would reset temporal accumulation buffers each time).
 *
 * The blit-to-screen step uses the real window dimensions (via [realFramebufferWidth]
 * and [realFramebufferHeight]) so the display shows a properly downscaled image instead
 * of a zoomed-in corner.
 *
 * Setting the actual Window fields (not just spoofing getter return values) is
 * critical because Sodium, Iris, and Minecraft's own calculateScale/setGuiScale
 * read framebufferWidth/framebufferHeight fields directly.
 */
object HighResCapture {
    private val log = Loggers.Capture.get()

    const val CAPTURE_WIDTH = 3840
    const val CAPTURE_HEIGHT = 2160

    private var activeTask: CaptureTask? = null
    private var savedGraphicsMode: GraphicsStatus? = null

    /** Real GLFW framebuffer dimensions, saved at session start. */
    var realFramebufferWidth: Int = 0
        private set
    var realFramebufferHeight: Int = 0
        private set

    private var sessionActive: Boolean = false

    /**
     * Begins a capture session by resizing to 4K and forcing Fancy graphics mode.
     *
     * Call once before the first capture. The resolution stays at 4K for the entire
     * session so that Iris pipelines are always created at the capture resolution.
     *
     * @return true if session started, false if one is already active
     */
    fun beginSession(): Boolean {
        if (sessionActive) {
            log.warn("Capture session already active, ignoring beginSession")
            return false
        }

        val mc = Minecraft.getInstance()

        // Force Fancy graphics mode. In Fabulous mode Minecraft routes block entities
        // to a separate entityTarget framebuffer that we don't resize.
        val currentMode = mc.options.graphicsMode().get()
        if (currentMode == GraphicsStatus.FABULOUS) {
            savedGraphicsMode = currentMode
            mc.options.graphicsMode().set(GraphicsStatus.FANCY)
            log.debug("Switched from Fabulous to Fancy for capture session")
        }

        applyCaptureDimensions(mc)
        FlawlessFrames.setEnabled(true)
        sessionActive = true

        log.info("Capture session begun") {
            "resolution" to "${CAPTURE_WIDTH}x${CAPTURE_HEIGHT}"
        }
        return true
    }

    /**
     * Ends the capture session and restores the real window dimensions and graphics mode.
     */
    fun endSession() {
        if (!sessionActive) {
            log.warn("No capture session active, ignoring endSession")
            return
        }

        activeTask = null
        FlawlessFrames.setEnabled(false)
        restoreDimensions()
        restoreGraphicsMode()
        sessionActive = false

        log.info("Capture session ended")
    }

    /** Whether a capture session is active (dimensions are at 4K). */
    fun isSessionActive(): Boolean = sessionActive

    /**
     * Starts a single high-resolution capture within an active session.
     *
     * The framebuffer is already at 4K from [beginSession], so no resize happens here.
     * The capture completes asynchronously over a couple of render frames.
     *
     * @param file Output path for the image file
     * @param hideHud Whether to hide the HUD during capture
     * @return Future that completes when the capture is saved, or null if a capture is already active
     */
    fun startCapture(
        file: Path,
        hideHud: Boolean = true,
    ): CompletableFuture<Path>? {
        if (!sessionActive) {
            log.error("Cannot start capture outside of an active session")
            return null
        }

        if (activeTask != null) {
            log.warn("High-res capture already in progress, ignoring request")
            return null
        }

        log.info("Starting high-res capture") {
            "target" to "${CAPTURE_WIDTH}x${CAPTURE_HEIGHT}"
            "file" to file.fileName.toString()
        }

        val future = CompletableFuture<Path>()
        activeTask = CaptureTask(file, hideHud, future)
        return future
    }

    /**
     * Called from GameRendererMixin after each frame render.
     * Advances the active capture task and the synthetic time clock.
     */
    fun onPostRender() {
        // Advance synthetic shader time after each rendered frame so the next
        // frame's Iris Timer.beginFrame() call sees an incremented timestamp.
        CaptureTimeOverride.advanceFrame()

        val task = activeTask ?: return

        if (task.onRenderTick()) {
            activeTask = null
        }
    }

    /** Whether a single capture is currently in-flight. */
    fun isCapturing(): Boolean = activeTask != null

    /**
     * Fake a window resize to the capture resolution. Mirrors what Minecraft.resizeDisplay()
     * does, but with our target dimensions instead of the actual GLFW framebuffer size.
     */
    private fun applyCaptureDimensions(mc: Minecraft) {
        val window = mc.window

        realFramebufferWidth = window.width
        realFramebufferHeight = window.height
        log.debug("Saved real framebuffer dimensions") {
            "saved" to "${realFramebufferWidth}x$realFramebufferHeight"
        }

        window.setWidth(CAPTURE_WIDTH)
        window.setHeight(CAPTURE_HEIGHT)

        val guiScale = window.calculateScale(mc.options.guiScale().get(), mc.isEnforceUnicode)
        window.setGuiScale(guiScale.toDouble())

        mc.mainRenderTarget?.resize(CAPTURE_WIDTH, CAPTURE_HEIGHT)
        mc.gameRenderer.resize(CAPTURE_WIDTH, CAPTURE_HEIGHT)

        log.info("Applied capture dimensions") {
            "resolution" to "${CAPTURE_WIDTH}x${CAPTURE_HEIGHT}"
            "guiScale" to guiScale.toString()
        }
    }

    /** Restore window dimensions to the real GLFW size. */
    private fun restoreDimensions() {
        val mc = Minecraft.getInstance()
        val window = mc.window

        window.setWidth(realFramebufferWidth)
        window.setHeight(realFramebufferHeight)

        val guiScale = window.calculateScale(mc.options.guiScale().get(), mc.isEnforceUnicode)
        window.setGuiScale(guiScale.toDouble())

        mc.mainRenderTarget?.resize(realFramebufferWidth, realFramebufferHeight)
        mc.gameRenderer.resize(realFramebufferWidth, realFramebufferHeight)

        log.info("Restored real dimensions") {
            "resolution" to "${realFramebufferWidth}x$realFramebufferHeight"
            "guiScale" to guiScale.toString()
        }
    }

    /** Restore graphics mode if we changed it at session start. */
    private fun restoreGraphicsMode() {
        savedGraphicsMode?.let { mode ->
            Minecraft
                .getInstance()
                .options
                .graphicsMode()
                .set(mode)
            log.debug("Restored graphics mode") { "mode" to mode }
            savedGraphicsMode = null
        }
    }
}
