package com.xevion.glint.capture

import com.xevion.glint.Glint
import com.xevion.glint.screenshot.Camera
import com.xevion.glint.screenshot.MinecraftInfo
import com.xevion.glint.screenshot.Position
import com.xevion.glint.screenshot.Resolution
import com.xevion.glint.screenshot.ScreenshotEntry
import com.xevion.glint.screenshot.SessionInfo
import com.xevion.glint.screenshot.SessionManifest
import com.xevion.glint.screenshot.ShaderMetadata
import kotlinx.serialization.json.Json
import net.minecraft.client.Minecraft
import java.io.File
import java.time.Instant
import java.time.LocalDateTime
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
    private var shadersToCapture: List<String?> = emptyList()
    private var currentIndex: Int = 0
    private var sessionDir: File? = null
    private var sessionId: String = ""
    private var startedAt: Instant? = null
    private val screenshotEntries: MutableList<ScreenshotEntry> = mutableListOf()

    private val json = Json { prettyPrint = true }

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
            shadersToCapture = listOf(null)
            originalShaderPack = null
        } else {
            originalShaderPack =
                if (IrisIntegration.isShaderPackInUse()) {
                    IrisIntegration.getShaderPackName()
                } else {
                    null
                }

            val availablePacks = IrisIntegration.listAvailableShaderPacks()
            // null represents vanilla (no shader)
            shadersToCapture = listOf(null) + availablePacks

            Glint.LOGGER.info(
                "Starting capture session with ${shadersToCapture.size} configurations: " +
                    "Vanilla + ${availablePacks.joinToString(", ")}",
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
    }

    private fun handleLoadingShader() {
        val targetShader = shadersToCapture.getOrNull(currentIndex)

        if (IrisIntegration.isAvailable) {
            val shaderName = targetShader ?: "(Vanilla)"
            Glint.LOGGER.info("Loading shader: $shaderName")

            val success =
                if (targetShader == null) {
                    IrisIntegration.disableShaders()
                } else {
                    IrisIntegration.enableShaders(targetShader)
                }

            if (!success) {
                Glint.LOGGER.error("Failed to load shader: $shaderName, skipping")
                advanceToNextShader()
                return
            }
        }

        transitionTo(State.WaitingForStabilization)
    }

    private fun handleWaitingForStabilization() {
        // Wait for chunks to load and FPS to stabilize
        // Simple heuristic: wait a fixed number of ticks
        // TODO: Could be smarter - check chunk loading status, FPS variance, etc.
        if (ticksInState >= STABILIZATION_TICKS) {
            val mc = Minecraft.getInstance()
            val chunksLoading = mc.level?.chunkSource?.loadedChunksCount ?: 0
            Glint.LOGGER.debug("Stabilization complete after $ticksInState ticks, chunks loaded: $chunksLoading")
            transitionTo(State.Capturing)
        }
    }

    private fun handleCapturing() {
        val mc = Minecraft.getInstance()
        val renderTarget = mc.mainRenderTarget

        val currentShader = shadersToCapture.getOrNull(currentIndex)
        val screenshotName = buildScreenshotFilename(currentShader)
        Glint.LOGGER.info("Capturing screenshot for: ${currentShader ?: "vanilla"} -> $screenshotName")

        // Collect metadata before capture
        val timestamp = Instant.now().toString()
        val parsed = currentShader?.let { parseShaderPackName(it) }
        val shaderMeta =
            if (currentShader != null && parsed != null) {
                ShaderMetadata(
                    packFile = currentShader,
                    id = parsed.id,
                    version = parsed.version,
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
     * Builds a screenshot filename from the shader pack name.
     *
     * Expected shader pack format: `<shader-id>_<version>_mc<mc-version>.zip`
     * Output format: `<shader-id>_<version>_<scene-id>.png`
     *
     * For vanilla: `vanilla_<scene-id>.png`
     */
    private fun buildScreenshotFilename(shaderPackName: String?): String {
        if (shaderPackName == null) {
            return "vanilla_$sceneId.png"
        }

        val parsed = parseShaderPackName(shaderPackName)
        return "${parsed.id}_${parsed.version}_$sceneId.png"
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
        // Wait a few ticks after capture before moving to next shader
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
            if (originalShaderPack != null) {
                IrisIntegration.enableShaders(originalShaderPack!!)
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
                        shaderPacks = shadersToCapture.filterNotNull(),
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
            manifestFile.writeText(json.encodeToString(SessionManifest.serializer(), manifest))
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
        // Number of ticks to wait for world/shader to stabilize (20 ticks = 1 second)
        private const val STABILIZATION_TICKS = 100 // 5 seconds

        // Number of ticks to wait after capturing before moving to next shader
        private const val POST_CAPTURE_COOLDOWN_TICKS = 10 // 0.5 seconds
    }
}
