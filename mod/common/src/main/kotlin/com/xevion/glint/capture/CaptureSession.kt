package com.xevion.glint.capture

import com.xevion.glint.Glint
import com.xevion.glint.orchestration.ShaderSpec
import com.xevion.glint.scene.SceneApplicator
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.screenshot.*
import net.minecraft.client.Minecraft
import java.io.File
import java.time.Instant

/**
 * Manages a multi-shader screenshot capture session for a single scene.
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
    private val sceneId: String,
    private val shaders: List<ShaderSpec>,
    private val outputDir: File,
    private val worldName: String,
) {
    private val logger = Glint.LOGGER
    private var state: State = State.Idle
    private var ticksInState: Int = 0

    private var originalShaderPack: String? = null
    private var shadersToCapture: List<ShaderConfig> = emptyList()
    private var currentIndex: Int = 0
    private var startedAt: Instant? = null
    private val screenshotEntries: MutableList<ScreenshotEntry> = mutableListOf()

    private val stabilizationDetector = StabilizationDetector()
    private var resolvedScene: com.xevion.glint.scene.ResolvedScene? = null
    private var sceneApplied: Boolean = false
    private var originalState: CaptureStateSnapshot? = null

    private var sessionData: CaptureSessionData? = null

    /**
     * Internal shader configuration (pack name + optional profile).
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

        if (mc.singleplayerServer == null) {
            logger.error("Capture system is not available in multiplayer")
            return false
        }

        resolvedScene = SceneManager.loadScene(sceneId)
        if (resolvedScene == null) {
            logger.error("Failed to load scene: $sceneId")
            return false
        }

        startedAt = Instant.now()
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

            shadersToCapture =
                shaders.map { spec ->
                    ShaderConfig(packName = spec.filename, profile = spec.profile)
                }

            logger.info(
                "Starting capture session with ${shadersToCapture.size} configurations: " +
                    shadersToCapture.joinToString(", ") { it.displayName },
            )
        }

        currentIndex = 0

        originalState = CaptureStateSnapshot.capture()
        logger.info("Original client state saved")

        transitionTo(State.ApplyingScene)
        return true
    }

    /**
     * Called every client tick to advance the session state machine.
     */
    fun tick() {
        if (state == State.Idle) return

        if (CaptureStateManager.consumeCancelRequest()) {
            logger.info("Capture session cancelled by user")
            transitionTo(State.Finishing)
            return
        }

        ticksInState++

        when (state) {
            State.Idle -> {}
            State.ApplyingScene -> handleApplyingScene()
            State.LoadingShader -> handleLoadingShader()
            State.WaitingForStabilization -> handleWaitingForStabilization()
            State.Capturing -> handleCapturing()
            State.PostCaptureCooldown -> handlePostCaptureCooldown()
            State.Finishing -> handleFinishing()
        }
    }

    val isRunning: Boolean
        get() = state != State.Idle

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

        val scene = resolvedScene
        if (scene != null) {
            logger.debug("Reapplying scene for shader: ${config.displayName}")
            if (!SceneApplicator.apply(scene)) {
                logger.warn("Failed to reapply scene for shader: ${config.displayName}")
            }
        }

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

        net.minecraft.client.Screenshot.grab(
            outputDir,
            screenshotName,
            renderTarget,
        ) { message ->
            logger.info("Screenshot saved: ${message.string}")
        }

        transitionTo(State.PostCaptureCooldown)
    }

    private fun buildScreenshotFilename(config: ShaderConfig): String {
        if (config.packName == null) {
            return "vanilla.png"
        }

        val shaderInfo = parseShaderPackName(config.packName)
        val profileSuffix = config.profile?.let { "_${sanitizeForFilename(it)}" } ?: ""
        return "${shaderInfo.id}_${shaderInfo.version}$profileSuffix.png"
    }

    private fun parseShaderPackName(filename: String): ShaderPackInfo {
        val baseName = filename.removeSuffix(".zip").removeSuffix(".ZIP")
        val parts = baseName.split("_")

        return if (parts.size >= 3 && parts.last().startsWith("mc")) {
            val mcVersion = parts.last()
            val version = parts[parts.size - 2]
            val id = parts.dropLast(2).joinToString("_")
            ShaderPackInfo(
                id = sanitizeForFilename(id),
                version = sanitizeForFilename(version),
                mcVersion = mcVersion.removePrefix("mc"),
            )
        } else if (parts.size >= 2) {
            val version = parts.last()
            val id = parts.dropLast(1).joinToString("_")
            ShaderPackInfo(
                id = sanitizeForFilename(id),
                version = sanitizeForFilename(version),
                mcVersion = null,
            )
        } else {
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

        ChunkForceLoader.releaseAll()

        sessionData = buildSessionData()

        if (IrisIntegration.isAvailable) {
            originalShaderPack?.let { shaderPack ->
                IrisIntegration.enableShaders(shaderPack)
            } ?: IrisIntegration.disableShaders()
        }

        originalState?.let {
            it.restore()
            logger.info("Original client state restored")
        }

        state = State.Idle
        ticksInState = 0
        currentIndex = 0
        shadersToCapture = emptyList()
        originalShaderPack = null
        screenshotEntries.clear()
        startedAt = null
        originalState = null

        logger.info("Capture session finished")
    }

    private fun buildSessionData(): CaptureSessionData {
        val mc = Minecraft.getInstance()
        val completedAt = Instant.now()

        val player = mc.player
        val position = player?.let { Position(x = it.x, y = it.y, z = it.z) }
        val camera = player?.let { Camera(yaw = it.yRot, pitch = it.xRot) }

        val dimension =
            mc.level
                ?.dimension()
                ?.location()
                ?.toString()

        val sessionDirPath = outputDir.relativeTo(mc.gameDirectory).path

        return CaptureSessionData(
            worldName = worldName,
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
