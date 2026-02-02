package com.xevion.glint

import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.GlintApi
import com.xevion.glint.input.KeybindHandler
import com.xevion.glint.session.SessionRegistry
import org.slf4j.LoggerFactory
import java.util.concurrent.CompletableFuture

object Glint {
    const val MOD_ID = "glint"
    val LOGGER = LoggerFactory.getLogger(MOD_ID)

    /**
     * Whether the mod was launched by the agent in autonomous capture mode.
     * Set via the GLINT_AUTONOMOUS environment variable.
     */
    var isAutonomous: Boolean = false
        private set

    fun init() {
        LOGGER.info("Initializing Glint mod")
        LogConfig.setupDebugLogging(MOD_ID)

        isAutonomous = System.getenv("GLINT_AUTONOMOUS")?.equals("true", ignoreCase = true) == true
        if (isAutonomous) {
            LOGGER.info("Autonomous mode enabled - will auto-start orchestration on title screen")
        }

        validateApiConfig()
        registerShutdownHook()
    }

    fun onClientTick() {
        SessionRegistry.tick()
        KeybindHandler.onTick()
    }

    private fun registerShutdownHook() {
        Runtime.getRuntime().addShutdownHook(
            Thread {
                onShutdown()
            },
        )
    }

    private fun onShutdown() {
        LOGGER.info("Shutting down Glint mod")
        com.xevion.glint.download.WorldDownloader
            .cleanupAllDownloads()
        com.xevion.glint.api.SceneSyncManager
            .shutdown()
    }

    /**
     * Validates API config on bootup if enabled and URL is set.
     * Updates validation status in config.
     */
    private fun validateApiConfig() {
        val config = ApiConfig.load()

        if (!config.enabled || config.apiUrl.isBlank()) {
            return
        }

        LOGGER.info("Validating API connection to {}", config.apiUrl)

        CompletableFuture
            .supplyAsync {
                GlintApi.testConnection(config.apiUrl)
            }.thenAccept { result ->
                result
                    .onSuccess {
                        LOGGER.info("API connection validated successfully")
                        val updatedConfig = config.copy(validated = true)
                        ApiConfig.save(updatedConfig)
                    }.onFailure { error ->
                        LOGGER.warn("API connection validation failed: {}", error.message)
                        val updatedConfig = config.copy(validated = false)
                        ApiConfig.save(updatedConfig)
                    }
            }
    }
}
