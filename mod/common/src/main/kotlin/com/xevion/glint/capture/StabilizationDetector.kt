package com.xevion.glint.capture

import com.xevion.glint.Loggers
import net.minecraft.client.Minecraft
import net.minecraft.world.level.chunk.EmptyLevelChunk
import net.minecraft.world.level.chunk.status.ChunkStatus

/**
 * Detects when the world has stabilized after shader or scene changes.
 *
 * Simple approach: is the renderer busy? Wait. Is everything idle? Settle, then approve.
 * No modes, no flags — just polls the actual state of the engine each tick.
 */
class StabilizationDetector {
    private val log = Loggers.Capture.get()

    private var ticksSinceIdle: Int = 0
    private var forceLoadingInitiated: Boolean = false

    fun reset() {
        ticksSinceIdle = 0
        forceLoadingInitiated = false
    }

    /**
     * Check if the world has stabilized. Call once per tick.
     *
     * @param ticksInState Total ticks spent in current stabilization phase (for logging)
     * @return true if stable, false if still waiting
     */
    fun isStable(ticksInState: Int): Boolean {
        val mc = Minecraft.getInstance()
        val level = mc.level ?: return false
        val player = mc.player ?: return false

        val logInterval = ticksInState % LOG_INTERVAL_TICKS == 0

        // Enforce a minimum wait so the engine has time to react to teleports/changes.
        if (ticksInState < MIN_WAIT_TICKS) {
            if (logInterval) {
                log.debug("Stabilize: minimum wait") {
                    "ticks" to ticksInState
                    "required" to MIN_WAIT_TICKS
                }
            }
            return false
        }

        // Force-load chunks once per cycle (singleplayer only)
        if (!forceLoadingInitiated && mc.singleplayerServer != null) {
            ChunkForceLoader.forceLoadRenderDistance()
            forceLoadingInitiated = true
        }

        // Check if anything is still working
        val chunksReady = areChunksReady(mc, logInterval)
        val lightingDone = !level.chunkSource.lightEngine.hasLightWork()
        val renderingDone = isRenderingIdle(mc, logInterval)

        if (!chunksReady || !lightingDone || !renderingDone) {
            ticksSinceIdle = 0

            if (logInterval) {
                if (!lightingDone) log.debug("Stabilize: lighting in progress")
            }

            return false
        }

        // Everything is idle — accumulate settling ticks
        ticksSinceIdle++

        if (ticksSinceIdle >= SETTLING_TICKS) {
            log.info("Stabilization complete") {
                "total_ticks" to ticksInState
                "idle_ticks" to ticksSinceIdle
                "chunks_loaded" to countLoadedChunks(mc)
            }
            return true
        }

        return false
    }

    /** Are all chunks around the player's position loaded? */
    private fun areChunksReady(
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
                if (dx * dx + dz * dz > radiusSquared) continue
                totalChunks++
                val chunk = chunkSource.getChunk(chunkX + dx, chunkZ + dz, ChunkStatus.FULL, false)
                if (chunk != null && chunk !is EmptyLevelChunk) {
                    loadedChunks++
                }
            }
        }

        if (loadedChunks < totalChunks && logInterval) {
            log.debug("Stabilize: chunks loading") {
                "loaded" to loadedChunks
                "total" to totalChunks
            }
        }

        return loadedChunks >= totalChunks
    }

    /**
     * Is Sodium's rendering pipeline idle?
     *
     * Checks the build queue, scheduled jobs, busy threads, and graph update flag.
     * Falls back to vanilla's [net.minecraft.client.renderer.LevelRenderer.hasRenderedAllSections]
     * if Sodium is not available.
     */
    private fun isRenderingIdle(
        mc: Minecraft,
        logInterval: Boolean,
    ): Boolean {
        if (!SodiumIntegration.isAvailable()) {
            return mc.levelRenderer.hasRenderedAllSections()
        }

        val needsUpdate = SodiumIntegration.needsGraphUpdate() ?: return true
        val scheduledJobs = SodiumIntegration.getScheduledJobCount() ?: 0
        val busyThreads = SodiumIntegration.getBusyThreadCount() ?: 0
        val queueEmpty = SodiumIntegration.isBuildQueueEmpty() ?: true

        val busy = needsUpdate || scheduledJobs > 0 || busyThreads > 0 || !queueEmpty

        if (busy && logInterval) {
            log.debug("Stabilize: renderer busy") {
                "needs_update" to needsUpdate
                "queued" to scheduledJobs
                "busy_threads" to busyThreads
                "queue_empty" to queueEmpty
            }
        }

        return !busy
    }

    private fun countLoadedChunks(mc: Minecraft): Int {
        val player = mc.player ?: return 0
        val level = mc.level ?: return 0
        val chunkSource = level.chunkSource

        val chunkX = player.blockPosition().x shr 4
        val chunkZ = player.blockPosition().z shr 4
        val renderDistance = mc.options.effectiveRenderDistance
        val radiusSquared = renderDistance * renderDistance

        var count = 0
        for (dx in -renderDistance..renderDistance) {
            for (dz in -renderDistance..renderDistance) {
                if (dx * dx + dz * dz > radiusSquared) continue
                val chunk = chunkSource.getChunk(chunkX + dx, chunkZ + dz, ChunkStatus.FULL, false)
                if (chunk != null && chunk !is EmptyLevelChunk) count++
            }
        }
        return count
    }

    companion object {
        private const val LOG_INTERVAL_TICKS = 10
        private const val MIN_WAIT_TICKS = 20 // 1 second floor
        private const val SETTLING_TICKS = 10 // 0.5 seconds of idle before approving
    }
}
