package com.xevion.glint.capture

import com.xevion.glint.Loggers
import com.xevion.glint.orchestration.ShaderSpec
import com.xevion.glint.scene.ResolvedScene
import com.xevion.glint.scene.SceneApplicator
import com.xevion.glint.scene.SceneApplyResult
import net.minecraft.client.Minecraft
import java.io.File
import java.nio.file.Path
import java.time.Instant
import java.util.concurrent.CompletableFuture

/**
 * Captures all scenes for a single already-loaded shader.
 *
 * The Orchestrator loads the shader and begins the 4K session before creating
 * this class. CaptureSession handles scene iteration: teleport → stabilize → capture.
 * Stabilization waits until the renderer is fully idle (chunks loaded, lighting done,
 * build queue empty) before capturing.
 */
class CaptureSession(
    private val shader: ShaderSpec,
    val scenes: List<SceneInput>,
    private val outputDir: File,
    private val worldName: String,
) {
    private val log = Loggers.Capture.get()
    private var state: State = State.Idle
    private var ticksInState: Int = 0

    private var currentSceneIndex: Int = 0
    private var startedAt: Instant? = null
    private var originalState: CaptureStateSnapshot? = null

    private val stabilizationDetector = StabilizationDetector()
    private var sceneApplied: Boolean = false
    private var lastApplyResult: SceneApplyResult = SceneApplyResult.APPLIED
    private var pendingCapture: CompletableFuture<Path>? = null

    // Per-scene capture tracking, aggregated per scene across the session
    private val sceneCaptures = mutableMapOf<String, MutableList<CaptureEntry>>()
    private val sceneStartTimes = mutableMapOf<String, Instant>()

    /** Called after the capture file is written to disk. */
    var onCaptureTaken: ((CaptureEntry, File, String) -> Unit)? = null

    /** Input scene for the session. */
    data class SceneInput(
        val sceneId: String,
        val scene: ResolvedScene,
    )

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

        if (scenes.isEmpty()) {
            log.error("No scenes to capture")
            return false
        }

        startedAt = Instant.now()
        currentSceneIndex = 0
        sceneCaptures.clear()
        sceneStartTimes.clear()

        originalState = CaptureStateSnapshot.capture()
        log.debug("Original client state saved")

        log.info("Starting capture session") {
            "shader" to shader.displayName
            "scene_count" to scenes.size
        }

        transitionTo(State.ApplyingScene)
        return true
    }

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

            State.WaitingForRebuild -> {
                handleWaitingForRebuild()
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

    val isRunning: Boolean get() = state != State.Idle

    /**
     * Returns session data for all scenes captured.
     * Each scene gets its own [CaptureSessionData] entry.
     */
    fun getAllSessionData(): List<CaptureSessionData> {
        val mc = Minecraft.getInstance()
        val completedAt = Instant.now()

        return scenes.mapNotNull { sceneInput ->
            val entries = sceneCaptures[sceneInput.sceneId] ?: return@mapNotNull null
            val sceneStart = sceneStartTimes[sceneInput.sceneId] ?: startedAt ?: Instant.now()

            val player = mc.player
            val position = player?.let { Position(x = it.x, y = it.y, z = it.z) }
            val camera = player?.let { Camera(yaw = it.yRot, pitch = it.xRot) }
            val dimension =
                mc.level
                    ?.dimension()
                    ?.location()
                    ?.toString()
            val sessionDirPath = outputDir.relativeTo(mc.gameDirectory).path

            CaptureSessionData(
                worldName = worldName,
                sceneId = sceneInput.sceneId,
                sessionDir = sessionDirPath,
                startedAt = sceneStart.toString(),
                completedAt = completedAt.toString(),
                totalCaptures = entries.size,
                shaders = listOfNotNull(shader.filename),
                minecraft =
                    MinecraftInfo(
                        version = mc.launchedVersion,
                        dimension = dimension,
                        position = position,
                        camera = camera,
                    ),
                captures = entries.toList(),
            )
        }
    }

    private fun transitionTo(newState: State) {
        log.debug("Capture session state transition") {
            "from" to state
            "to" to newState
        }
        state = newState
        ticksInState = 0

        if (newState == State.WaitingForRebuild) {
            SodiumIntegration.resetStabilizationState()
        }

        if (newState == State.WaitingForStabilization) {
            stabilizationDetector.reset()
        }

        if (newState == State.ApplyingScene) {
            sceneApplied = false
        }
    }

    private fun handleApplyingScene() {
        val sceneInput =
            scenes.getOrNull(currentSceneIndex) ?: run {
                transitionTo(State.Finishing)
                return
            }

        if (!sceneApplied) {
            sceneStartTimes.putIfAbsent(sceneInput.sceneId, Instant.now())

            log.info("Applying scene") {
                "scene_id" to sceneInput.sceneId
                "progress" to "${currentSceneIndex + 1}/${scenes.size}"
                "shader" to shader.displayName
            }

            val result = SceneApplicator.apply(sceneInput.scene)
            if (result == SceneApplyResult.FAILED) {
                log.error("Failed to apply scene") { "scene_id" to sceneInput.sceneId }
                advanceToNextScene()
                return
            }
            lastApplyResult = result
            sceneApplied = true
        }

        // Adaptive wait: rebuild-triggering changes go through a rebuild-ack gate,
        // non-rebuild changes just need a couple ticks for options to propagate.
        if (lastApplyResult == SceneApplyResult.APPLIED_WITH_REBUILD) {
            transitionTo(State.WaitingForRebuild)
        } else if (ticksInState >= OPTION_PROPAGATION_TICKS) {
            transitionTo(State.WaitingForStabilization)
        }
    }

    private fun handleWaitingForRebuild() {
        val renderingComplete = SodiumIntegration.isRenderingComplete()

        if (renderingComplete == true) {
            log.debug("Rebuild acknowledged by Sodium") { "ticks" to ticksInState }
            transitionTo(State.WaitingForStabilization)
            return
        }

        // Fallback for vanilla (no Sodium): use LevelRenderer
        if (renderingComplete == null) {
            val mc = Minecraft.getInstance()
            if (mc.levelRenderer.hasRenderedAllSections()) {
                log.debug("Rebuild complete (vanilla renderer)") { "ticks" to ticksInState }
                transitionTo(State.WaitingForStabilization)
                return
            }
        }

        if (ticksInState >= REBUILD_TIMEOUT_TICKS) {
            log.warn("Rebuild wait timed out, proceeding to stabilization") {
                "ticks" to ticksInState
                "timeout" to REBUILD_TIMEOUT_TICKS
            }
            transitionTo(State.WaitingForStabilization)
        }
    }

    private fun handleWaitingForStabilization() {
        if (stabilizationDetector.isStable(ticksInState)) {
            transitionTo(State.Capturing)
        }
    }

    private fun handleCapturing() {
        val pending = pendingCapture
        if (pending != null) {
            if (!pending.isDone) return
            pendingCapture = null
            transitionTo(State.PostCaptureCooldown)
            return
        }

        val sceneInput =
            scenes.getOrNull(currentSceneIndex) ?: run {
                advanceToNextScene()
                return
            }

        val captureFilename = buildCaptureFilename(shader)
        log.info("Taking high-res capture") {
            "shader" to shader.displayName
            "scene" to sceneInput.sceneId
            "file" to captureFilename
            "resolution" to "${HighResCapture.CAPTURE_WIDTH}x${HighResCapture.CAPTURE_HEIGHT}"
        }

        val timestamp = Instant.now().toString()
        val shaderInfo = shader.filename?.let { parseShaderPackName(it) }
        val shaderMeta =
            if (shader.filename != null && shaderInfo != null) {
                ShaderMetadata(
                    filename = shader.filename,
                    id = shaderInfo.id,
                    version = shaderInfo.version,
                    profile = shader.profile,
                    profileId = shader.profileId,
                )
            } else {
                null
            }

        val entry =
            CaptureEntry(
                file = captureFilename,
                timestamp = timestamp,
                shader = shaderMeta,
                resolution =
                    Resolution(
                        width = HighResCapture.CAPTURE_WIDTH,
                        height = HighResCapture.CAPTURE_HEIGHT,
                    ),
            )

        sceneCaptures.getOrPut(sceneInput.sceneId) { mutableListOf() }.add(entry)

        val screenshotsDir = File(outputDir, "screenshots")
        if (!screenshotsDir.exists()) {
            screenshotsDir.mkdirs()
        }

        val outputPath = screenshotsDir.toPath().resolve(captureFilename)
        val captureFile = File(outputDir, "screenshots/$captureFilename")
        val callback = onCaptureTaken
        val sceneId = sceneInput.sceneId

        val future =
            HighResCapture.startCapture(outputPath) ?: run {
                log.error("Failed to start high-res capture (already in progress)")
                advanceToNextScene()
                return
            }

        future.thenAccept {
            log.debug("High-res capture saved") { "file" to captureFilename }
            callback?.invoke(entry, captureFile, sceneId)
        }

        pendingCapture = future
    }

    private fun handlePostCaptureCooldown() {
        if (ticksInState >= POST_CAPTURE_COOLDOWN_TICKS) {
            advanceToNextScene()
        }
    }

    private fun advanceToNextScene() {
        currentSceneIndex++
        if (currentSceneIndex >= scenes.size) {
            transitionTo(State.Finishing)
        } else {
            transitionTo(State.ApplyingScene)
        }
    }

    private fun handleFinishing() {
        log.info("Capture session complete, restoring state") {
            "shader" to shader.displayName
            "scenes_captured" to sceneCaptures.size
        }

        ChunkForceLoader.releaseAll()

        originalState?.let {
            it.restore()
            log.debug("Original client state restored")
        }

        state = State.Idle
        ticksInState = 0
        currentSceneIndex = 0
        originalState = null
        pendingCapture = null
    }

    private fun buildCaptureFilename(config: ShaderSpec): String {
        val sceneInput = scenes.getOrNull(currentSceneIndex) ?: return "unknown.webp"
        val scenePrefix = sanitizeForFilename(sceneInput.sceneId)

        if (config.filename == null) {
            return "${scenePrefix}_vanilla.webp"
        }

        val shaderInfo = parseShaderPackName(config.filename)
        val profileSuffix = config.profile?.let { "_${sanitizeForFilename(it)}" } ?: ""
        return "${scenePrefix}_${shaderInfo.id}_${shaderInfo.version}$profileSuffix.webp"
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

    private enum class State {
        Idle,
        ApplyingScene,
        WaitingForRebuild,
        WaitingForStabilization,
        Capturing,
        PostCaptureCooldown,
        Finishing,
    }

    companion object {
        private const val POST_CAPTURE_COOLDOWN_TICKS = 10
        private const val OPTION_PROPAGATION_TICKS = 2
        private const val REBUILD_TIMEOUT_TICKS = 200
    }
}
