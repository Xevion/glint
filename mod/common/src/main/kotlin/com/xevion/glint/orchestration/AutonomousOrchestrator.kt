package com.xevion.glint.orchestration

import com.xevion.glint.Glint
import com.xevion.glint.capture.CaptureSession
import com.xevion.glint.capture.CaptureState
import com.xevion.glint.scene.ResolvedScene
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.screenshot.CaptureSessionData
import net.minecraft.client.Minecraft
import java.io.File
import java.time.Instant
import java.time.format.DateTimeFormatter

/**
 * Orchestrates autonomous multi-world, multi-scene, multi-shader capture.
 *
 * Workflow:
 * 1. Discover all worlds with scene definitions
 * 2. For each world:
 *    a. Load world
 *    b. For each scene in world:
 *       - Run CaptureSession (all shaders)
 *    c. Unload world
 * 3. Generate master manifest
 * 4. Exit Minecraft cleanly
 */
class AutonomousOrchestrator {
    private val logger = Glint.LOGGER
    private var state: State = State.Idle
    private var ticksInState: Int = 0

    private var worldScenePairs: List<WorldScenePair> = emptyList()
    private var currentPairIndex: Int = 0
    private var captureSession: CaptureSession? = null

    private var sessionDir: File? = null
    private var sessionId: String = ""
    private var startedAt: Instant? = null

    private val sessionDataList = mutableListOf<CaptureSessionData>()

    /**
     * Represents a world + scene pair to capture.
     */
    private data class WorldScenePair(
        val worldName: String,
        val scene: ResolvedScene,
        val sceneId: String,
    )

    /**
     * Starts autonomous orchestration.
     * @return true if started successfully, false if already running or no scenes found
     */
    fun start(): Boolean {
        if (state != State.Idle) {
            logger.warn("Autonomous orchestrator already running")
            return false
        }

        logger.info("Starting autonomous capture orchestration")
        startedAt = Instant.now()

        // Create master session directory
        sessionId =
            DateTimeFormatter
                .ofPattern("yyyy-MM-dd_HH-mm-ss")
                .withZone(java.time.ZoneId.systemDefault())
                .format(startedAt)
        val mc = Minecraft.getInstance()
        val createdDir = File(mc.gameDirectory, "glint/captures/auto_$sessionId")
        if (!createdDir.mkdirs()) {
            logger.error("Failed to create autonomous session directory: ${createdDir.absolutePath}")
            return false
        }
        sessionDir = createdDir
        logger.info("Autonomous session directory: ${createdDir.absolutePath}")

        transitionTo(State.DiscoveringScenes)
        return true
    }

    /**
     * Called every client tick to advance orchestration state.
     */
    fun tick() {
        if (state == State.Idle) return

        ticksInState++

        when (state) {
            State.Idle -> {}

            State.DiscoveringScenes -> {
                handleDiscoveringScenes()
            }

            State.LoadingWorld -> {
                handleLoadingWorld()
            }

            State.WaitingWorldLoad -> {
                handleWaitingWorldLoad()
            }

            State.RunningScene -> {
                handleRunningScene()
            }

            State.UnloadingWorld -> {
                handleUnloadingWorld()
            }

            State.GeneratingManifest -> {
                handleGeneratingManifest()
            }

            State.Exiting -> {
                handleExiting()
            }
        }
    }

    val isRunning: Boolean
        get() = state != State.Idle

    private fun transitionTo(newState: State) {
        logger.info("Orchestrator state: $state -> $newState")
        state = newState
        ticksInState = 0
    }

