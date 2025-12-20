package com.xevion.glint.capture

import com.xevion.glint.Glint
import java.nio.file.Path

/**
 * Handles integration with Iris Shaders mod.
 * Uses reflection to access Iris API since it's only available at runtime.
 */
object IrisIntegration {
    private const val IRIS_API_CLASS = "net.irisshaders.iris.api.v0.IrisApi"
    private const val IRIS_MAIN_CLASS = "net.irisshaders.iris.Iris"
    private const val IRIS_CONFIG_CLASS = "net.irisshaders.iris.config.IrisConfig"

    private val irisApiClass: Class<*>? by lazy {
        try {
            Class.forName(IRIS_API_CLASS)
        } catch (e: ClassNotFoundException) {
            Glint.LOGGER.debug("Iris API class not found: $IRIS_API_CLASS")
            null
        }
    }

    private val irisMainClass: Class<*>? by lazy {
        try {
            Class.forName(IRIS_MAIN_CLASS)
        } catch (e: ClassNotFoundException) {
            Glint.LOGGER.debug("Iris main class not found: $IRIS_MAIN_CLASS")
            null
        }
    }

    /**
     * Whether Iris mod is loaded and available.
     */
    val isAvailable: Boolean
        get() = irisApiClass != null

    /**
     * Gets the IrisApi instance via reflection.
     */
    private fun getApiInstance(): Any? {
        val clazz = irisApiClass ?: return null
        return try {
            val getInstance = clazz.getMethod("getInstance")
            getInstance.invoke(null)
        } catch (e: Exception) {
            Glint.LOGGER.debug("Failed to get IrisApi instance", e)
            null
        }
    }

    /**
     * Gets the IrisConfig instance via Iris.getIrisConfig().
     */
    private fun getIrisConfig(): Any? {
        val mainClass = irisMainClass ?: return null
        return try {
            val method = mainClass.getMethod("getIrisConfig")
            method.invoke(null)
        } catch (e: Exception) {
            Glint.LOGGER.debug("Failed to get IrisConfig instance", e)
            null
        }
    }

    /**
     * Checks if a shader pack is currently in use.
     */
    fun isShaderPackInUse(): Boolean {
        if (!isAvailable) return false

        return try {
            val api = getApiInstance() ?: return false
            val method = api.javaClass.getMethod("isShaderPackInUse")
            method.invoke(api) as Boolean
        } catch (e: NoSuchMethodException) {
            Glint.LOGGER.error("IrisApi.isShaderPackInUse() method signature changed", e)
            false
        } catch (e: Exception) {
            Glint.LOGGER.warn("Failed to check if shader pack is in use", e)
            false
        }
    }

    /**
     * Gets the name of the current shader pack, if any.
     */
    fun getShaderPackName(): String? {
        val mainClass = irisMainClass ?: return null

        return try {
            if (!isShaderPackInUse()) return null

            val method = mainClass.getMethod("getCurrentPackName")
            method.invoke(null) as String
        } catch (e: NoSuchMethodException) {
            Glint.LOGGER.error("Iris.getCurrentPackName() method signature changed", e)
            null
        } catch (e: ClassCastException) {
            Glint.LOGGER.error("Iris.getCurrentPackName() return type changed", e)
            null
        } catch (e: Exception) {
            Glint.LOGGER.warn("Failed to get shader pack name", e)
            null
        }
    }

    /**
     * Gets the shaderpacks directory path.
     */
    fun getShaderpacksDirectory(): Path? {
        val mainClass = irisMainClass ?: return null

        return try {
            val method = mainClass.getMethod("getShaderpacksDirectory")
            method.invoke(null) as Path
        } catch (e: NoSuchMethodException) {
            Glint.LOGGER.error("Iris.getShaderpacksDirectory() method signature changed", e)
            null
        } catch (e: ClassCastException) {
            Glint.LOGGER.error("Iris.getShaderpacksDirectory() return type changed", e)
            null
        } catch (e: Exception) {
            Glint.LOGGER.warn("Failed to get shaderpacks directory", e)
            null
        }
    }

    /**
     * Lists all available shader pack names from the shaderpacks directory.
     * Returns folder names and .zip filenames (with extension, as Iris expects).
     */
    fun listAvailableShaderPacks(): List<String> {
        val shaderpacksDir = getShaderpacksDirectory() ?: return emptyList()

        return try {
            val dir = shaderpacksDir.toFile()
            if (!dir.exists() || !dir.isDirectory) return emptyList()

            dir
                .listFiles()
                ?.filter { file ->
                    file.isDirectory || (file.isFile && file.extension.equals("zip", ignoreCase = true))
                }?.map { file ->
                    // Iris expects full filename for zips, folder name for directories
                    file.name
                }?.sorted()
                ?: emptyList()
        } catch (e: Exception) {
            Glint.LOGGER.warn("Failed to list shader packs", e)
            emptyList()
        }
    }

    /**
     * Sets the shader pack and enables/disables shaders.
     * @param packName The shader pack name, or null to disable shaders
     * @return true if successful
     */
    fun setShaderPack(packName: String?): Boolean {
        val config = getIrisConfig() ?: return false
        val mainClass = irisMainClass ?: return false

        return try {
            // Set the pack name (empty string or null means internal/disabled)
            val setPackMethod = config.javaClass.getMethod("setShaderPackName", String::class.java)
            setPackMethod.invoke(config, packName ?: "")

            // Set shaders enabled state
            val setEnabledMethod = config.javaClass.getMethod("setShadersEnabled", Boolean::class.java)
            setEnabledMethod.invoke(config, packName != null)

            // Save config
            val saveMethod = config.javaClass.getMethod("save")
            saveMethod.invoke(config)

            // Reload shaders
            val reloadMethod = mainClass.getMethod("reload")
            reloadMethod.invoke(null)

            Glint.LOGGER.info("Shader pack set to: ${packName ?: "(disabled)"}")
            true
        } catch (e: NoSuchMethodException) {
            Glint.LOGGER.error("Iris config method signature changed", e)
            false
        } catch (e: Exception) {
            Glint.LOGGER.error("Failed to set shader pack", e)
            false
        }
    }

    /**
     * Disables shaders (switches to vanilla rendering).
     * @return true if successful
     */
    fun disableShaders(): Boolean = setShaderPack(null)

    /**
     * Enables shaders with a specific pack.
     * @return true if successful
     */
    fun enableShaders(packName: String): Boolean = setShaderPack(packName)
}
