package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.GlintJsonFile
import com.xevion.glint.capture.CaptureEntry
import com.xevion.glint.capture.CaptureSession
import com.xevion.glint.capture.CaptureSessionData
import com.xevion.glint.capture.CaptureStateManager
import com.xevion.glint.capture.HighResCapture
import com.xevion.glint.capture.IrisIntegration
import com.xevion.glint.io.SessionDirectoryManager
import com.xevion.glint.scene.ResolvedScene
import com.xevion.glint.scene.SceneManager
import kotlinx.serialization.SerializationException
import net.minecraft.client.Minecraft
import java.io.File
import java.io.IOException
import java.time.Instant

/** Event fired when a single capture is taken during orchestration. */
class CaptureTakenEvent(
    val entry: CaptureEntry,
    val fileBytes: ByteArray,
    val sceneId: String,
)

/**
 * Orchestrates multi-world, multi-shader, multi-scene capture sessions.
 *
 * Executes captures in world → shader → scene order to minimize expensive operations:
 * - World loads happen once per unique world
 * - Shader loads happen once per shader per world (not per scene)
 * - Scene transitions are cheap teleports within an already-loaded shader
 *
 * The 4K framebuffer is kept active for the entire orchestration to avoid
 * per-session Iris pipeline recreation from resolution changes.
 */
class Orchestrator {
    private val log = Loggers.Orchestration.get()
    private val worldLoader = WorldLoader()

    /** Called on main thread after each capture is taken, with file bytes read eagerly. */
    var onCaptureTaken: ((CaptureTakenEvent) -> Unit)? = null

    private var state: State = State.Idle
    private var ticksInState: Int = 0

    private var spec: CaptureSpec? = null

    // Capture plan: world → shader → scenes
    private var capturePlan: List<WorldCaptures> = emptyList()
    private var currentWorldIndex: Int = 0
    private var currentShaderIndex: Int = 0

    private var captureSession: CaptureSession? = null
    private var sessionDir: File? = null
    private var sessionId: String = ""
    private var startedAt: Instant? = null
    private val sessionDataList = mutableListOf<CaptureSessionData>()

    private var originalShaderPack: String? = null
    private var highResSessionActive: Boolean = false

    /** A group of scenes in a single world, organized by shader. */
    private data class WorldCaptures(
        val worldFolder: String,
        val shaderRuns: List<ShaderRun>,
    )

    /** All scenes to capture with a specific shader. */
    private data class ShaderRun(
        val shader: ShaderSpec,
        val scenes: List<SceneEntry>,
    )

    private data class SceneEntry(
        val sceneId: String,
        val scene: ResolvedScene,
    )

    fun start(spec: CaptureSpec): Boolean {
        if (state != State.Idle) {
            log.warn("Orchestrator already running")
            return false
        }

        log.info("Starting orchestration") {
            "scene_count" to spec.sceneIds.size
            "shader_count" to spec.shaders.size
        }
        this.spec = spec

        if (!createSessionDirectory()) {
            return false
        }

        if (!CaptureStateManager.startCapture()) {
            log.warn("Cannot start orchestration - capture already active")
            return false
        }
        transitionTo(State.Planning)
        return true
    }

    fun tick() {
        if (state == State.Idle) return

        if (!CaptureStateManager.isActive()) {
            log.info("Orchestration cancelled by user")
            finishWithError("User cancelled")
            return
        }

        ticksInState++

        when (state) {
            State.Idle -> {}

            State.Planning -> {
                handlePlanning()
            }

            State.LoadingWorld -> {
                handleLoadingWorld()
            }

            State.LoadingShader -> {
                handleLoadingShader()
            }

            State.CapturingScenes -> {
                handleCapturingScenes()
            }

            State.GeneratingManifest -> {
                handleGeneratingManifest()
            }

            State.Finishing -> {
                handleFinishing()
            }
        }
    }

    val isRunning: Boolean get() = state != State.Idle

    private fun transitionTo(newState: State) {
        log.debug("Orchestrator state transition") {
            "from" to state
            "to" to newState
        }
        state = newState
        ticksInState = 0

        if (newState == State.LoadingWorld) {
            worldLoader.reset()
        }
    }

