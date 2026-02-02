package com.xevion.glint.capture

import com.xevion.glint.Glint
import net.minecraft.client.Minecraft
import net.minecraft.world.level.chunk.EmptyLevelChunk
import net.minecraft.world.level.chunk.status.ChunkStatus

/**
 * Detects when the world has stabilized after shader changes.
 *
 * Checks multiple conditions:
 * 1. All chunks in render distance are loaded
 * 2. Lighting calculations are complete
 * 3. Chunk rendering is complete (vanilla or Sodium)
 * 4. GPU shader compilation has settled
 */
class StabilizationDetector(
    private val settlingTicks: Int = DEFAULT_SETTLING_TICKS,
) {
    private var ticksSinceStable: Int = 0
    private var graphUpdateTicks: Int = 0

    /**
     * Reset state for a new stabilization cycle.
     */
    fun reset() {
        ticksSinceStable = 0
        graphUpdateTicks = 0
        SodiumIntegration.resetStabilizationState()
    }

    /**
     * Check if the world has stabilized.
     *
     * @param ticksInState Total ticks spent in current stabilization phase (for logging)
     * @return true if stable, false if still waiting
     */
    fun isStable(ticksInState: Int): Boolean {
        val mc = Minecraft.getInstance()
        val level = mc.level
        val player = mc.player

        if (level == null || player == null) {
            Glint.LOGGER.warn("Level or player not available during stabilization")
            return false
        }

        val logInterval = ticksInState % LOG_INTERVAL_TICKS == 0

        if (!areChunksLoaded(mc, logInterval)) {
            ticksSinceStable = 0
            return false
        }

        if (!isLightingComplete(level, logInterval)) {
            ticksSinceStable = 0
            return false
        }

        if (!isRenderingComplete(mc, logInterval)) {
            ticksSinceStable = 0
            return false
        }

        ticksSinceStable++
        if (ticksSinceStable >= settlingTicks) {
            val loadedChunks = countLoadedChunks(mc)
            Glint.LOGGER.info(
                "Stabilization complete after $ticksInState ticks total ($ticksSinceStable ticks stable, $loadedChunks chunks loaded)",
            )
            return true
        }

        return false
    }

    private fun areChunksLoaded(
        mc: Minecraft,
        logInterval: Boolean,
    ): Boolean {
        val player = mc.player ?: return false
        val level = mc.level ?: return false
        val chunkSource = level.chunkSource

        val chunkX = player.blockPosition().x shr 4
        val chunkZ = player.blockPosition().z shr 4
        val renderDistance = mc.options.effectiveRenderDistance
        val radiusSquared = renderDistance * renderDistance

        var totalChunks = 0
        var loadedChunks = 0

        for (dx in -renderDistance..renderDistance) {
            for (dz in -renderDistance..renderDistance) {
                val distSquared = dx * dx + dz * dz
                if (distSquared > radiusSquared) {
                    continue
                }

                totalChunks++
                val chunk = chunkSource.getChunk(chunkX + dx, chunkZ + dz, ChunkStatus.FULL, false)

                if (chunk != null && chunk !is EmptyLevelChunk) {
                    loadedChunks++
                }
            }
        }

        if (loadedChunks < totalChunks) {
            if (logInterval) {
                Glint.LOGGER.debug("Stabilize: chunks loaded $loadedChunks/$totalChunks (rd=$renderDistance)")
            }
            return false
        }

        return true
    }

    private fun countLoadedChunks(mc: Minecraft): Int {
        val player = mc.player ?: return 0
        val level = mc.level ?: return 0
        val chunkSource = level.chunkSource

        val chunkX = player.blockPosition().x shr 4
        val chunkZ = player.blockPosition().z shr 4
        val renderDistance = mc.options.effectiveRenderDistance
        val radiusSquared = renderDistance * renderDistance

        var loadedChunks = 0

        for (dx in -renderDistance..renderDistance) {
            for (dz in -renderDistance..renderDistance) {
                val distSquared = dx * dx + dz * dz
                if (distSquared > radiusSquared) {
                    continue
                }

                val chunk = chunkSource.getChunk(chunkX + dx, chunkZ + dz, ChunkStatus.FULL, false)
                if (chunk != null && chunk !is EmptyLevelChunk) {
                    loadedChunks++
                }
            }
        }

        return loadedChunks
    }

    private fun isLightingComplete(
        level: net.minecraft.world.level.Level,
        logInterval: Boolean,
    ): Boolean {
        val lightEngine = level.chunkSource.lightEngine
        val hasLightWork = lightEngine.hasLightWork()

        if (hasLightWork) {
            if (logInterval) {
                Glint.LOGGER.debug("Stabilize: lighting still running")
            }
            return false
        }

        return true
    }

    private fun isRenderingComplete(
        mc: Minecraft,
        logInterval: Boolean,
    ): Boolean {
        val levelRenderer = mc.levelRenderer
        val sodiumComplete = SodiumIntegration.isRenderingComplete()

        if (sodiumComplete != null) {
            if (!sodiumComplete) {
                val scheduledJobs = SodiumIntegration.getScheduledJobCount() ?: 0
                val busyThreads = SodiumIntegration.getBusyThreadCount() ?: 0
                val totalThreads = SodiumIntegration.getTotalThreadCount() ?: 0
                val needsUpdate = SodiumIntegration.needsGraphUpdate() ?: false

                if (needsUpdate) {
                    graphUpdateTicks++
                } else {
                    graphUpdateTicks = 0
                }

                if (logInterval) {
                    when {
                        scheduledJobs > 0 -> {
                            Glint.LOGGER.debug(
                                "Stabilize: Building chunks ($scheduledJobs queued, $busyThreads/$totalThreads threads active)",
                            )
                        }

                        needsUpdate -> {
                            Glint.LOGGER.debug("Stabilize: Graph update in progress (tick $graphUpdateTicks)")
                        }

                        busyThreads > 0 -> {
                            Glint.LOGGER.debug("Stabilize: Finishing builds ($busyThreads/$totalThreads threads active)")
                        }
                    }
                }
            }
            return sodiumComplete
        } else {
            return levelRenderer.hasRenderedAllSections()
        }

        return true
    }

    companion object {
        private const val DEFAULT_SETTLING_TICKS = 10
        private const val LOG_INTERVAL_TICKS = 10
    }
}
