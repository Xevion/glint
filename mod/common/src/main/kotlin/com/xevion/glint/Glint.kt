package com.xevion.glint

import com.xevion.glint.api.ApiConfig
import com.xevion.glint.api.UrlValidation
import com.xevion.glint.input.KeybindHandler
import com.xevion.glint.io.AsyncFileIO
import com.xevion.glint.orchestration.AutonomousRunner
import com.xevion.glint.session.SessionRegistry
import java.util.concurrent.CompletableFuture

object Glint {
    const val MOD_ID = "glint"
    private val log = Loggers.Core.get()

    /**
     * Whether the mod was launched by the agent in autonomous capture mode.
     * Set via the GLINT_AUTONOMOUS environment variable.
     */
    var isAutonomous: Boolean = false
        private set

    var apiUrl: String = ""
        private set
    var apiToken: String = ""
        private set

    /** Force mode selectors from GLINT_FORCE_SCENES / GLINT_FORCE_SHADERS env vars. */
    var forceScenes: String? = null
        private set
    var forceShaders: String? = null
        private set
    val isForceMode: Boolean get() = forceScenes != null || forceShaders != null

    var shaderLimit: Int = 10
        private set

    var autonomousRunner: AutonomousRunner? = null

    fun init() {
        // Trigger Loggers companion init — configures all category loggers via LogConfig
        Loggers.getAllCategories()
        log.info("Initializing Glint mod")

        isAutonomous = System.getenv("GLINT_AUTONOMOUS")?.equals("true", ignoreCase = true) == true
        if (isAutonomous) {
            log.info("Autonomous mode enabled - will auto-start orchestration on title screen")
            apiUrl = System.getenv("GLINT_API_URL") ?: "http://localhost:8080"
            apiToken = System.getenv("GLINT_API_TOKEN") ?: ""

            // Read force mode selectors
            forceScenes = System.getenv("GLINT_FORCE_SCENES")
            forceShaders = System.getenv("GLINT_FORCE_SHADERS")
            if (isForceMode) {
                log.info("Force mode enabled") {
                    "scenes" to (forceScenes ?: "+")
                    "shaders" to (forceShaders ?: "+")
                }
            }

            shaderLimit = System.getenv("GLINT_SHADER_LIMIT")?.toIntOrNull() ?: 10

            // Fall back to saved config token if env var is empty
            if (apiToken.isBlank()) {
                val config = ApiConfig.load()
                if (config.hasValidToken()) {
                    log.info("Using saved access token from config")
                    apiToken = config.accessToken
                    if (config.apiUrl.isNotBlank()) {
                        apiUrl = config.apiUrl
                    }
                } else {
                    log.error("GLINT_API_TOKEN is required in autonomous mode (env or config)")
                }
            }
        }

        validateApiConfig()
        registerShutdownHook()
    }

    fun onClientTick() {
        SessionRegistry.tick()
        autonomousRunner?.let { runner ->
            runner.tick()
            if (!runner.isRunning) {
                autonomousRunner = null
            }
        }
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
        log.info("Shutting down Glint mod")
        AsyncFileIO.shutdown()
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

        log.info("Validating API connection") { "url" to config.apiUrl }

        CompletableFuture
            .supplyAsync {
                UrlValidation.testConnection(config.apiUrl, token = config.accessToken)
            }.thenAccept { result ->
                result
                    .onSuccess {
                        log.info("API connection validated successfully")
                        val updatedConfig = config.copy(validated = true)
                        ApiConfig.save(updatedConfig)
                    }.onFailure { error ->
                        log.warn("API connection validation failed") { "error" to error.message }
                        val updatedConfig = config.copy(validated = false)
                        ApiConfig.save(updatedConfig)
                    }
            }
    }
}
