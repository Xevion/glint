package com.xevion.glint.input

import com.xevion.glint.Glint
import com.xevion.glint.capture.CaptureSession
import net.minecraft.client.Minecraft
import org.lwjgl.glfw.GLFW

/**
 * Handles keybindings for Glint.
 *
 * Keybindings:
 * - GRAVE (`) - Start multi-shader capture session
 */
object KeybindHandler {
    private var wasGravePressed = false
    private var captureSession: CaptureSession? = null

    /**
     * Called every client tick to poll key states and advance capture session.
     * Must be registered with the platform's tick event system.
     */
    fun onTick() {
        val mc = Minecraft.getInstance()

        // Always tick the capture session if running
        captureSession?.tick()

        // Clean up completed sessions
        if (captureSession?.isRunning == false) {
            captureSession = null
        }

        // Don't process keybinds if a screen is open or not in-game
        if (mc.screen != null || mc.level == null) {
            wasGravePressed = false
            return
        }

        val gravePressed = isKeyPressed(GLFW.GLFW_KEY_GRAVE_ACCENT)

        if (gravePressed && !wasGravePressed) {
            startCaptureSession()
        }

        wasGravePressed = gravePressed
    }

    private fun startCaptureSession() {
        if (captureSession?.isRunning == true) {
            Glint.LOGGER.warn("Capture session already in progress")
            return
        }

        captureSession = CaptureSession()
        if (!captureSession!!.start()) {
            captureSession = null
        }
    }

    /**
     * Check if a key is currently pressed.
     */
    private fun isKeyPressed(glfwKey: Int): Boolean {
        val window = Minecraft.getInstance().window.window
        return GLFW.glfwGetKey(window, glfwKey) == GLFW.GLFW_PRESS
    }
}
