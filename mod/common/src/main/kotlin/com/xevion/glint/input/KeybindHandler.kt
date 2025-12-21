package com.xevion.glint.input

import com.xevion.glint.Glint
import com.xevion.glint.capture.CaptureSession
import com.xevion.glint.scene.SceneManager
import net.minecraft.client.Minecraft
import org.lwjgl.glfw.GLFW

/**
 * Handles keybindings for Glint.
 *
 * Keybindings:
 * - GRAVE (`) - Start multi-shader capture session
 * - F8 - Save current state as scene JSON (copies to clipboard and logs to console)
 */
object KeybindHandler {
    private var wasGravePressed = false
    private var wasF8Pressed = false
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
            wasF8Pressed = false
            return
        }

        val gravePressed = isKeyPressed(GLFW.GLFW_KEY_GRAVE_ACCENT)
        val f8Pressed = isKeyPressed(GLFW.GLFW_KEY_F8)

        if (gravePressed && !wasGravePressed) {
            startCaptureSession()
        }

        if (f8Pressed && !wasF8Pressed) {
            saveCurrentScene()
        }

        wasGravePressed = gravePressed
        wasF8Pressed = f8Pressed
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

    private fun saveCurrentScene() {
        val sceneJson = SceneManager.saveCurrentStateAsScene("captured_scene")
        if (sceneJson != null) {
            // Copy to clipboard
            Minecraft.getInstance().keyboardHandler.clipboard = sceneJson

            // Log to console
            Glint.LOGGER.info("Scene JSON saved to clipboard and logged below:")
            Glint.LOGGER.info("\n$sceneJson")
        } else {
            Glint.LOGGER.error("Failed to save scene - player or level is null")
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
