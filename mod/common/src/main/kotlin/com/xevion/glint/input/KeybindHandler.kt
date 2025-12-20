package com.xevion.glint.input

import com.xevion.glint.Glint
import com.xevion.glint.capture.IrisIntegration
import com.xevion.glint.screenshot.ScreenshotHandler
import net.minecraft.client.Minecraft
import org.lwjgl.glfw.GLFW

/**
 * Handles keybindings for Glint.
 *
 * Keybindings:
 * - GRAVE (`) - Capture screenshot with metadata
 */
object KeybindHandler {
    private var wasGravePressed = false

    /**
     * Called every client tick to poll key states.
     * Must be registered with the platform's tick event system.
     */
    fun onTick() {
        val mc = Minecraft.getInstance()

        // Don't process keybinds if a screen is open or not in-game
        if (mc.screen != null || mc.level == null) {
            wasGravePressed = false
            return
        }

        val gravePressed = isKeyPressed(GLFW.GLFW_KEY_GRAVE_ACCENT)

        if (gravePressed && !wasGravePressed) {
            triggerDemoCapture()
        }

        wasGravePressed = gravePressed
    }

    private fun triggerDemoCapture() {
        val mc = Minecraft.getInstance()
        val renderTarget = mc.mainRenderTarget

        val shaderInfo = IrisIntegration.getShaderInfo()
        if (shaderInfo != null) {
            Glint.LOGGER.debug("Capturing with shader: ${shaderInfo.pack}")
        } else if (IrisIntegration.isAvailable) {
            Glint.LOGGER.debug("Capturing without shader (Iris available)")
        }

        net.minecraft.client.Screenshot.grab(
            mc.gameDirectory,
            null,
            renderTarget,
        ) { message ->
            Glint.LOGGER.info("${message.string}")
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
