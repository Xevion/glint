package com.xevion.glint.capture

import com.xevion.glint.Loggers
import net.minecraft.client.GraphicsStatus
import net.minecraft.client.Minecraft
import java.nio.file.Path
import java.util.concurrent.CompletableFuture

/**
 * Orchestrates high-resolution screenshot capture by faking a window resize.
 *
 * The approach mimics what Minecraft does in `resizeDisplay()`:
 * 1. Set Window.framebufferWidth/Height fields to the capture resolution
 * 2. Recalculate GUI scale from the new dimensions
 * 3. Resize the main render target (GPU framebuffer) to match
 * 4. Notify GameRenderer/LevelRenderer of the new size
 * 5. Wait for the GPU to produce fully-rendered frames at the new resolution
 * 6. Read back framebuffer pixels and write to disk
 * 7. Restore all state to the actual window dimensions
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
    private var savedFramebufferWidth: Int = 0
    private var savedFramebufferHeight: Int = 0

    /**
     * Starts a high-resolution capture. The capture completes asynchronously over several
     * render frames. Returns a future that completes when the file is written to disk.
     *
     * @param file Output path for the PNG file
     * @param hideHud Whether to hide the HUD during capture
     * @return Future that completes when the capture is saved, or null if a capture is already active
     */
    fun startCapture(
        file: Path,
        hideHud: Boolean = true,
    ): CompletableFuture<Path>? {
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

        val mc = Minecraft.getInstance()

        // Force Fancy graphics mode during capture. In Fabulous mode, Minecraft routes
        // block entities (enchanting table books, chests, etc.) to a separate entityTarget
        // framebuffer that we don't resize. Fancy mode renders everything to mainRenderTarget.
        // Iris shaders override the graphics pipeline regardless, so this doesn't affect
        // shader quality.
        val currentMode = mc.options.graphicsMode().get()
        if (currentMode == GraphicsStatus.FABULOUS) {
            savedGraphicsMode = currentMode
            mc.options.graphicsMode().set(GraphicsStatus.FANCY)
            log.debug("Switched from Fabulous to Fancy for capture")
        }

        applyCaptureDimensions(mc)

        return future
    }

    /**
     * Called from GameRendererMixin after each frame render.
     * Advances the active capture task through its frame sequence.
     */
    fun onPostRender() {
        val task = activeTask ?: return

        if (task.onRenderTick()) {
            activeTask = null
            restoreAll()
        }
    }

    /** Whether a high-resolution capture is currently active. */
    fun isCapturing(): Boolean = activeTask != null

    /**
     * Fake a window resize to the capture resolution. Mirrors what Minecraft.resizeDisplay()
     * does, but with our target dimensions instead of the actual GLFW framebuffer size.
     */
    private fun applyCaptureDimensions(mc: Minecraft) {
        val window = mc.window

        // Save the real framebuffer dimensions so we can restore them later.
        // Use the actual field values (not getWidth/getHeight which we may be spoofing).
        savedFramebufferWidth = window.width
        savedFramebufferHeight = window.height
        log.debug("Saved real framebuffer dimensions") {
            "saved" to "${savedFramebufferWidth}x$savedFramebufferHeight"
        }

        // Set the Window's framebufferWidth/framebufferHeight fields to our capture resolution.
        // This is the critical difference from our previous approach: anything that reads
        // these fields directly (Sodium, Iris, calculateScale, setGuiScale) will now see
        // the capture resolution instead of the real window size.
        window.setWidth(CAPTURE_WIDTH)
        window.setHeight(CAPTURE_HEIGHT)

        // Recalculate GUI scale from the new dimensions, same as resizeDisplay()
        val guiScale = window.calculateScale(mc.options.guiScale().get(), mc.isEnforceUnicode)
        window.setGuiScale(guiScale.toDouble())

        // Resize the GPU framebuffer to actually allocate 4K textures
        mc.mainRenderTarget?.resize(CAPTURE_WIDTH, CAPTURE_HEIGHT)

        // Notify GameRenderer → LevelRenderer so viewport/frustum are updated
        mc.gameRenderer.resize(CAPTURE_WIDTH, CAPTURE_HEIGHT)

        log.info("Applied capture dimensions") {
            "resolution" to "${CAPTURE_WIDTH}x${CAPTURE_HEIGHT}"
            "guiScale" to guiScale.toString()
        }
    }

    /** Restore all state to the real window dimensions. */
    private fun restoreAll() {
        val mc = Minecraft.getInstance()
        val window = mc.window

        // Restore the real framebuffer dimensions
        window.setWidth(savedFramebufferWidth)
        window.setHeight(savedFramebufferHeight)

        // Recalculate GUI scale from real dimensions
        val guiScale = window.calculateScale(mc.options.guiScale().get(), mc.isEnforceUnicode)
        window.setGuiScale(guiScale.toDouble())

        // Restore GPU framebuffer to window size
        mc.mainRenderTarget?.resize(savedFramebufferWidth, savedFramebufferHeight)

        // Notify GameRenderer/LevelRenderer
        mc.gameRenderer.resize(savedFramebufferWidth, savedFramebufferHeight)

        log.info("Restored real dimensions") {
            "resolution" to "${savedFramebufferWidth}x$savedFramebufferHeight"
            "guiScale" to guiScale.toString()
        }

        // Restore graphics mode if we changed it
        savedGraphicsMode?.let { mode ->
            mc.options.graphicsMode().set(mode)
            log.debug("Restored graphics mode") { "mode" to mode }
            savedGraphicsMode = null
        }
    }
}
