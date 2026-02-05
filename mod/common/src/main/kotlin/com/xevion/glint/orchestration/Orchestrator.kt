package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.capture.CaptureSession
import com.xevion.glint.capture.CaptureStateManager
import com.xevion.glint.debug
import com.xevion.glint.error
import com.xevion.glint.info
import com.xevion.glint.io.SessionDirectoryManager
import com.xevion.glint.scene.ResolvedScene
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.screenshot.CaptureSessionData
import com.xevion.glint.warn
import kotlinx.serialization.json.Json
import net.minecraft.client.Minecraft
import java.io.File
import java.time.Instant

/**
 * Orchestrates multi-world, multi-scene capture sessions.
 *
 * Resolves scene IDs from a [CaptureSpec], loads each world sequentially,
 * captures all scenes in that world, then moves to the next world.
 * Generates a master manifest on completion.
 */
class Orchestrator {
    private val log = Loggers.Orchestration.get()
    private val worldLoader = WorldLoader()

    private var state: State = State.Idle
    private var ticksInState: Int = 0

    private var spec: CaptureSpec? = null
    private var worldScenePairs: List<WorldScenePair> = emptyList()
    private var currentPairIndex: Int = 0

    private var captureSession: CaptureSession? = null
    private var sessionDir: File? = null
    private var sessionId: String = ""
    private var startedAt: Instant? = null
    private val sessionDataList = mutableListOf<CaptureSessionData>()

