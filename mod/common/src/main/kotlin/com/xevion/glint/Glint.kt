package com.xevion.glint

import com.xevion.glint.input.KeybindHandler
import com.xevion.glint.orchestration.AutonomousOrchestrator
import net.minecraft.client.Minecraft
import org.slf4j.Logger
import org.slf4j.LoggerFactory

object Glint {
    const val MOD_ID = "glint"
    val LOGGER: Logger = LoggerFactory.getLogger(MOD_ID)

    private var autonomousOrchestrator: AutonomousOrchestrator? = null
    private var autonomousStarted = false

    fun init() {
        LogConfig.setupDebugLogging(MOD_ID)
        LOGGER.info("Initializing Glint mod")

        // Check for autonomous mode
        val envAutonomous = System.getenv("GLINT_AUTONOMOUS")?.equals("true", ignoreCase = true) ?: false
        val jvmAutonomous = System.getProperty("glint.autonomous")?.equals("true", ignoreCase = true) ?: false

        if (envAutonomous || jvmAutonomous) {
            LOGGER.info("Autonomous mode enabled (env=$envAutonomous, jvm=$jvmAutonomous)")
            autonomousOrchestrator = AutonomousOrchestrator()
        }
    }

    fun onClientTick() {
        val orchestrator = autonomousOrchestrator
        if (orchestrator != null) {
            // In autonomous mode, wait for main menu to load before starting
            if (!autonomousStarted) {
                val mc = Minecraft.getInstance()
                if (mc.screen is net.minecraft.client.gui.screens.TitleScreen) {
                    LOGGER.info("Main menu loaded, starting autonomous orchestration")
                    autonomousStarted = true
                    orchestrator.start()
                }
            }

            orchestrator.tick()
        } else {
            // Normal interactive mode
            KeybindHandler.onTick()
        }
    }
}
