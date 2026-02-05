package com.xevion.glint.orchestration

import com.xevion.glint.Glint
import com.xevion.glint.capture.CaptureSession
import com.xevion.glint.capture.CaptureStateManager
import com.xevion.glint.io.SessionDirectoryManager
import com.xevion.glint.scene.ResolvedScene
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.screenshot.CaptureSessionData
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
    private val logger = Glint.LOGGER
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
            logger.warn("Orchestrator already running")
            return false
        }

        logger.info("Starting orchestration with ${spec.sceneIds.size} scenes, ${spec.shaders.size} shaders")
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
            logger.info("Orchestration cancelled by user")
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
        logger.info("Orchestrator: $state -> $newState")
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
                logger.error("No current world-scene pair")
                transitionTo(State.GeneratingManifest)
                return
            }

        when (val result = worldLoader.loadWorld(pair.worldFolder, ticksInState)) {
            is WorldLoader.LoadResult.Complete -> {
                logger.info("[${currentPairIndex + 1}/${worldScenePairs.size}] World ready: ${pair.worldFolder}")
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
            logger.info("[${currentPairIndex + 1}/${worldScenePairs.size}] Running scene: ${pair.sceneId}")

            val worldDir = File(currentSessionDir, pair.worldFolder)
            if (!worldDir.exists() && !worldDir.mkdirs()) {
                logger.error("Failed to create world directory: ${worldDir.absolutePath}")
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
                logger.error("Failed to start capture session for scene: ${pair.sceneId}")
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
        logger.info("Scene capture complete: ${pair.sceneId}")

        val sessionData = captureSession!!.getSessionData()
        if (sessionData != null) {
            if (sessionData.screenshots.isNotEmpty()) {
                sessionDataList.add(sessionData)
            } else {
                logger.warn("Session produced no screenshots: ${pair.sceneId}")
            }
        } else {
            logger.warn("No session data returned: ${pair.sceneId}")
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
            logger.warn("Screenshots directory not found: ${screenshotsDir.absolutePath}")
            return
        }

        if (sceneDir.exists()) {
            logger.warn("Scene directory already exists, deleting: ${sceneDir.absolutePath}")
            sceneDir.deleteRecursively()
        }

        if (!screenshotsDir.renameTo(sceneDir)) {
            logger.error("Failed to rename screenshots folder: ${screenshotsDir.absolutePath} -> ${sceneDir.absolutePath}")
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
        logger.info("Generating master manifest")
        writeManifest(partial = false)

        sessionDir?.let { dir ->
            logger.info("Orchestration complete!")
            logger.info("Results: ${dir.absolutePath}")
            logger.info("Total scenes captured: ${sessionDataList.size}")
        }

        transitionTo(State.Finishing)
    }

    private fun handleFinishing() {
        logger.info("Orchestration finished")
        cleanup()

        if (spec?.shutdownOnComplete == true) {
            logger.info("Shutting down Minecraft (shutdownOnComplete)")
            Minecraft.getInstance().stop()
        }
    }

    private fun finishWithError(reason: String) {
        logger.error("Orchestration failed: $reason")

        if (sessionDataList.isNotEmpty()) {
            writeManifest(partial = true)
        }

        cleanup()

        if (spec?.shutdownOnComplete == true) {
            logger.info("Shutting down Minecraft after failure (shutdownOnComplete)")
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
                logger.warn("Failed to load scene: $sceneId, skipping")
                continue
            }

            if (!worldLoader.worldExists(resolvedScene.worldFolderName)) {
                logger.warn("World directory not found for scene: $sceneId (world: ${resolvedScene.worldFolderName}), skipping")
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
            logger.info("Resolved ${sorted.size} scenes across $uniqueWorlds worlds")
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
                    logger.error("Failed to create output directory: ${dir.absolutePath}")
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
            logger.info("Session directory: ${sessionDir!!.absolutePath}")
            true
        } catch (e: Exception) {
            logger.error("Failed to create session directory", e)
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
            logger.info("${if (partial) "Partial " else ""}Manifest written: ${manifestFile.absolutePath}")
        } catch (e: Exception) {
            logger.error("Failed to write manifest", e)
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
