package com.xevion.glint.capture

import com.xevion.glint.Glint
import com.xevion.glint.io.SessionDirectoryManager
import com.xevion.glint.scene.SceneApplicator
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.screenshot.*
import net.minecraft.client.Minecraft
import java.io.File
import java.time.Instant

/**
 * Manages a multi-shader screenshot capture session.
 *
 * Workflow:
 * 1. Start session - load and apply scene
 * 2. For each shader (including vanilla):
 *    a. Load shader (or disable for vanilla)
 *    b. Wait for stabilization (chunks, FPS)
 *    c. Take screenshot
 *    d. Wait a few ticks
 * 3. Restore original shader and world state
 */
class CaptureSession(
    private val sceneId: String? = null,
    private val outputDir: File? = null,
    private val worldName: String? = null,
    private val orchestrated: Boolean = false,
) {
    private val logger = Glint.LOGGER
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
    private var resolvedScene: com.xevion.glint.scene.ResolvedScene? = null
    private var sceneApplied: Boolean = false
    private var originalState: CaptureStateSnapshot? = null

    // Session data returned on completion (for autonomous orchestration)
    private var sessionData: CaptureSessionData? = null

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
     * @return true if session started, false if already running or scene/Iris unavailable
     */
    fun start(): Boolean {
        if (state != State.Idle) {
            logger.warn("Capture session already in progress")
            return false
        }

        val mc = Minecraft.getInstance()

        // Capture system only works in single-player
        if (mc.singleplayerServer == null) {
            logger.error("Capture system is not available in multiplayer")
            return false
        }

        // Load scene(s)
        if (sceneId != null) {
            // Single scene mode
            resolvedScene = SceneManager.loadScene(sceneId)
            if (resolvedScene == null) {
                logger.error("Failed to load scene: $sceneId")
                return false
            }
        } else {
            // All scenes mode - for now, just get the first scene from current world
            // TODO: Full multi-scene capture will iterate through all scenes
            val collections = SceneManager.discoverAllCollections()
            if (collections.isEmpty()) {
                logger.error("No scene collections found")
                return false
            }
            val (fileName, collection) = collections.first()
            val firstScene = collection.scenes.firstOrNull()
            if (firstScene == null) {
                logger.error("No scenes found in collection: $fileName")
                return false
            }
            resolvedScene = SceneManager.loadScene(firstScene.id)
            if (resolvedScene == null) {
                logger.error("Failed to load scene: ${firstScene.id}")
                return false
            }
            logger.info("Capture All mode: using first scene '${firstScene.id}' from '$fileName'")
        }

        // Create session output directory
        startedAt = Instant.now()
        sessionDir =
            outputDir?.also {
                // Orchestrator provides parent directory, Screenshot.grab creates screenshots/ subdirectory
                sessionId = it.name
                logger.info("Using provided output directory: ${it.absolutePath}")
            } ?: run {
                val capturesDir = File(mc.gameDirectory, "glint/captures")
                val (dir, id) = SessionDirectoryManager.createSessionDirectory(capturesDir, startedAt!!)
                sessionId = id
                logger.info("Created capture session directory: ${dir.absolutePath}")
                dir
            }
        screenshotEntries.clear()

        if (!IrisIntegration.isAvailable) {
            logger.warn("Iris is not available, capturing vanilla only")
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
                            logger.warn("Failed to load shader pack for profile discovery: $pack")
                        }
                    }
                }

            logger.info(
                "Starting capture session with ${shadersToCapture.size} configurations: " +
                    shadersToCapture.joinToString(", ") { it.displayName },
            )
        }

        currentIndex = 0

        // Save original client state and mark capture as active
        originalState = CaptureStateSnapshot.capture()
        CaptureStateManager.startCapture()
        logger.info("Original client state saved, capture mode activated")

        transitionTo(State.ApplyingScene)
        return true
    }

    /**
     * Called every client tick to advance the session state machine.
     */
    fun tick() {
        if (state == State.Idle) return

        // Check for cancellation request from ESC key
        if (CaptureStateManager.consumeCancelRequest()) {
            logger.info("Capture session cancelled by user")
            transitionTo(State.Finishing)
            return
        }

        ticksInState++

        when (state) {
            State.Idle -> {}

            State.ApplyingScene -> {
                handleApplyingScene()
            }

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

    /**
     * Get session data after completion (null if still running or not started).
     */
    fun getSessionData(): CaptureSessionData? = sessionData

    private fun transitionTo(newState: State) {
        logger.info("Capture session state: $state -> $newState")
        state = newState
        ticksInState = 0

        if (newState == State.WaitingForStabilization) {
            stabilizationDetector.reset()
        }

        if (newState == State.ApplyingScene) {
            sceneApplied = false
        }
    }

    private fun handleApplyingScene() {
        val scene = resolvedScene
        if (scene == null) {
            logger.error("Scene not loaded")
            transitionTo(State.Finishing)
            return
        }

        // Apply scene once when entering this state
        if (!sceneApplied) {
            logger.info("Applying scene: ${scene.scene.id}")
            if (!SceneApplicator.apply(scene)) {
                logger.error("Failed to apply scene")
                transitionTo(State.Finishing)
                return
            }
            sceneApplied = true
            logger.debug("Scene applied successfully")
        }

        // Wait for scene to stabilize (chunks loaded, player positioned)
        if (ticksInState >= SCENE_APPLICATION_WAIT_TICKS) {
            transitionTo(State.LoadingShader)
        }
    }

    private fun handleLoadingShader() {
        val config = shadersToCapture.getOrNull(currentIndex)
        if (config == null) {
            logger.error("Invalid shader config at index $currentIndex")
            advanceToNextShader()
            return
        }

        if (IrisIntegration.isAvailable) {
            logger.info("Loading shader configuration: ${config.displayName}")

            val result =
                if (config.packName == null) {
                    IrisIntegration.disableShaders()
                } else {
                    IrisIntegration.enableShaders(config.packName).onFailure {
                        logger.error("Failed to load shader pack: ${config.packName}")
                        advanceToNextShader()
                        return
                    }

                    if (config.profile != null) {
                        IrisIntegration.applyShaderProfile(config.profile).onFailure {
                            logger.error("Failed to apply profile: ${config.profile}, skipping")
                            advanceToNextShader()
                            return
                        }
                    } else {
                        IrisIntegration.resetShaderOptions()
                    }

                    Result.success(Unit)
                }

            if (result.isFailure) {
                logger.error("Failed to load shader configuration: ${config.displayName}, skipping")
                advanceToNextShader()
                return
            }
        }

        // Reapply scene settings to ensure consistency across shader switches
        // Shaders may modify render settings, camera, or other options
        val scene = resolvedScene
        if (scene != null) {
            logger.debug("Reapplying scene for shader: ${config.displayName}")
            if (!SceneApplicator.apply(scene)) {
                logger.warn("Failed to reapply scene for shader: ${config.displayName}")
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
            logger.error("Invalid shader config at index $currentIndex")
            advanceToNextShader()
            return
        }

        val screenshotName = buildScreenshotFilename(config)
        logger.info("Capturing screenshot for: ${config.displayName} -> $screenshotName")

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
            logger.info("Screenshot saved: ${message.string}")
        }

        transitionTo(State.PostCaptureCooldown)
    }

    /**
     * Builds a screenshot filename from the shader configuration.
     *
     * Expected shader pack format: `<shader-id>_<version>_mc<mc-version>.zip`
     * Output format: `<shader-id>_<version>_<profile>.png`
     *
     * For vanilla: `vanilla.png`
     */
    private fun buildScreenshotFilename(config: ShaderConfig): String {
        if (config.packName == null) {
            return "vanilla.png"
        }

        val shaderInfo = parseShaderPackName(config.packName)
        val profileSuffix = config.profile?.let { "_${sanitizeForFilename(it)}" } ?: ""
        return "${shaderInfo.id}_${shaderInfo.version}$profileSuffix.png"
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
        logger.info("Capture session complete, restoring original state")

        // Build session data for return
        sessionData = buildSessionData()

        // Restore original shader
        if (IrisIntegration.isAvailable) {
            originalShaderPack?.let { shaderPack ->
                IrisIntegration.enableShaders(shaderPack)
            } ?: IrisIntegration.disableShaders()
        }

        // Restore original client state (options, camera, etc.)
        originalState?.let {
            it.restore()
            logger.info("Original client state restored")
        }

        // End capture mode (skip if orchestrated - orchestrator manages capture state)
        if (!orchestrated) {
            CaptureStateManager.endCapture()
        }

        // Reset state (keep sessionData for retrieval)
        state = State.Idle
        ticksInState = 0
        currentIndex = 0
        shadersToCapture = emptyList()
        originalShaderPack = null
        screenshotEntries.clear()
        sessionId = ""
        startedAt = null
        originalState = null

        logger.info("Capture session finished")
    }

    private fun buildSessionData(): CaptureSessionData {
        val mc = Minecraft.getInstance()
        val completedAt = Instant.now()

        // Get player position and camera
        val player = mc.player
        val position = player?.let { Position(x = it.x, y = it.y, z = it.z) }
        val camera = player?.let { Camera(yaw = it.yRot, pitch = it.xRot) }

        // Get dimension
        val dimension =
            mc.level
                ?.dimension()
                ?.location()
                ?.toString()

        // Determine world name (provided by orchestrator or from current world)
        val resolvedWorldName = worldName ?: mc.singleplayerServer?.worldData?.levelName ?: "unknown"

        // Session directory path relative to game directory
        val sessionDirPath =
            sessionDir?.let {
                it.relativeTo(mc.gameDirectory).path
            } ?: "unknown"

        return CaptureSessionData(
            worldName = resolvedWorldName,
            sceneId = resolvedScene?.scene?.id ?: "unknown",
            sessionDir = sessionDirPath,
            startedAt = startedAt.toString(),
            completedAt = completedAt.toString(),
            totalScreenshots = screenshotEntries.size,
            shaderPacks = shadersToCapture.mapNotNull { it.packName }.distinct(),
            minecraft =
                MinecraftInfo(
                    version = mc.launchedVersion,
                    dimension = dimension,
                    position = position,
                    camera = camera,
                ),
            screenshots = screenshotEntries.toList(),
        )
    }

    private enum class State {
        Idle,
        ApplyingScene,
        LoadingShader,
        WaitingForStabilization,
        Capturing,
        PostCaptureCooldown,
        Finishing,
    }

    companion object {
        private const val POST_CAPTURE_COOLDOWN_TICKS = 10
        private const val SCENE_APPLICATION_WAIT_TICKS = 40
    }
}
