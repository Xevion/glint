package com.xevion.glint.orchestration

import com.xevion.glint.Glint
import net.minecraft.client.Minecraft
import net.minecraft.client.gui.screens.AlertScreen
import java.io.File

/**
 * Manages world loading, unloading, and state detection.
 *
 * Provides a clean interface for orchestrators to transition between worlds
 * without dealing with fragile state detection logic.
 *
 * All world references use the save **folder name** (e.g. "ice-castle"), not
 * display names or level names.
 */
class WorldLoader {
    private val logger = Glint.LOGGER
    private var loadRequested = false
    private var unloadRequested = false
    private var waitStartTick = 0

    /**
     * Attempts to load a world by folder name, transitioning from current state.
     *
     * @param worldFolder The world save folder name (directory under saves/ or glint/worlds/)
     * @param ticksWaiting Current tick count used for timeout tracking
     * @return LoadResult indicating current progress
     */
    fun loadWorld(
        worldFolder: String,
        ticksWaiting: Int,
    ): LoadResult {
        val mc = Minecraft.getInstance()

        if (isInWorld(worldFolder)) {
            return LoadResult.Complete
        }

        // Detect error screen from a previous load attempt that failed
        if (loadRequested && mc.screen is AlertScreen) {
            logger.error("World load failed (Minecraft showed error screen): $worldFolder")
            reset()
            return LoadResult.Failed("World load failed for: $worldFolder (check saves folder)")
        }

        if (!isAtTitleScreen()) {
            if (!unloadRequested) {
                logger.info("Unloading current world to load: $worldFolder")
                mc.level?.disconnect()
                unloadRequested = true
                waitStartTick = ticksWaiting
                return LoadResult.UnloadingWorld
            }

            if (!isAtTitleScreen()) {
                if (hasTimedOut(ticksWaiting, waitStartTick)) {
                    return LoadResult.Timeout("Unload timeout")
                }
                return LoadResult.UnloadingWorld
            }

            logger.info("World unloaded")
            unloadRequested = false
        }

        if (!loadRequested) {
            // Pre-validate the world directory exists before asking Minecraft to load
            if (!worldExists(worldFolder)) {
                return LoadResult.Failed(
                    "World directory not found: $worldFolder (checked saves/ and glint/worlds/)",
                )
            }

            logger.info("Loading world: $worldFolder")
            try {
                mc.createWorldOpenFlows().openWorld(worldFolder) {
                    logger.debug("World load cancelled for: $worldFolder")
                }
            } catch (e: Exception) {
                logger.error("Exception initiating world load: $worldFolder", e)
                return LoadResult.Failed("Exception loading world: ${e.message}")
            }
            loadRequested = true
            waitStartTick = ticksWaiting
            return LoadResult.LoadingWorld
        }

        if (isWorldLoaded() && isInWorld(worldFolder)) {
            logger.info("World loaded: $worldFolder")
            reset()
            return LoadResult.Complete
        }

        if (hasTimedOut(ticksWaiting, waitStartTick)) {
            return LoadResult.Timeout("Load timeout for: $worldFolder")
        }

        return LoadResult.LoadingWorld
    }

    /**
     * Checks if a world directory exists in saves/ or glint/worlds/.
     */
    fun worldExists(worldFolder: String): Boolean {
        val worldPath = resolveWorldPath(worldFolder)
        return worldPath != null && worldPath.exists() && worldPath.isDirectory
    }

    /**
     * Resolves world path, checking glint/worlds/ first, then saves/.
     */
    private fun resolveWorldPath(worldFolder: String): File? {
        val mc = Minecraft.getInstance()

        val glintWorld = File(mc.gameDirectory, "glint/worlds/$worldFolder")
        if (glintWorld.exists() && glintWorld.isDirectory) {
            return glintWorld
        }

        val savesWorld = File(mc.gameDirectory, "saves/$worldFolder")
        if (savesWorld.exists() && savesWorld.isDirectory) {
            return savesWorld
        }

        return null
    }

    fun reset() {
        loadRequested = false
        unloadRequested = false
        waitStartTick = 0
    }

    /**
     * Checks if the player is currently in the specified world.
     *
     * Compares against the loaded world's level name from worldData, which
     * typically matches the folder name for Glint-managed worlds.
     */
    private fun isInWorld(worldFolder: String): Boolean {
        val mc = Minecraft.getInstance()
        return mc.level != null &&
            mc.singleplayerServer != null &&
            mc.singleplayerServer?.worldData?.levelName == worldFolder
    }

    private fun isWorldLoaded(): Boolean {
        val mc = Minecraft.getInstance()
        return mc.level != null && mc.singleplayerServer != null
    }

    private fun isAtTitleScreen(): Boolean {
        val mc = Minecraft.getInstance()
        return mc.level == null && mc.singleplayerServer == null
    }

    private fun hasTimedOut(
        currentTick: Int,
        startTick: Int,
    ): Boolean = (currentTick - startTick) > TIMEOUT_TICKS

    sealed interface LoadResult {
        data object Complete : LoadResult

        data object LoadingWorld : LoadResult

        data object UnloadingWorld : LoadResult

        data class Timeout(
            val reason: String,
        ) : LoadResult

        data class Failed(
            val reason: String,
        ) : LoadResult
    }

    companion object {
        private const val TIMEOUT_TICKS = 140 // 7 seconds
    }
}
