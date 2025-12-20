package com.xevion.glint.capture

import com.xevion.glint.Glint
import com.xevion.glint.screenshot.*
import kotlinx.serialization.json.Json
import net.minecraft.client.Minecraft
import net.minecraft.world.level.chunk.EmptyLevelChunk
import java.io.File
import java.time.Instant
import java.time.format.DateTimeFormatter

/**
 * Manages a multi-shader screenshot capture session.
 *
 * Workflow:
 * 1. Start session - captures current shader state
 * 2. For each shader (including vanilla):
 *    a. Load shader (or disable for vanilla)
 *    b. Wait for stabilization (chunks, FPS)
 *    c. Take screenshot
 *    d. Wait a few ticks
 * 3. Restore original shader state
 */
class CaptureSession(
    private val sceneId: String = "default",
) {
    private var state: State = State.Idle
    private var ticksInState: Int = 0

    private var originalShaderPack: String? = null
    private var shadersToCapture: List<ShaderConfig> = emptyList()
    private var currentIndex: Int = 0
    private var sessionDir: File? = null
    private var sessionId: String = ""
    private var startedAt: Instant? = null
    private val screenshotEntries: MutableList<ScreenshotEntry> = mutableListOf()

    private val stabilizationDetector = StabilizationDetector()

    /**
     * Represents a shader configuration to capture (shader pack + optional profile).
     * Localized to CaptureSession - not intended for external use.
     */
    private data class ShaderConfig(
        val packName: String?,
        val profile: String? = null,
    ) {
        val displayName: String
            get() =
                when {
                    packName == null -> "Vanilla"
                    profile != null -> "$packName ($profile)"
                    else -> packName
                }
    }

    /**
     * Starts a new capture session.
     * @return true if session started, false if already running or Iris unavailable
     */
    fun start(): Boolean {
        if (state != State.Idle) {
            Glint.LOGGER.warn("Capture session already in progress")
            return false
        }

        // Create session output directory
        val mc = Minecraft.getInstance()
        startedAt = Instant.now()
        sessionId =
            DateTimeFormatter
                .ofPattern("yyyy-MM-dd_HH-mm-ss")
                .withZone(java.time.ZoneId.systemDefault())
                .format(startedAt)
        sessionDir = File(mc.gameDirectory, "glint_captures/$sessionId")
        if (!sessionDir!!.mkdirs()) {
            Glint.LOGGER.error("Failed to create capture session directory: ${sessionDir!!.absolutePath}")
            return false
        }
        Glint.LOGGER.info("Capture session directory: ${sessionDir!!.absolutePath}")
        screenshotEntries.clear()

        if (!IrisIntegration.isAvailable) {
            Glint.LOGGER.warn("Iris is not available, capturing vanilla only")
            shadersToCapture = listOf(ShaderConfig(packName = null))
            originalShaderPack = null
        } else {
            originalShaderPack =
                if (IrisIntegration.isShaderPackInUse().getOrDefault(false)) {
                    IrisIntegration.getShaderPackName().getOrNull()
                } else {
                    null
                }

            val availablePacks = IrisIntegration.listAvailableShaderPacks().getOrDefault(emptyList())

            shadersToCapture =
                buildList {
                    add(ShaderConfig(packName = null))

                    for (pack in availablePacks) {
                        if (IrisIntegration.enableShaders(pack).isSuccess) {
                            val profiles = IrisIntegration.getShaderProfiles().getOrDefault(emptyList())

                            if (profiles.isEmpty()) {
                                add(ShaderConfig(packName = pack))
                            } else {
                                for (profile in profiles) {
                                    add(ShaderConfig(packName = pack, profile = profile))
                                }
                            }
                        } else {
                            Glint.LOGGER.warn("Failed to load shader pack for profile discovery: $pack")
                        }
                    }
                }

            Glint.LOGGER.info(
                "Starting capture session with ${shadersToCapture.size} configurations: " +
                    shadersToCapture.joinToString(", ") { it.displayName },
            )
        }

        currentIndex = 0
        transitionTo(State.LoadingShader)
        return true
    }

    /**
     * Called every client tick to advance the session state machine.
     */
    fun tick() {
        if (state == State.Idle) return

        ticksInState++

        when (state) {
            State.Idle -> {}

            State.LoadingShader -> {
                handleLoadingShader()
            }

            State.WaitingForStabilization -> {
                handleWaitingForStabilization()
            }

            State.Capturing -> {
                handleCapturing()
            }

            State.PostCaptureCooldown -> {
                handlePostCaptureCooldown()
            }

            State.Finishing -> {
                handleFinishing()
            }
        }
    }

    /**
     * Whether the session is currently running.
     */
    val isRunning: Boolean
        get() = state != State.Idle

    private fun transitionTo(newState: State) {
        Glint.LOGGER.debug("Capture session: $state -> $newState")
        state = newState
        ticksInState = 0

        if (newState == State.WaitingForStabilization) {
            stabilizationDetector.reset()
        }
    }

    private fun handleLoadingShader() {
        val config = shadersToCapture.getOrNull(currentIndex)
        if (config == null) {
            Glint.LOGGER.error("Invalid shader config at index $currentIndex")
            advanceToNextShader()
            return
        }

        if (IrisIntegration.isAvailable) {
            Glint.LOGGER.info("Loading shader configuration: ${config.displayName}")

            val result =
                if (config.packName == null) {
                    IrisIntegration.disableShaders()
                } else {
                    IrisIntegration.enableShaders(config.packName).onFailure {
                        Glint.LOGGER.error("Failed to load shader pack: ${config.packName}")
                        advanceToNextShader()
                        return
                    }

                    if (config.profile != null) {
                        IrisIntegration.applyShaderProfile(config.profile).onFailure {
                            Glint.LOGGER.error("Failed to apply profile: ${config.profile}, skipping")
                            advanceToNextShader()
                            return
                        }
                    } else {
                        IrisIntegration.resetShaderOptions()
                    }

                    Result.success(Unit)
                }

            if (result.isFailure) {
                Glint.LOGGER.error("Failed to load shader configuration: ${config.displayName}, skipping")
                advanceToNextShader()
                return
            }
        }

        // Force all chunks to rebuild - Iris may not immediately mark sections dirty
        Minecraft.getInstance().levelRenderer.allChanged()

        transitionTo(State.WaitingForStabilization)
    }

    private fun handleWaitingForStabilization() {
        if (stabilizationDetector.isStable(ticksInState)) {
            transitionTo(State.Capturing)
        }
    }

    private fun handleCapturing() {
        val mc = Minecraft.getInstance()
        val renderTarget = mc.mainRenderTarget

        val config = shadersToCapture.getOrNull(currentIndex)
        if (config == null) {
            Glint.LOGGER.error("Invalid shader config at index $currentIndex")
            advanceToNextShader()
            return
        }

        val screenshotName = buildScreenshotFilename(config)
        Glint.LOGGER.info("Capturing screenshot for: ${config.displayName} -> $screenshotName")

        val timestamp = Instant.now().toString()
        val shaderInfo = config.packName?.let { parseShaderPackName(it) }
        val shaderMeta =
            if (config.packName != null && shaderInfo != null) {
                ShaderMetadata(
                    packFile = config.packName,
                    id = shaderInfo.id,
                    version = shaderInfo.version,
                    profile = config.profile,
                )
            } else {
                null
            }

        screenshotEntries.add(
            ScreenshotEntry(
                file = screenshotName,
                timestamp = timestamp,
                shader = shaderMeta,
                resolution =
                    Resolution(
                        width = renderTarget.width,
                        height = renderTarget.height,
                    ),
            ),
        )

        // Take the screenshot using vanilla Screenshot API, saving to our session directory
        net.minecraft.client.Screenshot.grab(
            sessionDir!!,
            screenshotName,
            renderTarget,
        ) { message ->
            Glint.LOGGER.info("Screenshot saved: ${message.string}")
        }

        transitionTo(State.PostCaptureCooldown)
    }

    /**
     * Builds a screenshot filename from the shader configuration.
     *
     * Expected shader pack format: `<shader-id>_<version>_mc<mc-version>.zip`
     * Output format: `<shader-id>_<version>_<profile>_<scene-id>.png`
     *
     * For vanilla: `vanilla_<scene-id>.png`
     */
    private fun buildScreenshotFilename(config: ShaderConfig): String {
        if (config.packName == null) {
            return "vanilla_$sceneId.png"
        }

        val shaderInfo = parseShaderPackName(config.packName)
        val profileSuffix = config.profile?.let { "_${sanitizeForFilename(it)}" } ?: ""
        return "${shaderInfo.id}_${shaderInfo.version}${profileSuffix}_$sceneId.png"
    }

    /**
     * Parses a shader pack filename into its components.
     *
     * Expected format: `<shader-id>_<version>_mc<mc-version>.zip`
     * Falls back to using sanitized full name if parsing fails.
     */
    private fun parseShaderPackName(filename: String): ShaderPackInfo {
        // Strip .zip extension if present
        val baseName = filename.removeSuffix(".zip").removeSuffix(".ZIP")

        // Expected format: shader-id_version_mcX.Y.Z
        // Example: bsl_v10.0_mc1.21.4
        val parts = baseName.split("_")

        return if (parts.size >= 3 && parts.last().startsWith("mc")) {
            // Successfully parsed: id_version_mcX.Y.Z
            val mcVersion = parts.last()
            val version = parts[parts.size - 2]
            val id = parts.dropLast(2).joinToString("_")
            ShaderPackInfo(
                id = sanitizeForFilename(id),
                version = sanitizeForFilename(version),
                mcVersion = mcVersion.removePrefix("mc"),
            )
        } else if (parts.size >= 2) {
            // Partial parse: id_version (no mc version)
            val version = parts.last()
            val id = parts.dropLast(1).joinToString("_")
            ShaderPackInfo(
                id = sanitizeForFilename(id),
                version = sanitizeForFilename(version),
                mcVersion = null,
            )
        } else {
            // Fallback: use whole name as id
            ShaderPackInfo(
                id = sanitizeForFilename(baseName),
                version = "unknown",
                mcVersion = null,
            )
        }
    }

    private fun sanitizeForFilename(input: String): String = input.lowercase().replace(Regex("[^a-z0-9._-]"), "-")

    private data class ShaderPackInfo(
        val id: String,
        val version: String,
        val mcVersion: String?,
    )

    private fun handlePostCaptureCooldown() {
        if (ticksInState >= POST_CAPTURE_COOLDOWN_TICKS) {
            advanceToNextShader()
        }
    }

    private fun advanceToNextShader() {
        currentIndex++
        if (currentIndex >= shadersToCapture.size) {
            transitionTo(State.Finishing)
        } else {
            transitionTo(State.LoadingShader)
        }
    }

    private fun handleFinishing() {
        Glint.LOGGER.info("Capture session complete, restoring original shader state")

        // Write session manifest
        writeSessionManifest()

        // Restore original shader
        if (IrisIntegration.isAvailable) {
            val shaderPack = originalShaderPack
            if (shaderPack != null) {
                IrisIntegration.enableShaders(shaderPack)
            } else {
                IrisIntegration.disableShaders()
            }
        }

        // Reset state
        state = State.Idle
        ticksInState = 0
        currentIndex = 0
        shadersToCapture = emptyList()
        originalShaderPack = null
        screenshotEntries.clear()
        sessionId = ""
        startedAt = null

        Glint.LOGGER.info("Capture session finished")
    }

    private fun writeSessionManifest() {
        val mc = Minecraft.getInstance()
        val completedAt = Instant.now()

        // Get player position and camera
        val player = mc.player
        val position =
            player?.let {
                Position(x = it.x, y = it.y, z = it.z)
            }
        val camera =
            player?.let {
                Camera(yaw = it.yRot, pitch = it.xRot)
            }

        // Get dimension
        val dimension =
            mc.level
                ?.dimension()
                ?.location()
                ?.toString()

        val manifest =
            SessionManifest(
                session =
                    SessionInfo(
                        id = sessionId,
                        sceneId = sceneId,
                        startedAt = startedAt.toString(),
                        completedAt = completedAt.toString(),
                        totalScreenshots = screenshotEntries.size,
                        shaderPacks = shadersToCapture.mapNotNull { it.packName }.distinct(),
                    ),
                minecraft =
                    MinecraftInfo(
                        version = mc.launchedVersion,
                        dimension = dimension,
                        position = position,
                        camera = camera,
                    ),
                screenshots = screenshotEntries.toList(),
            )

        val manifestFile = File(sessionDir!!, "session.json")
        try {
            manifestFile.writeText(JSON.encodeToString(SessionManifest.serializer(), manifest))
            Glint.LOGGER.info("Session manifest written to: ${manifestFile.absolutePath}")
        } catch (e: Exception) {
            Glint.LOGGER.error("Failed to write session manifest", e)
        }
    }

    private enum class State {
        Idle,
        LoadingShader,
        WaitingForStabilization,
        Capturing,
        PostCaptureCooldown,
        Finishing,
    }

    companion object {
        private val JSON = Json { prettyPrint = true }
        private const val POST_CAPTURE_COOLDOWN_TICKS = 10
    }
}