    private fun handleDiscoveringScenes() {
        val mc = Minecraft.getInstance()
        val savesDir = File(mc.gameDirectory, "saves")

        if (!savesDir.exists() || !savesDir.isDirectory) {
            logger.error("Saves directory not found: ${savesDir.absolutePath}")
            abortWithError("Saves directory not found")
            return
        }

        // Discover all scene collections using SceneManager
        val collections = SceneManager.discoverAllCollections()

        if (collections.isEmpty()) {
            logger.error("No scene collections found in glint/scenes/")
            abortWithError("No scene definitions found")
            return
        }

        logger.info("Found ${collections.size} scene collections")

        // Build world-scene pairs, expanding variants
        val pairs = mutableListOf<WorldScenePair>()
        for ((_, collection) in collections) {
            for (scene in collection.scenes) {
                // Process base scene
                val resolvedScene = SceneManager.loadScene(scene.id)
                if (resolvedScene == null) {
                    logger.warn("Failed to load scene: ${scene.id}")
                    continue
                }

                // Verify world exists in saves/ and matches collection world
                if (resolvedScene.worldName != collection.world) {
                    logger.warn("Scene world mismatch: expected ${collection.world}, got ${resolvedScene.worldName} for scene ${scene.id}")
                    continue
                }

                val worldDir = File(savesDir, resolvedScene.worldName)
                if (!worldDir.exists() || !worldDir.isDirectory) {
                    logger.warn(
                        "World directory not found for scene: ${scene.id} (world: ${resolvedScene.worldName}, expected: ${worldDir.absolutePath})",
                    )
                    continue
                }

                pairs.add(WorldScenePair(resolvedScene.worldName, resolvedScene, scene.id))
                logger.debug("Added base scene: ${scene.id} (world: ${resolvedScene.worldName})")

                // Process variants
                for (variant in scene.variants) {
                    val resolvedVariant = SceneManager.loadScene(variant.id)
                    if (resolvedVariant == null) {
                        logger.warn("Failed to load variant: ${variant.id}")
                        continue
                    }

                    pairs.add(WorldScenePair(resolvedVariant.worldName, resolvedVariant, variant.id))
                    logger.debug("Added variant: ${variant.id} (world: ${resolvedVariant.worldName})")
                }
            }
        }

        if (pairs.isEmpty()) {
            logger.error("No valid world-scene pairs discovered")
            abortWithError("No valid scenes found")
            return
        }

        worldScenePairs = pairs
        currentPairIndex = 0

        val uniqueWorlds = pairs.map { it.worldName }.distinct().size
        logger.info("Discovered ${pairs.size} scenes across $uniqueWorlds worlds")
        logger.info("Scene list: ${pairs.joinToString(", ") { it.sceneId }}")

        transitionTo(State.LoadingWorld)
    }

    private fun handleLoadingWorld() {
        val pair = getCurrentPair()
        if (pair == null) {
            logger.error("No current world-scene pair available")
            transitionTo(State.GeneratingManifest)
            return
        }

        val mc = Minecraft.getInstance()
        val currentWorldName = mc.singleplayerServer?.worldData?.levelName

        // If already in the correct world, skip loading
        if (currentWorldName == pair.worldName) {
            logger.info("Already in world: ${pair.worldName}, proceeding to scene")
            transitionTo(State.WaitingWorldLoad)
            return
        }

        // Disconnect from current world if loaded
        if (mc.level != null) {
            logger.info("Disconnecting from current world")
            mc.disconnect()
        }

        logger.info("[${currentPairIndex + 1}/${worldScenePairs.size}] Loading world: ${pair.worldName}")

        // Load the world using WorldOpenFlows.openWorld
        mc.createWorldOpenFlows().openWorld(pair.worldName) {
            logger.debug("World load callback for: ${pair.worldName}")
        }

        transitionTo(State.WaitingWorldLoad)
    }

    private fun handleWaitingWorldLoad() {
        val mc = Minecraft.getInstance()
        val pair = getCurrentPair() ?: return

        // Wait for world to finish loading
        if (mc.level == null || mc.singleplayerServer == null) {
            if (ticksInState > WORLD_LOAD_TIMEOUT_TICKS) {
                logger.error("World load timeout: ${pair.worldName}")
                abortWithError("World load timeout")
            }
            return
        }

        // Verify we're in the correct world
        val currentWorldName = mc.singleplayerServer?.worldData?.levelName
        if (currentWorldName != pair.worldName) {
            logger.error("Loaded wrong world: expected ${pair.worldName}, got $currentWorldName")
            abortWithError("World mismatch")
            return
        }

        // Wait a few extra ticks for world to stabilize
        if (ticksInState >= WORLD_STABILIZATION_TICKS) {
            logger.info("World loaded successfully: ${pair.worldName}")
            transitionTo(State.RunningScene)
        }
    }

    private fun handleRunningScene() {
        val pair = getCurrentPair() ?: return
        val currentSessionDir = sessionDir ?: return

        // Create and start capture session if not already running
        if (captureSession == null) {
            logger.info(
                "[${currentPairIndex + 1}/${worldScenePairs.size}] Running scene: ${pair.sceneId}",
            )

            // Create world directory - Screenshot.grab creates screenshots/ subdirectory
            val worldDir = File(currentSessionDir, pair.worldName)
            if (!worldDir.exists() && !worldDir.mkdirs()) {
                logger.error("Failed to create world directory: ${worldDir.absolutePath}")
                abortWithError("Failed to create world directory")
                return
            }

            captureSession =
                CaptureSession(
                    sceneId = pair.sceneId,
                    outputDir = worldDir,
                    worldName = pair.worldName,
                )

            if (!captureSession!!.start()) {
                logger.error("Failed to start capture session for scene: ${pair.sceneId}")
                abortWithError("Failed to start capture session")
                return
            }
        }

        // Tick the capture session
        captureSession?.tick()

        // Check if session completed
        if (captureSession?.isRunning == false) {
            logger.info("Scene capture complete: ${pair.sceneId}")

            // Collect session data for master manifest
            val sessionData = captureSession?.getSessionData()
            if (sessionData != null) {
                if (sessionData.screenshots.isNotEmpty()) {
                    sessionDataList.add(sessionData)
                } else {
                    logger.warn("Session produced no screenshots: ${pair.sceneId}")
                }
            } else {
                logger.warn("No session data returned from capture session: ${pair.sceneId}")
            }

            // Rename screenshots/ folder to scene ID
            val worldDir = File(currentSessionDir, pair.worldName)
            val screenshotsDir = File(worldDir, "screenshots")
            val sceneDir = File(worldDir, pair.sceneId)

            if (screenshotsDir.exists()) {
                if (sceneDir.exists()) {
                    logger.warn("Scene directory already exists, deleting: ${sceneDir.absolutePath}")
                    sceneDir.deleteRecursively()
                }
                if (!screenshotsDir.renameTo(sceneDir)) {
                    logger.error("Failed to rename screenshots folder: ${screenshotsDir.absolutePath} -> ${sceneDir.absolutePath}")
                    logger.warn("Continuing with screenshots in original location")
                }
            } else {
                logger.warn("Screenshots directory not found: ${screenshotsDir.absolutePath}")
            }

            captureSession = null
            advanceToNextPair()
        }
    }

