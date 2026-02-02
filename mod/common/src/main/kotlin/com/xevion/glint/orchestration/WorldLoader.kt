package com.xevion.glint.orchestration

import com.xevion.glint.Glint
import net.minecraft.client.Minecraft
import java.io.File

/**
 * Manages world loading, unloading, and state detection.
 *
 * Provides a clean interface for orchestrators to transition between worlds
 * without dealing with fragile state detection logic.
 */
class WorldLoader {
    private val logger = Glint.LOGGER
    private var loadRequested = false
    private var unloadRequested = false
    private var waitStartTick = 0

    /**
     * Attempts to load a world, transitioning from current state.
     *
     * @return LoadResult indicating current progress
     */
    fun loadWorld(
        worldName: String,
        ticksWaiting: Int,
    ): LoadResult {
        val mc = Minecraft.getInstance()

        if (isInWorld(worldName)) {
            return LoadResult.Complete
        }

        if (!isAtTitleScreen()) {
            if (!unloadRequested) {
                logger.info("Unloading current world to load: $worldName")
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
            logger.info("Loading world: $worldName")
            mc.createWorldOpenFlows().openWorld(worldName) {
                logger.debug("World load cancelled for: $worldName")
            }
            loadRequested = true
            waitStartTick = ticksWaiting
            return LoadResult.LoadingWorld
        }

        if (isWorldLoaded() && isInWorld(worldName)) {
            logger.info("World loaded: $worldName")
            reset()
            return LoadResult.Complete
        }

        if (hasTimedOut(ticksWaiting, waitStartTick)) {
            return LoadResult.Timeout("Load timeout for: $worldName")
        }

        return LoadResult.LoadingWorld
    }

    /**
     * Checks if a world directory exists in saves/ or glint/worlds/.
     */
    fun worldExists(worldName: String): Boolean {
        val worldPath = resolveWorldPath(worldName)
        return worldPath != null && worldPath.exists() && worldPath.isDirectory
    }

    /**
     * Resolves world path, checking glint/worlds/ first, then saves/.
     */
    private fun resolveWorldPath(worldName: String): File? {
        val mc = Minecraft.getInstance()

        val glintWorld = File(mc.gameDirectory, "glint/worlds/$worldName")
        if (glintWorld.exists() && glintWorld.isDirectory) {
            return glintWorld
        }

        val savesWorld = File(mc.gameDirectory, "saves/$worldName")
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

    private fun isInWorld(worldName: String): Boolean {
        val mc = Minecraft.getInstance()
        return mc.level != null &&
            mc.singleplayerServer != null &&
            mc.singleplayerServer?.worldData?.levelName == worldName
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
    }

    companion object {
        private const val TIMEOUT_TICKS = 140 // 7 seconds
    }
}
