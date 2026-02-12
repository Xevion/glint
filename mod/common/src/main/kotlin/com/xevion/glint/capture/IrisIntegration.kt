package com.xevion.glint.capture

import com.xevion.glint.Loggers
import com.xevion.glint.mixin.ShaderPackAccessor
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

    /**
     * Configures and reloads Iris with the given shader pack and optional profile.
     *
     * Batches config changes and profile options into a single [Iris.reload] call
     * when possible. If the pack is changing AND a profile is requested, two reloads
     * are needed (first to load the pack so its profile definitions are available,
     * then to apply the profile options).
     */
    fun enableShaders(
        packName: String,
        profile: String? = null,
    ): Result<Unit> {
        if (!available) return Result.failure(IllegalStateException("Iris not available"))

        return runCatching {
            val currentPack = Iris.getCurrentPackName()
            val samePack = currentPack == packName

            if (profile != null && samePack && Iris.getCurrentPack().isPresent) {
                // Same pack, just changing profile — queue options then single reload
                queueProfile(profile)
                Iris.reload()
                log.info("Shader profile changed") {
                    "pack" to packName
                    "profile" to profile
                }
            } else if (profile != null) {
                // Different pack with profile — need two reloads:
                // first to load pack, second to apply profile from that pack
                setShaderPackConfig(packName)
                Iris.getShaderPackOptionQueue().clear()
                Iris.reload()

                queueProfile(profile)
                Iris.reload()
                log.info("Shader pack changed with profile") {
                    "pack" to packName
                    "profile" to profile
                }
            } else {
                // No profile — single reload with cleared options
                setShaderPackConfig(packName)
                Iris.getShaderPackOptionQueue().clear()
                Iris.reload()
                log.info("Shader pack changed") { "pack" to packName }
            }
        }.onFailure { e ->
            log.error(e, "Failed to enable shaders") {
                "pack" to packName
                "profile" to profile
            }
        }
    }

    fun disableShaders(): Result<Unit> {
        if (!available) return Result.failure(IllegalStateException("Iris not available"))

        return runCatching {
            val config = Iris.getIrisConfig()
            config.setShaderPackName("")
            config.setShadersEnabled(false)
            config.save()
            Iris.reload()
            log.info("Shaders disabled")
        }.onFailure { e ->
            log.error(e, "Failed to disable shaders")
        }
    }

    private fun setShaderPackConfig(packName: String) {
        val config = Iris.getIrisConfig()
        config.setShaderPackName(packName)
        config.setShadersEnabled(true)
        config.save()
    }

    private fun queueProfile(profileName: String) {
        val pack =
            Iris.getCurrentPack().orElse(null)
                ?: error("No shader pack loaded, cannot apply profile")

        val menuContainer = (pack as ShaderPackAccessor).getMenuContainer()
        val profileSet = menuContainer.profiles

        var foundProfile: Any? = null
        profileSet.forEach { name, p ->
            if (name == profileName) {
                foundProfile = p
            }
        }

        val profile =
            foundProfile ?: throw IllegalArgumentException("Profile not found: $profileName")

        Iris.queueShaderPackOptionsFromProfile(profile as net.irisshaders.iris.shaderpack.option.Profile)

        val queue = Iris.getShaderPackOptionQueue()
        if (queue.isEmpty()) {
            log.warn("Profile queued but option queue is empty") { "profile" to profileName }
        }
    }

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
}
