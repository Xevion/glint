package com.xevion.glint.api

import com.xevion.glint.Loggers
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import net.minecraft.client.Minecraft
import java.io.File

/**
 * Configuration for Glint backend API connection.
 * Stored in .minecraft/glint/config.json
 */
@Serializable
data class ApiConfig(
    val apiUrl: String = "http://localhost:8080",
    val worldId: String = "",
    val worldName: String = "",
    val enabled: Boolean = false,
    val validated: Boolean = false,
    val accessToken: String = "",
    val tokenExpiresAt: Long = 0L,
) {
    companion object {
        private val log = Loggers.Api.get()

        private val JSON =
            Json {
                prettyPrint = true
                ignoreUnknownKeys = true
            }

        private fun getConfigFile(): File {
            val mc = Minecraft.getInstance()
            val glintDir = File(mc.gameDirectory, "glint")
            glintDir.mkdirs()
            return File(glintDir, "config.json")
        }

        /**
         * Loads the API config from disk.
         * Returns default config if file doesn't exist.
         */
        fun load(): ApiConfig {
            val configFile = getConfigFile()

            if (!configFile.exists()) {
                log.debug("API config not found, using defaults")
                return ApiConfig()
            }

            return try {
                val content = configFile.readText()
                JSON.decodeFromString(serializer(), content)
            } catch (e: Exception) {
                log.error(e, "Failed to load API config, using defaults")
                ApiConfig()
            }
        }

        /**
         * Saves the API config to disk.
         */
        fun save(config: ApiConfig): Boolean {
            val configFile = getConfigFile()

            return try {
                val content = JSON.encodeToString(serializer(), config)
                configFile.writeText(content)
                log.debug("Saved API config")
                true
            } catch (e: Exception) {
                log.error(e, "Failed to save API config")
                false
            }
        }
    }

    /**
     * Checks if the config is valid for API operations.
     */
    fun isValid(): Boolean = enabled && apiUrl.isNotBlank() && worldId.isNotBlank() && validated && hasValidToken()

    /**
     * Checks if the config needs validation (has URL but not validated).
     */
    fun needsValidation(): Boolean = apiUrl.isNotBlank() && !validated

    /**
     * Checks if the access token is present and not expired.
     */
    fun hasValidToken(): Boolean = accessToken.isNotBlank() && tokenExpiresAt > System.currentTimeMillis()

    /**
     * Checks if the token is close to expiring (within 1 hour).
     */
    fun isTokenExpiringSoon(): Boolean = accessToken.isNotBlank() && tokenExpiresAt - System.currentTimeMillis() < 3600_000L
}
