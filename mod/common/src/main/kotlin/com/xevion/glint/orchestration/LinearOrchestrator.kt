package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.WorkItem
import com.xevion.glint.api.effectiveTimeOfDayTicks
import com.xevion.glint.api.effectiveWeather
import com.xevion.glint.api.toShaderSpec
import com.xevion.glint.capture.CaptureEntry
import com.xevion.glint.capture.CaptureSessionData
import com.xevion.glint.capture.CaptureStateManager
import com.xevion.glint.capture.CaptureTimeOverride
import com.xevion.glint.capture.ChunkForceLoader
import com.xevion.glint.capture.HighResCapture
import com.xevion.glint.capture.MinecraftInfo
import com.xevion.glint.capture.Resolution
import com.xevion.glint.capture.StabilizationDetector
import com.xevion.glint.scene.CameraPosition
import com.xevion.glint.scene.InjectionProcess
import com.xevion.glint.scene.LoadedScene
import com.xevion.glint.scene.SceneInjector
import net.minecraft.client.Minecraft
import java.io.File
import java.nio.file.Path
import java.time.Instant
import java.util.concurrent.CompletableFuture

/**
 * Orchestrates autonomous capture using scene package injection.
 *
 * Walks a flat list of [WorkItem]s in backend-provided order, detecting transitions
 * between scenes, presets, and shaders to minimize expensive operations:
 * - Scene change → full injection pipeline (chunks + entities + camera)
 * - Preset change → environment-only (time, weather, moon phase)
 * - Shader change → Iris pipeline reload
 *
 * Uses a void staging world ([StagingWorld]) as the injection target.
 * Scene data is served from memory via [SceneInjector] and the ChunkStorageMixin.
 *
 * Delegates environment manipulation to [EnvironmentController] and session I/O
 * to [CaptureSession].
 */
class LinearOrchestrator {
    private val log = Loggers.Orchestration.get()
    private val stagingWorld = StagingWorld()
    private val sceneInjector = SceneInjector()
    private val stabilizationDetector = StabilizationDetector()

    /** Called on main thread after each capture is taken, with file bytes read eagerly. */
    var onCaptureTaken: ((CaptureTakenEvent) -> Unit)? = null

    private var state: State = State.Idle
    private var ticksInState: Int = 0

    // Linear path context
    private var workItems: List<WorkItem> = emptyList()
    private var runId: String? = null
    private var outputDir: String? = null
    private var scenePackages: Map<String, File> = emptyMap()

    // Work item iteration
    private var currentItemIndex: Int = 0

    // Transition tracking: detect when scene/preset/shader changes between items
    private var currentPackageHash: String? = null
    private var currentPresetId: String? = null
    private var currentShaderVersionId: String? = null
    private var currentShaderSpec: ShaderSpec? = null

    // Scene injection state
    private var loadedScene: LoadedScene? = null
    private var injectionProcess: InjectionProcess? = null

    /** Tracks cross-thread state for InjectionProcess ticking dispatched via server.execute {}. */
    private sealed interface ServerDispatchState {
        /** No server-thread work in flight. */
        data object Idle : ServerDispatchState

        /** A server.execute{} lambda is queued/running. */
        data object Pending : ServerDispatchState

        /** Server thread signaled that injection tick returned terminal. */
        data object Terminal : ServerDispatchState
    }

    // Server-thread dispatch for injection ticking: prevents queuing multiple
    // server.execute calls while one is still pending
    @Volatile private var serverDispatch: ServerDispatchState = ServerDispatchState.Idle

    // Capture state
    private var pendingCapture: CompletableFuture<Path>? = null
    private var highResSessionActive: Boolean = false

    // Collaborators (constructed fresh per run)
    private var environmentController: EnvironmentController? = null
    private var captureSession: CaptureSession? = null