    private fun handlePlanning() {
        val plan = buildCapturePlan()
        if (plan.isEmpty()) {
            finishWithError("No valid scenes found")
            return
        }

        capturePlan = plan
        currentWorldIndex = 0
        currentShaderIndex = 0

        val totalScenes = plan.sumOf { w -> w.shaderRuns.sumOf { it.scenes.size } }
        val totalShaderLoads = plan.sumOf { it.shaderRuns.size }
        log.info("Capture plan ready") {
            "worlds" to plan.size
            "shader_loads" to totalShaderLoads
            "total_captures" to totalScenes
        }

        // Save original shader state and begin 4K session for entire orchestration
        if (IrisIntegration.isAvailable) {
            originalShaderPack =
                if (IrisIntegration.isShaderPackInUse().getOrDefault(false)) {
                    IrisIntegration.getShaderPackName().getOrNull()
                } else {
                    null
                }
        }

        if (!HighResCapture.beginSession()) {
            finishWithError("Failed to begin high-res capture session")
            return
        }
        highResSessionActive = true

        transitionTo(State.LoadingWorld)
    }

    private fun handleLoadingWorld() {
        val world =
            getCurrentWorld() ?: run {
                transitionTo(State.GeneratingManifest)
                return
            }

        when (val result = worldLoader.loadWorld(world.worldFolder, ticksInState)) {
            is WorldLoader.LoadResult.Complete -> {
                log.info("World ready") {
                    "world" to world.worldFolder
                    "world_progress" to "${currentWorldIndex + 1}/${capturePlan.size}"
                    "shaders_in_world" to world.shaderRuns.size
                }
                currentShaderIndex = 0
                transitionTo(State.LoadingShader)
            }

            is WorldLoader.LoadResult.LoadingWorld,
            is WorldLoader.LoadResult.UnloadingWorld,
            -> {}

            is WorldLoader.LoadResult.Timeout -> {
                finishWithError(result.reason)
            }

            is WorldLoader.LoadResult.Failed -> {
                finishWithError(result.reason)
            }
        }
    }

    private fun handleLoadingShader() {
        val world =
            getCurrentWorld() ?: run {
                advanceToNextWorld()
                return
            }
        val shaderRun =
            world.shaderRuns.getOrNull(currentShaderIndex) ?: run {
                advanceToNextWorld()
                return
            }

        // Shader loading is synchronous — set config and reload in a single tick
        val shader = shaderRun.shader
        log.info("Loading shader") {
            "shader" to shader.displayName
            "shader_progress" to "${currentShaderIndex + 1}/${world.shaderRuns.size}"
            "scenes" to shaderRun.scenes.size
        }

        if (IrisIntegration.isAvailable) {
            val result =
                if (shader.filename == null) {
                    IrisIntegration.disableShaders()
                } else {
                    IrisIntegration.enableShaders(shader.filename, shader.profile)
                }

            if (result.isFailure) {
                log.error("Failed to load shader, skipping") {
                    "shader" to shader.displayName
                }
                advanceToNextShader()
                return
            }
        }

        // Start capture session for this shader's scenes
        val currentSessionDir =
            sessionDir ?: run {
                finishWithError("No session directory")
                return
            }
        val worldDir = File(currentSessionDir, world.worldFolder)
        if (!worldDir.exists() && !worldDir.mkdirs()) {
            finishWithError("Failed to create world directory")
            return
        }

        val newSession =
            CaptureSession(
                shader = shader,
                scenes = shaderRun.scenes.map { CaptureSession.SceneInput(it.sceneId, it.scene) },
                outputDir = worldDir,
                worldName = world.worldFolder,
            ).also { session ->
                session.onCaptureTaken = { entry, file, sceneId ->
                    if (file.exists()) {
                        val bytes = file.readBytes()
                        onCaptureTaken?.invoke(CaptureTakenEvent(entry, bytes, sceneId))
                    } else {
                        log.warn("Capture file not found, skipping upload") {
                            "path" to file.absolutePath
                            "scene_id" to sceneId
                        }
                    }
                }
            }

        if (!newSession.start()) {
            log.error("Failed to start capture session") {
                "shader" to shader.displayName
            }
            advanceToNextShader()
            return
        }
        captureSession = newSession

        transitionTo(State.CapturingScenes)
    }

