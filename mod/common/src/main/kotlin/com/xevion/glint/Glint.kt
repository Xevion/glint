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

    fun init() {
        LOGGER.info("Initializing Glint mod")
        LogConfig.setupDebugLogging(MOD_ID)
        validateApiConfig()
    }

    fun onClientTick() {
        SessionRegistry.tick()
        KeybindHandler.onTick()
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
