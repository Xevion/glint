package com.xevion.glint.input

import com.xevion.glint.Glint
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.ui.ApiConfigWizardScreen
import com.xevion.glint.ui.GlintHubScreen
import net.minecraft.client.Minecraft
import org.lwjgl.glfw.GLFW

/**
 * Handles keybindings for Glint.
 *
 * Keybindings:
 * - J - Open Glint menu
 *
 * All other actions are accessible via the Glint menu (J key).
 */
object KeybindHandler {
    private val inputTracker = InputStateTracker()

    private val keybinds =
        listOf(
            Keybind(
                key = GLFW.GLFW_KEY_J,
                action = { openSceneManager() },
                description = "Open Glint menu",
            ),
        )

    /**
     * Called every client tick to poll key states.
     * Must be registered with the platform's tick event system.
     */
    fun onTick() {
        val mc = Minecraft.getInstance()

        if (mc.screen != null) {
            inputTracker.reset()
            return
        }

        keybinds.forEach { keybind ->
            val pressed = isKeyPressed(keybind.key)
            if (inputTracker.wasJustPressed(keybind.key, pressed)) {
                keybind.action()
            }
        }
    }

    private fun openSceneManager() {
        val mc = Minecraft.getInstance()
        val config = ApiConfig.load()

        if (config.needsValidation()) {
            Glint.LOGGER.info("Opening API configuration (needs validation)")
            val hub = GlintHubScreen(null)
            mc.setScreen(ApiConfigWizardScreen(hub, showConnectionFirst = true))
        } else {
            Glint.LOGGER.info("Opening Glint Hub")
            mc.setScreen(GlintHubScreen(null))
        }
    }

    private fun isKeyPressed(glfwKey: Int): Boolean {
        val window = Minecraft.getInstance().window.window
        return GLFW.glfwGetKey(window, glfwKey) == GLFW.GLFW_PRESS
    }

    private data class Keybind(
        val key: Int,
        val action: () -> Unit,
        val description: String,
    )

    /**
     * Tracks pressed/released state for edge detection.
     */
    private class InputStateTracker {
        private val previousState = mutableMapOf<Int, Boolean>()

        fun wasJustPressed(
            key: Int,
            currentlyPressed: Boolean,
        ): Boolean {
            val was = previousState[key] ?: false
            previousState[key] = currentlyPressed
            return currentlyPressed && !was
        }

        fun reset() {
            previousState.clear()
        }
    }
}