    /**
     * Starts orchestration from pre-ordered [WorkItem]s with scene package paths.
     *
     * @param items Pre-sorted work items from the backend
     * @param runId Capture run ID
     * @param packages Map of package hash → local ZIP file
     * @param outputDir Output directory for captures (relative to game directory)
     */
    fun start(
        items: List<WorkItem>,
        runId: String,
        packages: Map<String, File>,
        outputDir: String? = null,
    ): Boolean {
        if (state != State.Idle) {
            log.warn("LinearOrchestrator already running")
            return false
        }

        log.info("Starting linear orchestration") {
            "items" to items.size
            "run_id" to runId
            "packages" to packages.size
        }

        this.workItems = items
        this.runId = runId
        this.scenePackages = packages
        this.outputDir = outputDir

        val session = CaptureSession(runId, outputDir)
        if (!session.createSessionDirectory()) {
            return false
        }
        captureSession = session

        environmentController = EnvironmentController()

        if (!CaptureStateManager.startCapture()) {
            log.warn("Cannot start orchestration — capture already active")
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

            State.LoadingStagingWorld -> {
                handleLoadingStagingWorld()
            }

            State.InjectingScene -> {
                handleInjectingScene()
            }

            State.ApplyingPreset -> {
                handleApplyingPreset()
            }

            State.LoadingShader -> {
                handleLoadingShader()
            }

            State.Stabilizing -> {
                handleStabilizing()
            }

            State.SettlingForCapture -> {
                handleSettlingForCapture()
            }

            State.TakingCapture -> {
                handleTakingCapture()
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

    private fun handlePlanning() {
        if (workItems.isEmpty()) {
            finishWithError("No work items")
            return
        }

        // Validate all required scene packages are available
        val missingPackages =
            workItems
                .filter { it.scenePackage != null }
                .map { it.scenePackage!!.hash }
                .distinct()
                .filter { it !in scenePackages }
        if (missingPackages.isNotEmpty()) {
            finishWithError("Missing scene packages: ${missingPackages.joinToString()}")
            return
        }

        val totalScenes = workItems.map { it.scenePackage?.hash }.distinct().size
        val totalPresets = workItems.map { it.scenePackage?.hash to it.preset?.id }.distinct().size
        val totalShaders = workItems.map { it.shader.versionId }.distinct().size
        log.info("Linear capture plan ready") {
            "items" to workItems.size
            "scenes" to totalScenes
            "presets" to totalPresets
            "shaders" to totalShaders
        }

        environmentController!!.saveOriginalShader()

        if (!HighResCapture.beginSession()) {
            finishWithError("Failed to begin high-res capture session")
            return
        }
        highResSessionActive = true

        currentItemIndex = 0
        stagingWorld.ensureReady()
        transitionTo(State.LoadingStagingWorld)
    }

    private fun handleLoadingStagingWorld() {
        if (stagingWorld.tick()) {
            if (stagingWorld.state == StagingWorld.State.READY) {
                log.info("Staging world ready")
                environmentController!!.applyStandardRenderSettings()
                environmentController!!.freezeWorldState()
                advanceToItem()
            } else {
                finishWithError("Staging world failed: ${stagingWorld.error}")
            }
        }
    }

    private fun handleInjectingScene() {
        val process = injectionProcess
        if (process == null) {
            // Start injection for the current work item's scene package
            val item =
                currentWorkItem() ?: run {
                    finishWithError("No current work item for injection")
                    return
                }

            val packageHash = item.scenePackage?.hash
            if (packageHash == null) {
                finishWithError("Work item missing package hash: ${item.scene.id}")
                return
            }

            val packageFile = scenePackages[packageHash]
            if (packageFile == null) {
                finishWithError("Scene package not found for hash: $packageHash")
                return
            }

            val level = stagingWorld.getServerLevel()
            if (level == null) {
                finishWithError("Staging world has no server level")
                return
            }

            // Deactivate previous scene if any (must run on server thread)
            loadedScene?.let {
                level.server.execute { sceneInjector.deactivate(level) }
                loadedScene = null
            }

            log.info("Loading scene package") {
                "scene" to item.scene.name
                "hash" to packageHash
            }

            val scene =
                try {
                    sceneInjector.load(packageFile.toPath())
                } catch (e: Exception) {
                    finishWithError("Failed to load scene package: ${e.message}")
                    return
                }

            loadedScene = scene

            // Camera position from work item (backend-authoritative, may differ from package meta)
            val camera =
                CameraPosition(
                    x = item.scene.x,
                    y = item.scene.y,
                    z = item.scene.z,
                    yaw = item.scene.yaw.toFloat(),
                    pitch = item.scene.pitch.toFloat(),
                )

            // Inject with zero offset (staging world, no need to offset)
            injectionProcess = sceneInjector.inject(scene, level, 0, 0, camera)
            currentPackageHash = packageHash
            return
        }

        // Check if a previously dispatched server-thread tick has completed
        when (serverDispatch) {
            ServerDispatchState.Terminal -> {
                serverDispatch = ServerDispatchState.Idle
                injectionProcess = null
                if (process.isComplete) {
                    log.info("Scene injection complete")
                    environmentController!!.applySceneViewSettings(currentWorkItem()!!)
                    transitionTo(State.ApplyingPreset)
                } else {
                    finishWithError("Scene injection failed: ${process.error}")
                }
                return
            }

            ServerDispatchState.Pending -> {
                return
            }

            ServerDispatchState.Idle -> { /* fall through to dispatch */ }
        }

        // InjectionProcess.tick() operates on ServerLevel (addRegionTicket, chunk loading,
        // entity spawning, player teleport) — must run on the server thread to avoid
        // concurrent modification of chunk distance tracking data structures.
        val server = Minecraft.getInstance().singleplayerServer
        if (server == null) {
            finishWithError("No integrated server available for injection tick")
            return
        }
        serverDispatch = ServerDispatchState.Pending
        server.execute {
            val terminal = process.tick()
            serverDispatch = if (terminal) ServerDispatchState.Terminal else ServerDispatchState.Idle
        }
    }

    private fun handleApplyingPreset() {
        val item =
            currentWorkItem() ?: run {
                finishWithError("No current work item for preset")
                return
            }

        environmentController!!.applyPresetEnvironment(item)
        currentPresetId = item.preset?.id

        log.info("Preset applied") {
            "preset" to (item.preset?.name ?: item.preset?.id ?: "default")
            "time" to item.effectiveTimeOfDayTicks
            "weather" to item.effectiveWeather
        }

        transitionTo(State.LoadingShader)
    }

    private fun handleLoadingShader() {
        val item =
            currentWorkItem() ?: run {
                finishWithError("No current work item for shader")
                return
            }

        val env = environmentController!!
        val newSpec = item.shader.toShaderSpec()
        val sameVersion = item.shader.versionId == currentShaderVersionId && currentShaderSpec != null
        val sameProfile = newSpec.profileId == currentShaderSpec?.profileId

        // Skip reload only when both shader version AND profile are unchanged
        if (sameVersion && sameProfile) {
            log.debug("Shader unchanged, skipping reload") {
                "shader" to newSpec.displayName
            }
            ChunkForceLoader.forceLoadRenderDistance()
            transitionTo(State.Stabilizing)
            return
        }

        // Skip items whose shader pack already failed to load (avoids repeated
        // reload attempts for every profile of a broken pack)
        if (newSpec.filename != null && env.isShaderFailed(newSpec.filename)) {
            log.warn("Shader pack previously failed, skipping") {
                "shader" to newSpec.displayName
            }
            skipCurrentItem()
            return
        }

        if (sameVersion) {
            log.info("Switching profile") {
                "shader" to newSpec.displayName
                "from" to (currentShaderSpec?.profile ?: "default")
                "to" to (newSpec.profile ?: "default")
            }
        } else {
            log.info("Loading shader") {
                "shader" to newSpec.displayName
            }
        }

        if (env.loadShader(newSpec).isFailure) {
            skipCurrentItem()
            return
        }

        currentShaderVersionId = item.shader.versionId
        currentShaderSpec = newSpec

        ChunkForceLoader.forceLoadRenderDistance()
        transitionTo(State.Stabilizing)
    }

    private fun handleStabilizing() {
        if (stabilizationDetector.isStable()) {
            // Snap weather levels after stabilization to counteract stale packets
            currentWorkItem()?.let { environmentController!!.applyWeatherSnap(it) }
            CaptureTimeOverride.activate()
            transitionTo(State.SettlingForCapture)
        }
    }

    private fun handleSettlingForCapture() {
        if (ticksInState >= PRE_CAPTURE_SETTLE_FRAMES) {
            log.debug("Pre-capture settle complete") {
                "settle_frames" to PRE_CAPTURE_SETTLE_FRAMES
            }
            transitionTo(State.TakingCapture)
        }
    }

    private fun handleTakingCapture() {
        val pending = pendingCapture
        if (pending != null) {
            if (!pending.isDone) return
            pendingCapture = null
            CaptureTimeOverride.deactivate()
            advanceToNextItem()
            return
        }

        val item =
            currentWorkItem() ?: run {
                CaptureTimeOverride.deactivate()
                advanceToNextItem()
                return
            }

        val session = captureSession!!
        val shader = currentShaderSpec ?: ShaderSpec(filename = null)
        val captureFilename = session.buildCaptureFilename(item, shader)

        log.info("Taking capture") {
            "shader" to shader.displayName
            "scene" to item.scene.name
            "preset" to (item.preset?.name ?: "default")
            "file" to captureFilename
        }

        val timestamp = Instant.now().toString()
        val shaderMeta = session.buildShaderMetadata(shader)

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

        val currentSessionDir =
            session.sessionDir ?: run {
                finishWithError("No session directory")
                return
            }
        val screenshotsDir = File(currentSessionDir, "screenshots")
        screenshotsDir.mkdirs()

        val outputPath = screenshotsDir.toPath().resolve(captureFilename)
        val captureFile = File(screenshotsDir, captureFilename)
        val callback = onCaptureTaken
        val sceneId = item.scene.id
        val presetId = item.preset?.id

        val (fileFuture, analysisFuture) =
            HighResCapture.startCapture(outputPath) ?: run {
                log.error("Failed to start high-res capture")
                CaptureTimeOverride.deactivate()
                skipCurrentItem()
                return
            }

        fileFuture.thenAccept {
            log.debug("Capture saved") { "file" to captureFilename }
            if (captureFile.exists()) {
                val bytes = captureFile.readBytes()
                callback?.invoke(
                    CaptureTakenEvent(entry, bytes, sceneId, presetId, analysisFuture),
                )
            } else {
                log.warn("Capture file not found after save") {
                    "path" to captureFile.absolutePath
                }
            }
        }

        // Track for manifest
        val mc = Minecraft.getInstance()
        val player = mc.player
        val dimension =
            mc.level
                ?.dimension()
                ?.location()
                ?.toString()
        val sessionDirPath = currentSessionDir.relativeTo(mc.gameDirectory).path

        session.recordCapture(
            CaptureSessionData(
                worldName = StagingWorld.FOLDER_NAME,
                sceneId = sceneId,
                sessionDir = sessionDirPath,
                startedAt = (session.startedAt ?: Instant.now()).toString(),
                completedAt = Instant.now().toString(),
                totalCaptures = 1,
                shaders = listOfNotNull(shader.filename),
                minecraft =
                    MinecraftInfo(
                        version = mc.launchedVersion,
                        dimension = dimension,
                        position =
                            player?.let {
                                com.xevion.glint.capture
                                    .Position(x = it.x, y = it.y, z = it.z)
                            },
                        camera =
                            player?.let {
                                com.xevion.glint.capture
                                    .Camera(yaw = it.yRot, pitch = it.xRot)
                            },
                    ),
                captures = listOf(entry),
            ),
        )

        pendingCapture = fileFuture
    }

    private fun handleGeneratingManifest() {
        val session = captureSession!!
        log.info("Generating manifest")
        session.writeManifest(partial = false)

        session.sessionDir?.let { dir ->
            log.info("Orchestration complete") {
                "results_dir" to dir.absolutePath
                "captures" to session.entries.size
            }
        }

        transitionTo(State.Finishing)
    }

    private fun handleFinishing() {
        log.info("Orchestration finished")
        cleanup()
    }

    /**
     * Determines what state to enter for the current work item based on
     * what has changed since the last item (scene, preset, or shader).
     */
    private fun advanceToItem() {
        if (currentItemIndex >= workItems.size) {
            transitionTo(State.GeneratingManifest)
            return
        }

        val item = workItems[currentItemIndex]

        // Scene changed → need full injection
        if (item.scenePackage?.hash != currentPackageHash) {
            transitionTo(State.InjectingScene)
            return
        }

        // Preset changed → apply environment only
        if (item.preset?.id != currentPresetId) {
            transitionTo(State.ApplyingPreset)
            return
        }

        // Shader may have changed → check and load
        transitionTo(State.LoadingShader)
    }

    private fun advanceToNextItem() {
        currentItemIndex++
        advanceToItem()
    }

    private fun skipCurrentItem() {
        log.warn("Skipping item") {
            "index" to currentItemIndex
            "scene" to currentWorkItem()?.scene?.name
            "shader" to currentWorkItem()?.shader?.name
        }
        advanceToNextItem()
    }

    private fun finishWithError(reason: String) {
        log.error("Orchestration failed") { "reason" to reason }
        val session = captureSession
        if (session != null && session.entries.isNotEmpty()) {
            session.writeManifest(partial = true)
        }
        cleanup()
    }

    private fun cleanup() {
        // Deactivate scene injection
        val level = stagingWorld.getServerLevel()
        if (level != null && loadedScene != null) {
            level.server.execute { sceneInjector.deactivate(level) }
        }

        // Close collaborators
        environmentController?.close()
        captureSession?.close()

        // End high-res session
        if (highResSessionActive) {
            HighResCapture.endSession()
            highResSessionActive = false
        }

        CaptureTimeOverride.deactivate()
        ChunkForceLoader.releaseAll()
        CaptureStateManager.endCapture()

        // Reset orchestration state
        state = State.Idle
        ticksInState = 0
        workItems = emptyList()
        scenePackages = emptyMap()
        currentItemIndex = 0
        currentPackageHash = null
        currentPresetId = null
        currentShaderVersionId = null
        currentShaderSpec = null
        loadedScene = null
        injectionProcess = null
        serverDispatch = ServerDispatchState.Idle
        pendingCapture = null
        onCaptureTaken = null
        environmentController = null
        captureSession = null
        runId = null
        outputDir = null
        stagingWorld.reset()
    }

    private fun currentWorkItem(): WorkItem? = workItems.getOrNull(currentItemIndex)

    private fun transitionTo(newState: State) {
        log.debug("LinearOrchestrator state: $state → $newState")
        state = newState
        ticksInState = 0

        if (newState == State.Stabilizing) {
            stabilizationDetector.reset()
        }
    }

    private enum class State {
        Idle,
        Planning,
        LoadingStagingWorld,
        InjectingScene,
        ApplyingPreset,
        LoadingShader,
        Stabilizing,
        SettlingForCapture,
        TakingCapture,
        GeneratingManifest,
        Finishing,
    }

    companion object {
        /**
         * Number of frames to render with synthetic time before capturing.
         * Lets TAA/temporal effects reconverge after the time override resets.
         */
        private const val PRE_CAPTURE_SETTLE_FRAMES = 10
    }
}
