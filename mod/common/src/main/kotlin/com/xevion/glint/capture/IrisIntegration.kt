package com.xevion.glint.capture

import com.xevion.glint.Glint
import com.xevion.glint.screenshot.ShaderInfo
import java.util.Optional

/**
 * Handles integration with Iris Shaders mod.
 * Uses reflection to access Iris API since it's only available at runtime.
 */
object IrisIntegration {
    private const val IRIS_API_CLASS = "net.irisshaders.iris.api.v0.IrisApi"
    private const val IRIS_MAIN_CLASS = "net.irisshaders.iris.Iris"

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
     * Gets the current shader pack info, or null if no shader is active.
     */
    fun getShaderInfo(): ShaderInfo? {
        if (!isAvailable) return null

        return try {
            if (!isShaderPackInUse()) return null

            val packName = getShaderPackName()
            if (packName == null) {
                Glint.LOGGER.warn("Shader pack is in use but name could not be retrieved")
                return null
            }

            ShaderInfo(
                pack = packName,
                enabled = true,
            )
        } catch (e: Exception) {
            Glint.LOGGER.warn("Failed to get Iris shader info", e)
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
}
