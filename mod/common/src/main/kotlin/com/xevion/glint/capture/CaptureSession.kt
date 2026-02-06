package com.xevion.glint.capture

import com.xevion.glint.Loggers
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
    private val log = Loggers.Capture.get()
    private var state: State = State.Idle
    private var ticksInState: Int = 0

    private var originalShaderPack: String? = null
    private var shadersToCapture: List<ShaderSpec> = emptyList()
    private var currentIndex: Int = 0
    private var startedAt: Instant? = null
    private val screenshotEntries: MutableList<ScreenshotEntry> = mutableListOf()

    private val stabilizationDetector = StabilizationDetector()
    private var resolvedScene: com.xevion.glint.scene.ResolvedScene? = null
    private var sceneApplied: Boolean = false
    private var originalState: CaptureStateSnapshot? = null

    private var sessionData: CaptureSessionData? = null

    /** Called on Util.ioPool() thread after the screenshot file is written to disk. */
    var onScreenshotTaken: ((ScreenshotEntry, File) -> Unit)? = null

    /**
     * Starts a new capture session.
     * @return true if session started, false if already running or scene/Iris unavailable
     */
    fun start(): Boolean {
        if (state != State.Idle) {
            log.warn("Capture session already in progress")
            return false
        }

        val mc = Minecraft.getInstance()

        if (mc.singleplayerServer == null) {
            log.error("Capture system is not available in multiplayer")
            return false
        }

        resolvedScene = SceneManager.loadScene(sceneId)
        if (resolvedScene == null) {
            log.error("Failed to load scene") {
                "scene_id" to sceneId
            }
            return false
        }

        startedAt = Instant.now()
        screenshotEntries.clear()

        if (!IrisIntegration.isAvailable) {
            log.warn("Iris is not available, capturing vanilla only")
            shadersToCapture = listOf(ShaderSpec(filename = null))
            originalShaderPack = null
        } else {
            originalShaderPack =
                if (IrisIntegration.isShaderPackInUse().getOrDefault(false)) {
                    IrisIntegration.getShaderPackName().getOrNull()
                } else {
                    null
                }

            shadersToCapture = shaders

            log.info("Starting capture session") {
                "config_count" to shadersToCapture.size
            }
        }

        currentIndex = 0

        originalState = CaptureStateSnapshot.capture()
        log.debug("Original client state saved")

        transitionTo(State.ApplyingScene)
        return true
    }

    /**
     * Called every client tick to advance the session state machine.
     */
    fun tick() {
        if (state == State.Idle) return

        if (CaptureStateManager.consumeCancelRequest()) {
            log.info("Capture session cancelled by user")
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

    val isRunning: Boolean
        get() = state != State.Idle

    fun getSessionData(): CaptureSessionData? = sessionData

    private fun transitionTo(newState: State) {
        log.debug("Capture session state transition") {
            "from" to state
            "to" to newState
        }
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
            log.error("Scene not loaded")
            transitionTo(State.Finishing)
            return
        }

        if (!sceneApplied) {
            log.debug("Applying scene") {
                "scene_id" to scene.scene.id
            }
            if (!SceneApplicator.apply(scene)) {
                log.error("Failed to apply scene")
                transitionTo(State.Finishing)
                return
            }
            sceneApplied = true
            log.debug("Scene applied successfully")
        }

        if (ticksInState >= SCENE_APPLICATION_WAIT_TICKS) {
            transitionTo(State.LoadingShader)
        }
    }

    private fun handleLoadingShader() {
        val config = shadersToCapture.getOrNull(currentIndex)
        if (config == null) {
            log.error("Invalid shader config") {
                "index" to currentIndex
            }
            advanceToNextShader()
            return
        }

        if (IrisIntegration.isAvailable) {
            log.info("Loading shader configuration") {
                "shader" to config.displayName
            }

            val result =
                if (config.filename == null) {
                    IrisIntegration.disableShaders()
                } else {
                    IrisIntegration.enableShaders(config.filename).onFailure {
                        log.error("Failed to load shader pack") {
                            "pack" to config.filename
                        }
                        advanceToNextShader()
                        return
                    }

                    if (config.profile != null) {
                        IrisIntegration.applyShaderProfile(config.profile).onFailure {
                            log.error("Failed to apply profile, skipping") {
                                "profile" to config.profile
                            }
                            advanceToNextShader()
                            return
                        }
                    } else {
                        IrisIntegration.resetShaderOptions()
                    }

                    Result.success(Unit)
                }

            if (result.isFailure) {
                log.error("Failed to load shader configuration, skipping") {
                    "shader" to config.displayName
                }
                advanceToNextShader()
                return
            }
        }

        val scene = resolvedScene
        if (scene != null) {
            log.debug("Reapplying scene for shader") {
                "shader" to config.displayName
            }
            if (!SceneApplicator.apply(scene)) {
                log.warn("Failed to reapply scene for shader") {
                    "shader" to config.displayName
                }
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
            log.error("Invalid shader config") {
                "index" to currentIndex
            }
            advanceToNextShader()
            return
        }

        val screenshotName = buildScreenshotFilename(config)
        log.info("Capturing screenshot") {
            "shader" to config.displayName
            "file" to screenshotName
        }

        val timestamp = Instant.now().toString()
        val shaderInfo = config.filename?.let { parseShaderPackName(it) }
        val shaderMeta =
            if (config.filename != null && shaderInfo != null) {
                ShaderMetadata(
                    packFile = config.filename,
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

        // Capture values before the async write — Screenshot.grab() writes the file
        // asynchronously on Util.ioPool(), so we invoke the callback from inside the
        // consumer where the file is guaranteed to exist.
        val capturedEntry = screenshotEntries.last()
        val screenshotFile = File(outputDir, "screenshots/$screenshotName")
        val callback = onScreenshotTaken

        net.minecraft.client.Screenshot.grab(
            outputDir,
            screenshotName,
            renderTarget,
        ) { message ->
            log.debug("Screenshot saved") {
                "message" to message.string
            }
            callback?.invoke(capturedEntry, screenshotFile)
        }

        transitionTo(State.PostCaptureCooldown)
    }

    private fun buildScreenshotFilename(config: ShaderSpec): String {
        if (config.filename == null) {
            return "vanilla.png"
        }

        val shaderInfo = parseShaderPackName(config.filename)
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
        log.info("Capture session complete, restoring original state")

        ChunkForceLoader.releaseAll()

        sessionData = buildSessionData()

        if (IrisIntegration.isAvailable) {
            originalShaderPack?.let { shaderPack ->
                IrisIntegration.enableShaders(shaderPack)
            } ?: IrisIntegration.disableShaders()
        }

        originalState?.let {
            it.restore()
            log.debug("Original client state restored")
        }

        state = State.Idle
        ticksInState = 0
        currentIndex = 0
        shadersToCapture = emptyList()
        originalShaderPack = null
        screenshotEntries.clear()
        startedAt = null
        originalState = null

        log.info("Capture session finished")
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
            shaderPacks = shadersToCapture.mapNotNull { it.filename }.distinct(),
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
