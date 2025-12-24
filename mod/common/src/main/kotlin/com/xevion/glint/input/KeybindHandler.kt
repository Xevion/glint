package com.xevion.glint.input

import com.xevion.glint.Glint
import com.xevion.glint.capture.CaptureSession
import com.xevion.glint.orchestration.AutonomousOrchestrator
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.ui.SceneManagerScreen
import net.minecraft.client.Minecraft
import org.lwjgl.glfw.GLFW

/**
 * Handles keybindings for Glint.
 *
 * Keybindings:
 * - GRAVE (`) - Start multi-shader capture session
 * - F8 - Save current state as scene JSON (copies to clipboard and logs to console)
 * - F9 - Start autonomous orchestration (for testing)
 * - J - Open scene manager
 */
object KeybindHandler {
    private var wasGravePressed = false
    private var wasF8Pressed = false
    private var wasF9Pressed = false
    private var wasJPressed = false
    private var captureSession: CaptureSession? = null
    private var autonomousOrchestrator: AutonomousOrchestrator? = null

    /**
     * Called every client tick to poll key states and advance capture session.
     * Must be registered with the platform's tick event system.
     */
    fun onTick() {
        val mc = Minecraft.getInstance()

        // Always tick active sessions
        captureSession?.tick()
        autonomousOrchestrator?.tick()

        // Clean up completed sessions
        if (captureSession?.isRunning == false) {
            captureSession = null
        }
        if (autonomousOrchestrator?.isRunning == false) {
            autonomousOrchestrator = null
        }

        // Don't process keybinds if a screen is open or not in-game
        if (mc.screen != null || mc.level == null) {
            wasGravePressed = false
            wasF8Pressed = false
            wasF9Pressed = false
            wasJPressed = false
            return
        }

        val gravePressed = isKeyPressed(GLFW.GLFW_KEY_GRAVE_ACCENT)
        val f8Pressed = isKeyPressed(GLFW.GLFW_KEY_F8)
        val f9Pressed = isKeyPressed(GLFW.GLFW_KEY_F9)
        val jPressed = isKeyPressed(GLFW.GLFW_KEY_J)

        if (gravePressed && !wasGravePressed) {
            startCaptureSession()
        }

        if (f8Pressed && !wasF8Pressed) {
            saveCurrentScene()
        }

        if (f9Pressed && !wasF9Pressed) {
            startAutonomousOrchestration()
        }

        if (jPressed && !wasJPressed) {
            openSceneManager()
        }

        wasGravePressed = gravePressed
        wasF8Pressed = f8Pressed
        wasF9Pressed = f9Pressed
        wasJPressed = jPressed
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

    private fun startAutonomousOrchestration() {
        if (autonomousOrchestrator?.isRunning == true) {
            Glint.LOGGER.warn("Autonomous orchestration already in progress")
            return
        }

        Glint.LOGGER.info("Starting autonomous orchestration (manual trigger)")
        autonomousOrchestrator = AutonomousOrchestrator()
        if (!autonomousOrchestrator!!.start()) {
            autonomousOrchestrator = null
        }
    }

    private fun openSceneManager() {
        val mc = Minecraft.getInstance()
        mc.setScreen(SceneManagerScreen(null))
        Glint.LOGGER.info("Opening Scene Manager")
    }

    /**
     * Check if a key is currently pressed.
     */
    private fun isKeyPressed(glfwKey: Int): Boolean {
        val window = Minecraft.getInstance().window.window
        return GLFW.glfwGetKey(window, glfwKey) == GLFW.GLFW_PRESS
    }
}
