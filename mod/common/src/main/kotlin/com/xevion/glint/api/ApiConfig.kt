package com.xevion.glint.api

import com.xevion.glint.Glint
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
) {
    companion object {
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
                Glint.LOGGER.info("API config not found, using defaults")
                return ApiConfig()
            }

            return try {
                val content = configFile.readText()
                JSON.decodeFromString(serializer(), content)
            } catch (e: Exception) {
                Glint.LOGGER.error("Failed to load API config, using defaults", e)
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
                Glint.LOGGER.info("Saved API config")
                true
            } catch (e: Exception) {
                Glint.LOGGER.error("Failed to save API config", e)
                false
            }
        }
    }

    /**
     * Checks if the config is valid for API operations.
     */
    fun isValid(): Boolean = enabled && apiUrl.isNotBlank() && worldId.isNotBlank() && validated

    /**
     * Checks if the config needs validation (has URL but not validated).
     */
    fun needsValidation(): Boolean = apiUrl.isNotBlank() && !validated
}