    private fun handleCapturingScenes() {
        val session =
            captureSession ?: run {
                advanceToNextShader()
                return
            }

        session.tick()

        if (!session.isRunning) {
            finalizeShaderRun()
            advanceToNextShader()
        }
    }

    private fun handleGeneratingManifest() {
        log.info("Generating master manifest")
        writeManifest(partial = false)

        sessionDir?.let { dir ->
            log.info("Orchestration complete") {
                "results_dir" to dir.absolutePath
                "sessions" to sessionDataList.size
            }
        }

        transitionTo(State.Finishing)
    }

    private fun handleFinishing() {
        log.info("Orchestration finished")
        cleanup()

        if (spec?.shutdownOnComplete == true) {
            log.info("Shutting down Minecraft (shutdownOnComplete)")
            Minecraft.getInstance().stop()
        }
    }

    private fun advanceToNextShader() {
        captureSession = null
        val world = getCurrentWorld()
        if (world != null && currentShaderIndex + 1 < world.shaderRuns.size) {
            currentShaderIndex++
            transitionTo(State.LoadingShader)
        } else {
            advanceToNextWorld()
        }
    }

    private fun advanceToNextWorld() {
        captureSession = null
        currentWorldIndex++
        currentShaderIndex = 0

        if (currentWorldIndex >= capturePlan.size) {
            transitionTo(State.GeneratingManifest)
        } else {
            transitionTo(State.LoadingWorld)
        }
    }

    private fun finalizeShaderRun() {
        val session = captureSession ?: return
        val allData = session.getAllSessionData()

        for (data in allData) {
            if (data.captures.isNotEmpty()) {
                sessionDataList.add(data)
            } else {
                log.warn("Session produced no captures") { "scene_id" to data.sceneId }
            }
        }

        // Rename screenshots directories to scene IDs
        val world = getCurrentWorld() ?: return
        val currentSessionDir = sessionDir ?: return
        val worldDir = File(currentSessionDir, world.worldFolder)

        for (sceneInput in session.scenes) {
            renameCapturesDirectory(sceneInput.sceneId, worldDir)
        }
    }

    private fun renameCapturesDirectory(
        sceneId: String,
        worldDir: File,
    ) {
        val capturesDir = File(worldDir, "screenshots")
        val sceneDir = File(worldDir, sceneId)

        if (!capturesDir.exists()) return

        if (sceneDir.exists()) {
            sceneDir.deleteRecursively()
        }

        if (!capturesDir.renameTo(sceneDir)) {
            log.error("Failed to rename captures directory") {
                "from" to capturesDir.absolutePath
                "to" to sceneDir.absolutePath
            }
        }
    }

    private fun finishWithError(reason: String) {
        log.error("Orchestration failed") { "reason" to reason }

        if (sessionDataList.isNotEmpty()) {
            writeManifest(partial = true)
        }

        cleanup()

        if (spec?.shutdownOnComplete == true) {
            log.info("Shutting down Minecraft after failure (shutdownOnComplete)")
            Minecraft.getInstance().stop()
        }
    }

    private fun cleanup() {
        captureSession = null

        // End 4K session if active
        if (highResSessionActive) {
            HighResCapture.endSession()
            highResSessionActive = false
        }

        // Restore original shader
        if (IrisIntegration.isAvailable) {
            originalShaderPack?.let { IrisIntegration.enableShaders(it) }
                ?: IrisIntegration.disableShaders()
        }

        CaptureStateManager.endCapture()

        state = State.Idle
        ticksInState = 0
        capturePlan = emptyList()
        currentWorldIndex = 0
        currentShaderIndex = 0
        sessionDir = null
        sessionId = ""
        startedAt = null
        sessionDataList.clear()
        worldLoader.reset()
        originalShaderPack = null
        spec = null
    }

