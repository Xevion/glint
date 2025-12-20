package com.xevion.glint.capture

import com.xevion.glint.Glint
import com.xevion.glint.mixin.ShaderPackAccessor
import net.irisshaders.iris.Iris
import java.nio.file.Path

/**
 * Handles integration with Iris Shaders mod.
 * Uses compile-time Iris API with runtime availability checks for type-safe integration.
 */
object IrisIntegration {
    private val available: Boolean =
        runCatching {
            Class.forName("net.irisshaders.iris.Iris")
            Glint.LOGGER.info("Iris integration enabled")
            true
        }.getOrElse { e ->
            when (e) {
                is ClassNotFoundException -> Glint.LOGGER.debug("Iris not detected, shader features disabled")
                is NoClassDefFoundError -> Glint.LOGGER.warn("Iris detected but failed to load", e)
            }
            false
        }

    val isAvailable: Boolean get() = available

    fun isShaderPackInUse(): Result<Boolean> {
        if (!available) return Result.success(false)
        return runCatching {
            Iris.getCurrentPack().isPresent
        }.onFailure { e ->
            Glint.LOGGER.error("Error checking shader pack status", e)
        }
    }

    fun getShaderPackName(): Result<String?> {
        if (!available) return Result.success(null)
        return runCatching {
            Iris.getCurrentPackName()
        }.onFailure { e ->
            Glint.LOGGER.error("Error getting shader pack name", e)
        }
    }

    fun getShaderpacksDirectory(): Result<Path?> {
        if (!available) return Result.success(null)
        return runCatching {
            Iris.getShaderpacksDirectory()
        }.onFailure { e ->
            Glint.LOGGER.error("Error getting shaderpacks directory", e)
        }
    }

    fun listAvailableShaderPacks(): Result<List<String>> {
        val shaderpacksDir = getShaderpacksDirectory().getOrNull() ?: return Result.success(emptyList())

        return runCatching {
            val dir = shaderpacksDir.toFile()
            if (!dir.exists() || !dir.isDirectory) return@runCatching emptyList()

            dir
                .listFiles()
                ?.filter { file ->
                    file.isDirectory || (file.isFile && file.extension.equals("zip", ignoreCase = true))
                }?.map { file ->
                    file.name
                }?.sorted()
                ?: emptyList()
        }.onFailure { e ->
            Glint.LOGGER.warn("Failed to list shader packs", e)
        }
    }

    private fun setShaderPack(packName: String?): Result<Unit> {
        if (!available) return Result.failure(IllegalStateException("Iris not available"))

        return runCatching {
            val config = Iris.getIrisConfig()
            config.setShaderPackName(packName ?: "")
            config.setShadersEnabled(packName != null)
            config.save()
            Iris.reload()
            Glint.LOGGER.info("Shader pack set to: ${packName ?: "(disabled)"}")
        }.onFailure { e ->
            Glint.LOGGER.error("Failed to set shader pack: $packName", e)
        }
    }

    fun disableShaders(): Result<Unit> = setShaderPack(null)

    fun enableShaders(packName: String): Result<Unit> = setShaderPack(packName)

    fun getShaderProfiles(): Result<List<String>> {
        if (!available || !isShaderPackInUse().getOrDefault(false)) {
            return Result.success(emptyList())
        }

        return runCatching {
            val pack = Iris.getCurrentPack().orElse(null) ?: return@runCatching emptyList()
            val menuContainer = (pack as ShaderPackAccessor).getMenuContainer()
            val profileSet = menuContainer.profiles

            val profileNames = mutableListOf<String>()
            profileSet.forEach { name, _ -> profileNames.add(name) }
            profileNames.toList()
        }.onFailure { e ->
            Glint.LOGGER.error("Error getting shader profiles", e)
        }
    }

    fun applyShaderProfile(profileName: String): Result<Unit> {
        if (!available || !isShaderPackInUse().getOrDefault(false)) {
            return Result.failure(IllegalStateException("Iris not available or no shader pack loaded"))
        }

        return runCatching {
            val pack =
                Iris.getCurrentPack().orElse(null)
                    ?: throw IllegalStateException("No shader pack loaded")

            val menuContainer = (pack as ShaderPackAccessor).getMenuContainer()
            val profileSet = menuContainer.profiles

            var foundProfile: Any? = null
            profileSet.forEach { name, p ->
                if (name == profileName) {
                    foundProfile = p
                }
            }

            val profile =
                foundProfile ?: run {
                    Glint.LOGGER.warn("Profile not found: $profileName")
                    throw IllegalArgumentException("Profile not found: $profileName")
                }

            Iris.queueShaderPackOptionsFromProfile(profile as net.irisshaders.iris.shaderpack.option.Profile)

            val queue = Iris.getShaderPackOptionQueue()
            if (queue.isEmpty()) {
                Glint.LOGGER.warn("Profile queued but option queue is empty: $profileName")
            }

            Iris.reload()
            Glint.LOGGER.info("Applied shader profile: $profileName")
        }.onFailure { e ->
            Glint.LOGGER.error("Failed to apply shader profile: $profileName", e)
        }
    }

    fun resetShaderOptions(): Result<Unit> {
        if (!available || !isShaderPackInUse().getOrDefault(false)) {
            return Result.failure(IllegalStateException("Iris not available or no shader pack loaded"))
        }

        return runCatching {
            Iris.getShaderPackOptionQueue().clear()
            Iris.reload()
            Glint.LOGGER.info("Reset shader options to defaults")
        }.onFailure { e ->
            Glint.LOGGER.error("Failed to reset shader options", e)
        }
    }
}
