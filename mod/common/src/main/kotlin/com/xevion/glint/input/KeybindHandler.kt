package com.xevion.glint.input

import com.xevion.glint.Loggers
import com.xevion.glint.api.ApiConfig
import com.xevion.glint.ui.ApiConfigWizardScreen
import com.xevion.glint.ui.GlintMainScreen
import com.xevion.glint.ui.SceneSetupScreen
import net.minecraft.client.Minecraft
import org.lwjgl.glfw.GLFW

/**
 * Handles keybindings for Glint.
 *
 * Keybindings:
 * - J - Open Glint menu
 * - K - Open Scene Setup (environment composition)
 *
 * All other actions are accessible via the Glint menu (J key).
 */
object KeybindHandler {
    private val log = Loggers.Input.get()
    private val inputTracker = InputStateTracker()

    private val keybinds =
        listOf(
            Keybind(
                key = GLFW.GLFW_KEY_J,
                action = { openSceneManager() },
                description = "Open Glint menu",
            ),
            Keybind(
                key = GLFW.GLFW_KEY_K,
                action = { openSceneSetup() },
                description = "Open Scene Setup",
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
            log.debug("Opening API configuration (needs validation)")
            val hub = GlintMainScreen(null)
            mc.setScreen(ApiConfigWizardScreen(hub, showConnectionFirst = true))
        } else {
            log.debug("Opening Glint Hub")
            mc.setScreen(GlintMainScreen(null))
        }
    }

    private fun openSceneSetup() {
        val mc = Minecraft.getInstance()
        if (mc.level == null) return
        log.debug("Opening Scene Setup")
        mc.setScreen(SceneSetupScreen())
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
