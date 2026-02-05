package com.xevion.glint.capture

import com.xevion.glint.Loggers
import com.xevion.glint.debug
import com.xevion.glint.error
import com.xevion.glint.info
import com.xevion.glint.mixin.ShaderPackAccessor
import com.xevion.glint.warn
import net.irisshaders.iris.Iris
import java.nio.file.Path

/**
 * Handles integration with Iris Shaders mod.
 * Uses compile-time Iris API with runtime availability checks for type-safe integration.
 */
object IrisIntegration {
    private val log = Loggers.Capture.get()
    private val available: Boolean =
        runCatching {
            Class.forName("net.irisshaders.iris.Iris")
            log.info("Iris integration enabled")
            true
        }.getOrElse { e ->
            when (e) {
                is ClassNotFoundException -> log.debug("Iris not detected, shader features disabled")
                is NoClassDefFoundError -> log.warn(e, "Iris detected but failed to load")
            }
            false
        }

    val isAvailable: Boolean get() = available

    fun isShaderPackInUse(): Result<Boolean> {
        if (!available) return Result.success(false)
        return runCatching {
            Iris.getCurrentPack().isPresent
        }.onFailure { e ->
            log.error(e, "Error checking shader pack status")
        }
    }

    fun getShaderPackName(): Result<String?> {
        if (!available) return Result.success(null)
        return runCatching {
            Iris.getCurrentPackName()
        }.onFailure { e ->
            log.error(e, "Error getting shader pack name")
        }
    }

    fun getShaderpacksDirectory(): Result<Path?> {
        if (!available) return Result.success(null)
        return runCatching {
            Iris.getShaderpacksDirectory()
        }.onFailure { e ->
            log.error(e, "Error getting shaderpacks directory")
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
            log.warn(e, "Failed to list shader packs")
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
            log.info("Shader pack changed") { "pack" to (packName ?: "(disabled)") }
        }.onFailure { e ->
            log.error(e, "Failed to set shader pack") { "pack" to packName }
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
            log.error(e, "Error getting shader profiles")
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
                    log.warn("Profile not found") { "profile" to profileName }
                    throw IllegalArgumentException("Profile not found: $profileName")
                }

            Iris.queueShaderPackOptionsFromProfile(profile as net.irisshaders.iris.shaderpack.option.Profile)

            val queue = Iris.getShaderPackOptionQueue()
            if (queue.isEmpty()) {
                log.warn("Profile queued but option queue is empty") { "profile" to profileName }
            }

            Iris.reload()
            log.info("Applied shader profile") { "profile" to profileName }
        }.onFailure { e ->
            log.error(e, "Failed to apply shader profile") { "profile" to profileName }
        }
    }

    fun resetShaderOptions(): Result<Unit> {
        if (!available || !isShaderPackInUse().getOrDefault(false)) {
            return Result.failure(IllegalStateException("Iris not available or no shader pack loaded"))
        }

        return runCatching {
            Iris.getShaderPackOptionQueue().clear()
            Iris.reload()
            log.info("Reset shader options to defaults")
        }.onFailure { e ->
            log.error(e, "Failed to reset shader options")
        }
    }
}
