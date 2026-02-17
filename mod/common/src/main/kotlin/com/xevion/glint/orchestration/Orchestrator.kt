package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.GlintJsonFile
import com.xevion.glint.api.WorkItem
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
    val presetId: String? = null,
)

/**
 * Orchestrates multi-world, multi-shader, multi-scene capture sessions.
 *
 * Executes captures in world → shader → scene order to minimize expensive operations:
 * - World loads happen once per unique world
 * - Shader loads happen once per shader per world (not per scene)
 * - Scene transitions are cheap teleports within an already-loaded shader
 *
 * Supports two entry points:
 * - [start] with a [CaptureSpec] for interactive captures (UI-driven)
 * - [startLinear] with pre-ordered [WorkItem]s for autonomous captures (backend-driven)
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

    // Interactive path
    private var spec: CaptureSpec? = null

    // Linear (autonomous) path
    private var workItems: List<WorkItem>? = null
    private var linearRunId: String? = null
    private var linearOutputDir: String? = null

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

    /** All scenes to capture with a specific shader (and optional profile). */
    private data class ShaderRun(
        val shader: ShaderSpec,
        val scenes: List<SceneEntry>,
    )

    private data class SceneEntry(
        val sceneId: String,
        val scene: ResolvedScene,
    )

    /** Starts orchestration from a [CaptureSpec] (interactive UI path). */
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

    /**
     * Starts orchestration from pre-ordered [WorkItem]s (autonomous capture path).
     *
     * The backend returns items already sorted in optimal execution order
     * (world → shader → profile → clustered scenes). This method builds the
     * same internal plan structure by walking the pre-ordered list and detecting
     * transitions.
     *
     * Scene definitions must already be written to disk before calling this.
     */
    fun startLinear(
        items: List<WorkItem>,
        runId: String,
        outputDir: String? = null,
    ): Boolean {
        if (state != State.Idle) {
            log.warn("Orchestrator already running")
            return false
        }

        log.info("Starting linear orchestration") {
            "items" to items.size
            "run_id" to runId
        }
        this.workItems = items
        this.linearRunId = runId
        this.linearOutputDir = outputDir

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
        val plan =
            if (workItems != null) {
                buildCapturePlanFromWorkItems()
            } else {
                buildCapturePlan()
            }
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
        workItems = null
        linearRunId = null
        linearOutputDir = null
    }

    /**
     * Builds the capture plan from a [CaptureSpec] (interactive path).
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

    /**
     * Builds the capture plan from pre-ordered [WorkItem]s (autonomous path).
     *
     * Walks the backend's pre-sorted list and groups consecutive items into the
     * same plan structure (world → shader/profile → scenes). The backend orders
     * items as world → shader → profile → clustered scenes, so consecutive items
     * sharing the same world+shader+profile become one [ShaderRun].
     */
    private fun buildCapturePlanFromWorkItems(): List<WorldCaptures> {
        val items = workItems ?: return emptyList()
        if (items.isEmpty()) return emptyList()

        val worlds = mutableListOf<WorldCaptures>()
        var currentWorldSlug: String? = null
        var currentShaderRuns = mutableListOf<ShaderRun>()

        // Track current shader run grouping key: (shaderVersionId, profileId)
        var currentGroupKey: Pair<String, String?>? = null
        var currentShaderSpec: ShaderSpec? = null
        var currentScenes = mutableListOf<SceneEntry>()

        for (item in items) {
            val worldFolder = item.sceneSlug
            val groupKey = item.shaderVersionId to item.profileId

            // World transition — flush current world
            if (worldFolder != currentWorldSlug) {
                flushShaderRun(currentShaderSpec, currentScenes, currentShaderRuns)
                flushWorld(currentWorldSlug, currentShaderRuns, worlds)
                currentWorldSlug = worldFolder
                currentShaderRuns = mutableListOf()
                currentGroupKey = null
                currentShaderSpec = null
                currentScenes = mutableListOf()
            }

            // Shader/profile transition — flush current shader run
            if (groupKey != currentGroupKey) {
                flushShaderRun(currentShaderSpec, currentScenes, currentShaderRuns)
                currentGroupKey = groupKey
                currentShaderSpec = buildShaderSpecFromItem(item)
                currentScenes = mutableListOf()
            }

            // Resolve scene and add to current run
            val resolvedScene = SceneManager.loadScene(item.sceneId)
            if (resolvedScene == null) {
                log.warn("Failed to load scene from work item, skipping") { "scene_id" to item.sceneId }
                continue
            }
            currentScenes.add(SceneEntry(item.sceneId, resolvedScene))
        }

        // Flush remaining
        flushShaderRun(currentShaderSpec, currentScenes, currentShaderRuns)
        flushWorld(currentWorldSlug, currentShaderRuns, worlds)

        if (worlds.isNotEmpty()) {
            val totalScenes = worlds.sumOf { w -> w.shaderRuns.sumOf { it.scenes.size } }
            log.info("Resolved linear capture plan") {
                "worlds" to worlds.size
                "shader_runs" to worlds.sumOf { it.shaderRuns.size }
                "total_captures" to totalScenes
            }
        }

        return worlds
    }

    private fun flushShaderRun(
        shader: ShaderSpec?,
        scenes: MutableList<SceneEntry>,
        target: MutableList<ShaderRun>,
    ) {
        if (shader != null && scenes.isNotEmpty()) {
            target.add(ShaderRun(shader, scenes.toList()))
        }
    }

    private fun flushWorld(
        worldFolder: String?,
        shaderRuns: MutableList<ShaderRun>,
        target: MutableList<WorldCaptures>,
    ) {
        if (worldFolder != null && shaderRuns.isNotEmpty()) {
            target.add(WorldCaptures(worldFolder, shaderRuns.toList()))
        }
    }

    private fun buildShaderSpecFromItem(item: WorkItem): ShaderSpec =
        if (item.shaderSlug == "vanilla") {
            ShaderSpec(filename = null)
        } else {
            val hash8 = item.fileHash?.take(8)
            val filename =
                if (hash8 != null) {
                    "${item.shaderSlug}-${item.version}-$hash8.zip"
                } else {
                    "${item.shaderSlug}-${item.version}.zip"
                }
            ShaderSpec(
                filename = filename,
                profile = item.profileName,
                profileId = item.profileId,
            )
        }

    private fun getCurrentWorld(): WorldCaptures? = capturePlan.getOrNull(currentWorldIndex)

    private fun createSessionDirectory(): Boolean {
        val mc = Minecraft.getInstance()
        startedAt = Instant.now()

        return try {
            val outputDir = spec?.outputDir ?: linearOutputDir
            val runId = spec?.runId ?: linearRunId

            if (outputDir != null) {
                val dir = File(mc.gameDirectory, outputDir)
                if (!dir.exists() && !dir.mkdirs()) {
                    log.error("Failed to create output directory") { "path" to dir.absolutePath }
                    return false
                }
                sessionId = runId?.let { "run_$it" } ?: dir.name
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

        val runId = spec?.runId ?: linearRunId
        val manifest =
            OrchestrationManifest.create(
                sessionDataList,
                sessionId,
                startedAt ?: Instant.now(),
                runId = runId,
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