    /**
     * Builds the capture plan: world → shader → scenes.
     *
     * Groups scenes by world, then for each world creates a [ShaderRun] per
     * shader in the spec. This ensures each shader is loaded only once per world.
     */
    private fun buildCapturePlan(): List<WorldCaptures> {
        val currentSpec = spec ?: return emptyList()

        // Resolve all scenes, grouped by world
        val scenesByWorld = mutableMapOf<String, MutableList<SceneEntry>>()
        val worldOrder = mutableListOf<String>()

        for (sceneId in currentSpec.sceneIds) {
            val resolvedScene = SceneManager.loadScene(sceneId)
            if (resolvedScene == null) {
                log.warn("Failed to load scene, skipping") { "scene_id" to sceneId }
                continue
            }

            val worldFolder = resolvedScene.worldFolderName
            if (!worldLoader.worldExists(worldFolder)) {
                log.warn("World directory not found, skipping scene") {
                    "scene_id" to sceneId
                    "world" to worldFolder
                }
                continue
            }

            if (worldFolder !in scenesByWorld) {
                worldOrder.add(worldFolder)
            }
            scenesByWorld
                .getOrPut(worldFolder) { mutableListOf() }
                .add(SceneEntry(sceneId, resolvedScene))
        }

        // For each world, create a ShaderRun per shader
        val plan =
            worldOrder.mapNotNull { worldFolder ->
                val scenes = scenesByWorld[worldFolder] ?: return@mapNotNull null
                val shaderRuns =
                    currentSpec.shaders.map { shader ->
                        ShaderRun(shader, scenes)
                    }
                WorldCaptures(worldFolder, shaderRuns)
            }

        if (plan.isNotEmpty()) {
            log.info("Resolved capture plan") {
                "worlds" to plan.size
                "scenes" to
                    plan.sumOf { w ->
                        w.shaderRuns
                            .firstOrNull()
                            ?.scenes
                            ?.size ?: 0
                    }
                "shaders" to currentSpec.shaders.size
            }
        }

        return plan
    }

    private fun getCurrentWorld(): WorldCaptures? = capturePlan.getOrNull(currentWorldIndex)

    private fun createSessionDirectory(): Boolean {
        val mc = Minecraft.getInstance()
        val currentSpec = spec ?: return false
        startedAt = Instant.now()

        return try {
            if (currentSpec.outputDir != null) {
                val dir = File(mc.gameDirectory, currentSpec.outputDir)
                if (!dir.exists() && !dir.mkdirs()) {
                    log.error("Failed to create output directory") { "path" to dir.absolutePath }
                    return false
                }
                sessionId = currentSpec.runId?.let { "run_$it" } ?: dir.name
                sessionDir = dir
            } else {
                val capturesDir = File(mc.gameDirectory, "glint/captures")
                val (dir, id) = SessionDirectoryManager.createSessionDirectory(capturesDir, startedAt!!)
                sessionId = id
                sessionDir = dir
            }
            log.info("Session directory created") { "path" to sessionDir!!.absolutePath }
            true
        } catch (e: IOException) {
            log.error(e, "Failed to create session directory")
            false
        } catch (e: SecurityException) {
            log.error(e, "Failed to create session directory")
            false
        }
    }

    private fun writeManifest(partial: Boolean) {
        val currentSessionDir = sessionDir ?: return
        val manifestName = if (partial) "manifest_partial.json" else "manifest.json"
        val manifestFile = File(currentSessionDir, manifestName)

        val manifest =
            OrchestrationManifest.create(
                sessionDataList,
                sessionId,
                startedAt ?: Instant.now(),
                runId = spec?.runId,
            )

        try {
            manifestFile.writeText(GlintJsonFile.encodeToString(OrchestrationManifest.serializer(), manifest))
            log.info("Manifest written") {
                "partial" to partial
                "path" to manifestFile.absolutePath
            }
        } catch (e: IOException) {
            log.error(e, "Failed to write manifest")
        } catch (e: SerializationException) {
            log.error(e, "Failed to serialize manifest")
        }
    }

    private enum class State {
        Idle,
        Planning,
        LoadingWorld,
        LoadingShader,
        CapturingScenes,
        GeneratingManifest,
        Finishing,
    }
}