    private fun advanceToNextPair() {
        currentPairIndex++

        if (currentPairIndex >= worldScenePairs.size) {
            // All scenes complete
            transitionTo(State.GeneratingManifest)
        } else {
            // Check if next scene is in a different world
            val currentWorld = worldScenePairs[currentPairIndex - 1].worldName
            val nextWorld = worldScenePairs[currentPairIndex].worldName

            if (currentWorld != nextWorld) {
                transitionTo(State.UnloadingWorld)
            } else {
                // Same world, proceed to next scene
                transitionTo(State.RunningScene)
            }
        }
    }

    private fun handleUnloadingWorld() {
        val mc = Minecraft.getInstance()

        // Disconnect from current world
        logger.info("Unloading world")
        mc.disconnect()

        // Wait a few ticks for cleanup
        if (ticksInState >= WORLD_UNLOAD_WAIT_TICKS) {
            transitionTo(State.LoadingWorld)
        }
    }

    private fun handleGeneratingManifest() {
        logger.info("Generating master manifest")

        val manifest = OrchestrationManifest.create(sessionDataList, sessionId, startedAt ?: Instant.now())
        val currentSessionDir = sessionDir
        if (currentSessionDir == null) {
            logger.error("Session directory is null, cannot write manifest")
            transitionTo(State.Exiting)
            return
        }

        val manifestFile = File(currentSessionDir, "manifest.json")

        try {
            val json = kotlinx.serialization.json.Json { prettyPrint = true }
            manifestFile.writeText(json.encodeToString(OrchestrationManifest.serializer(), manifest))
            logger.info("Master manifest written: ${manifestFile.absolutePath}")
        } catch (e: Exception) {
            logger.error("Failed to write master manifest", e)
        }

        logger.info("Autonomous capture complete!")
        logger.info("Results: ${currentSessionDir.absolutePath}")
        logger.info("Total scenes captured: ${sessionDataList.size}")

        transitionTo(State.Exiting)
    }

    private fun handleExiting() {
        logger.info("Exiting Minecraft")

        // Clean shutdown
        Minecraft.getInstance().stop()

        state = State.Idle
    }

    private fun abortWithError(reason: String) {
        logger.error("Autonomous capture aborted: $reason")

        // Cleanup active capture session
        captureSession = null
        CaptureState.endCapture()

        // Write partial manifest if any sessions completed
        if (sessionDataList.isNotEmpty()) {
            val currentSessionDir = sessionDir
            if (currentSessionDir != null) {
                val manifest = OrchestrationManifest.create(sessionDataList, sessionId, startedAt ?: Instant.now())
                val manifestFile = File(currentSessionDir, "manifest_partial.json")
                try {
                    val json = kotlinx.serialization.json.Json { prettyPrint = true }
                    manifestFile.writeText(json.encodeToString(OrchestrationManifest.serializer(), manifest))
                    logger.info("Partial manifest written: ${manifestFile.absolutePath}")
                } catch (e: Exception) {
                    logger.error("Failed to write partial manifest", e)
                }
            }
        }

        // Exit with error code
        Runtime.getRuntime().halt(1)
    }

    private fun getCurrentPair(): WorldScenePair? = worldScenePairs.getOrNull(currentPairIndex)

    private enum class State {
        Idle,
        DiscoveringScenes,
        LoadingWorld,
        WaitingWorldLoad,
        RunningScene,
        UnloadingWorld,
        GeneratingManifest,
        Exiting,
    }

    companion object {
        private const val WORLD_LOAD_TIMEOUT_TICKS = 20 * 30 // 30 seconds
        private const val WORLD_STABILIZATION_TICKS = 20 * 2 // 2 seconds
        private const val WORLD_UNLOAD_WAIT_TICKS = 20 // 1 second
    }
}