    private data class WorldScenePair(
        val worldFolder: String,
        val scene: ResolvedScene,
        val sceneId: String,
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

        CaptureStateManager.startCapture()
        transitionTo(State.DiscoveringScenes)
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

            State.DiscoveringScenes -> {
                handleDiscoveringScenes()
            }

            State.LoadingWorld -> {
                handleLoadingWorld()
            }

            State.RunningScene -> {
                handleRunningScene()
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

    private fun handleDiscoveringScenes() {
        val pairs = buildWorldScenePairs()
        if (pairs.isEmpty()) {
            finishWithError("No valid scenes found")
            return
        }

        worldScenePairs = pairs
        currentPairIndex = 0
        transitionTo(State.LoadingWorld)
    }

    private fun handleLoadingWorld() {
        val pair =
            getCurrentPair() ?: run {
                log.error("No current world-scene pair")
                transitionTo(State.GeneratingManifest)
                return
            }

        when (val result = worldLoader.loadWorld(pair.worldFolder, ticksInState)) {
            is WorldLoader.LoadResult.Complete -> {
                log.info("World ready") {
                    "progress" to "${currentPairIndex + 1}/${worldScenePairs.size}"
                    "world" to pair.worldFolder
                }
                transitionTo(State.RunningScene)
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

    private fun handleRunningScene() {
        val pair = getCurrentPair()!!
        val currentSessionDir = sessionDir!!
        val currentSpec = spec!!

        if (captureSession == null) {
            log.info("Running scene") {
                "progress" to "${currentPairIndex + 1}/${worldScenePairs.size}"
                "scene_id" to pair.sceneId
            }

            val worldDir = File(currentSessionDir, pair.worldFolder)
            if (!worldDir.exists() && !worldDir.mkdirs()) {
                log.error("Failed to create world directory") { "path" to worldDir.absolutePath }
                finishWithError("Failed to create world directory")
                return
            }

            captureSession =
                CaptureSession(
                    sceneId = pair.sceneId,
                    shaders = currentSpec.shaders,
                    outputDir = worldDir,
                    worldName = pair.worldFolder,
                )

            if (!captureSession!!.start()) {
                log.error("Failed to start capture session") { "scene_id" to pair.sceneId }
                captureSession = null
                finishWithError("Failed to start capture session")
                return
            }
        }

        captureSession!!.tick()

        if (!captureSession!!.isRunning) {
            finalizeCaptureSession(pair, currentSessionDir)
            advanceToNextPair()
        }
    }

    private fun finalizeCaptureSession(
        pair: WorldScenePair,
        currentSessionDir: File,
    ) {
        log.info("Scene capture complete") { "scene_id" to pair.sceneId }

        val sessionData = captureSession!!.getSessionData()
        if (sessionData != null) {
            if (sessionData.screenshots.isNotEmpty()) {
                sessionDataList.add(sessionData)
            } else {
                log.warn("Session produced no screenshots") { "scene_id" to pair.sceneId }
            }
        } else {
            log.warn("No session data returned") { "scene_id" to pair.sceneId }
        }

        renameScreenshotsDirectory(pair, currentSessionDir)
        captureSession = null
    }

    private fun renameScreenshotsDirectory(
        pair: WorldScenePair,
        currentSessionDir: File,
    ) {
        val worldDir = File(currentSessionDir, pair.worldFolder)
        val screenshotsDir = File(worldDir, "screenshots")
        val sceneDir = File(worldDir, pair.sceneId)

        if (!screenshotsDir.exists()) {
            log.warn("Screenshots directory not found") { "path" to screenshotsDir.absolutePath }
            return
        }

        if (sceneDir.exists()) {
            log.warn("Scene directory already exists, deleting") { "path" to sceneDir.absolutePath }
            sceneDir.deleteRecursively()
        }

        if (!screenshotsDir.renameTo(sceneDir)) {
            log.error("Failed to rename screenshots folder") {
                "from" to screenshotsDir.absolutePath
                "to" to sceneDir.absolutePath
            }
        }
    }

    private fun advanceToNextPair() {
        currentPairIndex++

        if (currentPairIndex >= worldScenePairs.size) {
            transitionTo(State.GeneratingManifest)
        } else {
            val currentWorld = worldScenePairs[currentPairIndex - 1].worldFolder
            val nextWorld = worldScenePairs[currentPairIndex].worldFolder

            if (currentWorld != nextWorld) {
                transitionTo(State.LoadingWorld)
            } else {
                transitionTo(State.RunningScene)
            }
        }
    }

    private fun handleGeneratingManifest() {
        log.info("Generating master manifest")
        writeManifest(partial = false)

        sessionDir?.let { dir ->
            log.info("Orchestration complete") {
                "results_dir" to dir.absolutePath
                "scenes_captured" to sessionDataList.size
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
        CaptureStateManager.endCapture()

        state = State.Idle
        ticksInState = 0
        worldScenePairs = emptyList()
        currentPairIndex = 0
        sessionDir = null
        sessionId = ""
        startedAt = null
        sessionDataList.clear()
        worldLoader.reset()
        spec = null
    }

    /**
     * Resolves scene IDs from the spec into (world, scene) pairs, grouped by world
     * and preserving scene order.
     */
    private fun buildWorldScenePairs(): List<WorldScenePair> {
        val currentSpec = spec ?: return emptyList()

        val pairs = mutableListOf<WorldScenePair>()
        for (sceneId in currentSpec.sceneIds) {
            val resolvedScene = SceneManager.loadScene(sceneId)
            if (resolvedScene == null) {
                log.warn("Failed to load scene, skipping") { "scene_id" to sceneId }
                continue
            }

            if (!worldLoader.worldExists(resolvedScene.worldFolderName)) {
                log.warn("World directory not found, skipping scene") {
                    "scene_id" to sceneId
                    "world" to resolvedScene.worldFolderName
                }
                continue
            }

            pairs.add(WorldScenePair(resolvedScene.worldFolderName, resolvedScene, sceneId))
        }

        // Group by world to minimize world loads, preserving first-seen order
        val worldOrder = pairs.map { it.worldFolder }.distinct()
        val grouped = pairs.groupBy { it.worldFolder }
        val sorted = worldOrder.flatMap { world -> grouped[world] ?: emptyList() }

        if (sorted.isNotEmpty()) {
            val uniqueWorlds = sorted.map { it.worldFolder }.distinct().size
            log.info("Resolved scenes") {
                "scene_count" to sorted.size
                "world_count" to uniqueWorlds
            }
        }

        return sorted
    }

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
                sessionId = currentSpec.jobId?.let { "job_$it" } ?: dir.name
                sessionDir = dir
            } else {
                val capturesDir = File(mc.gameDirectory, "glint/captures")
                val (dir, id) = SessionDirectoryManager.createSessionDirectory(capturesDir, startedAt!!)
                sessionId = id
                sessionDir = dir
            }
            log.info("Session directory created") { "path" to sessionDir!!.absolutePath }
            true
        } catch (e: Exception) {
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
                jobId = spec?.jobId,
            )

        try {
            val json = Json { prettyPrint = true }
            manifestFile.writeText(json.encodeToString(OrchestrationManifest.serializer(), manifest))
            log.info("Manifest written") {
                "partial" to partial
                "path" to manifestFile.absolutePath
            }
        } catch (e: Exception) {
            log.error(e, "Failed to write manifest")
        }
    }

    private fun getCurrentPair(): WorldScenePair? = worldScenePairs.getOrNull(currentPairIndex)

    private enum class State {
        Idle,
        DiscoveringScenes,
        LoadingWorld,
        RunningScene,
        GeneratingManifest,
        Finishing,
    }
}
