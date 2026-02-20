package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.GlintJsonFile
import com.xevion.glint.api.WorkItem
import com.xevion.glint.api.effectiveMoonPhase
import com.xevion.glint.api.effectiveTimeOfDayTicks
import com.xevion.glint.api.effectiveWeather
import com.xevion.glint.api.effectiveWeatherIntensity
import com.xevion.glint.api.toShaderSpec
import com.xevion.glint.capture.CaptureEntry
import com.xevion.glint.capture.CaptureSessionData
import com.xevion.glint.capture.CaptureStateManager
import com.xevion.glint.capture.CaptureTimeOverride
import com.xevion.glint.capture.ChunkForceLoader
import com.xevion.glint.capture.HighResCapture
import com.xevion.glint.capture.IrisIntegration
import com.xevion.glint.capture.MinecraftInfo
import com.xevion.glint.capture.Resolution
import com.xevion.glint.capture.StabilizationDetector
import com.xevion.glint.io.SessionDirectoryManager
import com.xevion.glint.scene.CameraPosition
import com.xevion.glint.scene.InjectionProcess
import com.xevion.glint.scene.LoadedScene
import com.xevion.glint.scene.SceneInjector
import net.minecraft.client.Minecraft
import net.minecraft.world.level.GameRules
import net.minecraft.world.level.GameType
import java.io.File
import java.io.IOException
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
    private var sessionDir: File? = null
    private var sessionId: String = ""
    private var startedAt: Instant? = null
    private var highResSessionActive: Boolean = false
    private var originalShaderPack: String? = null
    private var renderSettingsApplied: Boolean = false

    // Capture tracking for manifest
    private val captureEntries = mutableListOf<CaptureSessionData>()

    // Shader filenames that failed to load — skip remaining items using the same pack
    private val failedShaderPacks = mutableSetOf<String>()

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

        if (!createSessionDirectory()) {
            return false
        }

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

    // -- State handlers --

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

        // Save original shader state and begin 4K session
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

        currentItemIndex = 0
        stagingWorld.ensureReady()
        transitionTo(State.LoadingStagingWorld)
    }

    private fun handleLoadingStagingWorld() {
        if (stagingWorld.tick()) {
            if (stagingWorld.state == StagingWorld.State.READY) {
                log.info("Staging world ready")
                applyStandardRenderSettings()
                freezeWorldState()
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
                    applySceneViewSettings(currentWorkItem()!!)
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

        applyPresetEnvironment(item)
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
        if (newSpec.filename != null && newSpec.filename in failedShaderPacks) {
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

        if (IrisIntegration.isAvailable) {
            val result =
                if (newSpec.filename == null) {
                    IrisIntegration.disableShaders()
                } else {
                    IrisIntegration.enableShaders(newSpec.filename, newSpec.profile)
                }

            if (result.isFailure) {
                log.error("Failed to load shader, skipping item") {
                    "shader" to newSpec.displayName
                }
                if (newSpec.filename != null) {
                    failedShaderPacks.add(newSpec.filename)
                }
                skipCurrentItem()
                return
            }
        }

        currentShaderVersionId = item.shader.versionId
        currentShaderSpec = newSpec

        ChunkForceLoader.forceLoadRenderDistance()
        transitionTo(State.Stabilizing)
    }

    private fun handleStabilizing() {
        if (stabilizationDetector.isStable()) {
            // Snap weather levels after stabilization to counteract stale packets
            currentWorkItem()?.let { applyWeatherSnap(it) }
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

        val shader = currentShaderSpec ?: ShaderSpec(filename = null)
        val captureFilename = buildCaptureFilename(item, shader)

        log.info("Taking capture") {
            "shader" to shader.displayName
            "scene" to item.scene.name
            "preset" to (item.preset?.name ?: "default")
            "file" to captureFilename
        }

        val timestamp = Instant.now().toString()
        val shaderMeta = buildShaderMetadata(shader)

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
            sessionDir ?: run {
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

        captureEntries.add(
            CaptureSessionData(
                worldName = StagingWorld.FOLDER_NAME,
                sceneId = sceneId,
                sessionDir = sessionDirPath,
                startedAt = (startedAt ?: Instant.now()).toString(),
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
        log.info("Generating manifest")
        writeManifest(partial = false)

        sessionDir?.let { dir ->
            log.info("Orchestration complete") {
                "results_dir" to dir.absolutePath
                "captures" to captureEntries.size
            }
        }

        transitionTo(State.Finishing)
    }

    private fun handleFinishing() {
        log.info("Orchestration finished")
        cleanup()
    }

    // -- Navigation --

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

    // -- Environment application --

    /**
     * Applies the effective environment for a work item.
     * Preset values override scene defaults when present.
     */
    private fun applyPresetEnvironment(item: WorkItem) {
        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer ?: return
        val overworld = server.overworld()

        // Time of day
        val time = item.effectiveTimeOfDayTicks
        overworld.dayTime = time.toLong()

        // Weather
        val weather = item.effectiveWeather
        val intensity = item.effectiveWeatherIntensity
        when (weather) {
            "clear" -> overworld.setWeatherParameters(6000, 0, false, false)
            "rain" -> overworld.setWeatherParameters(0, 6000, true, false)
            "thunder" -> overworld.setWeatherParameters(0, 6000, true, true)
        }

        // Snap weather levels immediately
        applyWeatherSnap(item)

        // Moon phase (if applicable — requires setting world time to the right day)
        val moonPhase = item.effectiveMoonPhase
        if (moonPhase != null) {
            // Moon phase is derived from dayTime / 24000. To set a specific phase,
            // adjust the day count while keeping the time-of-day component.
            val dayCount = moonPhase.toLong() // Phase 0 = day 0, phase 1 = day 1, etc.
            overworld.dayTime = dayCount * 24000L + time.toLong()
        }
    }

    /** Snaps weather levels on both server and client to target values. */
    private fun applyWeatherSnap(item: WorkItem) {
        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer ?: return
        val overworld = server.overworld()

        val weather = item.effectiveWeather
        val intensity = item.effectiveWeatherIntensity.toFloat()

        val (targetRain, targetThunder) =
            when (weather) {
                "clear" -> {
                    0f to 0f
                }

                "rain" -> {
                    (if (intensity > 0f) intensity else 1f) to 0f
                }

                "thunder" -> {
                    val level = if (intensity > 0f) intensity else 1f
                    level to level
                }

                else -> {
                    0f to 0f
                }
            }

        overworld.setRainLevel(targetRain)
        overworld.setThunderLevel(targetThunder)
        mc.level?.let { clientLevel ->
            clientLevel.setRainLevel(targetRain)
            clientLevel.setThunderLevel(targetThunder)
        }
    }

    // -- Render settings --

    /**
     * Applies standardized render settings for captures.
     * Called once after the staging world is ready.
     */
    private fun applyStandardRenderSettings() {
        val mc = Minecraft.getInstance()
        val options = mc.options

        // Standard capture settings (SceneConfig.DEFAULT equivalent)
        options.graphicsMode().set(net.minecraft.client.GraphicsStatus.FANCY)
        options.ambientOcclusion().set(true)
        options.entityShadows().set(true)
        options.particles().set(net.minecraft.server.level.ParticleStatus.ALL)
        options.cloudStatus().set(net.minecraft.client.CloudStatus.FANCY)
        options.bobView().set(false)
        options.gamma().set(0.0)
        options.screenEffectScale().set(0.0)
        options.fovEffectScale().set(0.0)
        options.hideGui = true

        // Set spectator mode
        mc.singleplayerServer?.let { server ->
            val player = server.playerList.players.firstOrNull()
            if (player != null && !player.isSpectator) {
                player.setGameMode(GameType.SPECTATOR)
            }
        }

        renderSettingsApplied = true
        log.debug("Standard render settings applied")
    }

    /** Applies per-scene FOV and render distance from the work item. */
    private fun applySceneViewSettings(item: WorkItem) {
        val mc = Minecraft.getInstance()
        mc.options.fov().set(item.scene.fov)
        mc.options.renderDistance().set(item.scene.renderDistance)
        mc.options.simulationDistance().set(item.scene.renderDistance)

        log.debug("Scene view settings applied") {
            "fov" to item.scene.fov
            "render_distance" to item.scene.renderDistance
        }
    }

    /** Freezes daylight cycle and weather cycle via game rules. */
    private fun freezeWorldState() {
        val mc = Minecraft.getInstance()
        mc.singleplayerServer?.let { server ->
            server
                .overworld()
                .gameRules
                .getRule(GameRules.RULE_DAYLIGHT)
                .set(false, server)
            server
                .overworld()
                .gameRules
                .getRule(GameRules.RULE_WEATHER_CYCLE)
                .set(false, server)
            log.debug("World state frozen")
        }
    }

    // -- Shader helpers --

    private fun buildShaderMetadata(shader: ShaderSpec): com.xevion.glint.capture.ShaderMetadata? {
        if (shader.filename == null) return null
        val info = parseShaderPackName(shader.filename)
        return com.xevion.glint.capture.ShaderMetadata(
            filename = shader.filename,
            id = info.id,
            version = info.version,
            profile = shader.profile,
            profileId = shader.profileId,
        )
    }

    private fun parseShaderPackName(filename: String): ShaderPackInfo {
        val baseName = filename.removeSuffix(".zip").removeSuffix(".ZIP")
        val parts = baseName.split("-")

        return if (parts.size >= 3) {
            ShaderPackInfo(
                id = sanitize(parts.dropLast(2).joinToString("-")),
                version = sanitize(parts[parts.size - 2]),
            )
        } else if (parts.size >= 2) {
            ShaderPackInfo(
                id = sanitize(parts.dropLast(1).joinToString("-")),
                version = sanitize(parts.last()),
            )
        } else {
            ShaderPackInfo(id = sanitize(baseName), version = "unknown")
        }
    }

    private data class ShaderPackInfo(
        val id: String,
        val version: String,
    )

    // -- Capture filename --

    private fun buildCaptureFilename(
        item: WorkItem,
        shader: ShaderSpec,
    ): String {
        val scenePrefix = sanitize(item.scene.slug)
        val presetSuffix = item.preset?.slug?.let { "_${sanitize(it)}" } ?: ""

        if (shader.filename == null) {
            return "${scenePrefix}${presetSuffix}_vanilla.webp"
        }

        val info = parseShaderPackName(shader.filename)
        val profileSuffix = shader.profile?.let { "_${sanitize(it)}" } ?: ""
        return "${scenePrefix}${presetSuffix}_${info.id}_${info.version}$profileSuffix.webp"
    }

    private fun sanitize(input: String): String = input.lowercase().replace(Regex("[^a-z0-9._-]"), "-")

    // -- Session directory --

    private fun createSessionDirectory(): Boolean {
        val mc = Minecraft.getInstance()
        startedAt = Instant.now()

        return try {
            if (outputDir != null) {
                val dir = File(mc.gameDirectory, outputDir!!)
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

    // -- Manifest --

    private fun writeManifest(partial: Boolean) {
        val currentSessionDir = sessionDir ?: return
        val manifestName = if (partial) "manifest_partial.json" else "manifest.json"
        val manifestFile = File(currentSessionDir, manifestName)

        val manifest =
            OrchestrationManifest.create(
                captureEntries,
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
        } catch (e: kotlinx.serialization.SerializationException) {
            log.error(e, "Failed to serialize manifest")
        }
    }

    // -- Error + cleanup --

    private fun finishWithError(reason: String) {
        log.error("Orchestration failed") { "reason" to reason }
        if (captureEntries.isNotEmpty()) {
            writeManifest(partial = true)
        }
        cleanup()
    }

    private fun cleanup() {
        // End any active injection (must run on server thread for chunk ticket removal)
        val level = stagingWorld.getServerLevel()
        if (level != null && loadedScene != null) {
            level.server.execute { sceneInjector.deactivate(level) }
        }
        loadedScene = null
        injectionProcess = null
        serverDispatch = ServerDispatchState.Idle

        // End 4K session if active
        if (highResSessionActive) {
            HighResCapture.endSession()
            highResSessionActive = false
        }

        // Deactivate time override
        CaptureTimeOverride.deactivate()
        ChunkForceLoader.releaseAll()

        // Restore original shader
        if (IrisIntegration.isAvailable) {
            originalShaderPack?.let { IrisIntegration.enableShaders(it) }
                ?: IrisIntegration.disableShaders()
        }

        CaptureStateManager.endCapture()

        state = State.Idle
        ticksInState = 0
        workItems = emptyList()
        scenePackages = emptyMap()
        currentItemIndex = 0
        currentPackageHash = null
        currentPresetId = null
        currentShaderVersionId = null
        currentShaderSpec = null
        sessionDir = null
        sessionId = ""
        startedAt = null
        captureEntries.clear()
        failedShaderPacks.clear()
        renderSettingsApplied = false
        originalShaderPack = null
        runId = null
        outputDir = null
        pendingCapture = null
        stagingWorld.reset()
    }

    // -- Helpers --

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
