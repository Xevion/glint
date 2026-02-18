package com.xevion.glint.capture

import com.xevion.glint.Loggers
import com.xevion.glint.withMinecraftContext
import net.minecraft.client.Minecraft
import net.minecraft.world.level.chunk.EmptyLevelChunk
import net.minecraft.world.level.chunk.status.ChunkStatus

/**
 * Phased stabilization detector that gates on known preconditions in order.
 *
 * Phases advance forward only — once a precondition is met, it's never rechecked.
 * This eliminates the convergence problem where new chunks arriving would reset
 * an idle counter, causing indefinite re-polling.
 *
 * Phases:
 * 1. [Phase.ServerGeneration] — server has generated all chunks in render distance
 * 2. [Phase.ChunkReceipt] — client has received all chunks from server
 * 3. [Phase.RenderIdle] — Sodium build queue, lighting engine, and render graph are idle
 * 4. [Phase.Settling] — all checks pass simultaneously for a short settle period
 */
class StabilizationDetector {
    private val log = Loggers.Capture.get()

    private var phase: Phase = Phase.ServerGeneration
    private var totalTicks: Int = 0
    private var ticksInPhase: Int = 0
    private var settlingTicks: Int = 0

    fun reset() {
        phase = Phase.ServerGeneration
        totalTicks = 0
        ticksInPhase = 0
        settlingTicks = 0
    }

    /**
     * Check if the world has stabilized. Call once per tick.
     *
     * @return true if stable (all phases complete), false if still waiting
     */
    fun isStable(): Boolean =
        withMinecraftContext {
            totalTicks++
            ticksInPhase++

            when (phase) {
                Phase.ServerGeneration -> tickServerGeneration()
                Phase.ChunkReceipt -> tickChunkReceipt(mc)
                Phase.RenderIdle -> tickRenderIdle(mc)
                Phase.Settling -> tickSettling(mc)
            }
        } ?: false

    // -- Phase handlers --

    private fun tickServerGeneration(): Boolean {
        if (ChunkForceLoader.isGenerationComplete()) {
            advancePhase(Phase.ChunkReceipt)
            return false
        }

        if (ticksInPhase >= SERVER_GEN_TIMEOUT_TICKS) {
            log.warn("ServerGeneration timed out, advancing") {
                "ticks" to ticksInPhase
                "timeout" to SERVER_GEN_TIMEOUT_TICKS
            }
            advancePhase(Phase.ChunkReceipt)
            return false
        }

        if (ticksInPhase % LOG_INTERVAL_TICKS == 0) {
            log.debug("Stabilize [ServerGeneration]: waiting for server chunk generation") {
                "ticks" to ticksInPhase
                "generation_complete" to ChunkForceLoader.isGenerationComplete()
            }
        }

        return false
    }

    private fun tickChunkReceipt(mc: Minecraft): Boolean {
        val (loaded, total) = countClientChunks(mc)
        val ready = loaded >= total

        if (ready) {
            advancePhase(Phase.RenderIdle)
            return false
        }

        if (ticksInPhase >= CHUNK_RECEIPT_TIMEOUT_TICKS) {
            log.warn("ChunkReceipt timed out, advancing") {
                "ticks" to ticksInPhase
                "timeout" to CHUNK_RECEIPT_TIMEOUT_TICKS
                "loaded" to loaded
                "total" to total
            }
            advancePhase(Phase.RenderIdle)
            return false
        }

        if (ticksInPhase % LOG_INTERVAL_TICKS == 0) {
            log.debug("Stabilize [ChunkReceipt]: waiting for client chunks") {
                "ticks" to ticksInPhase
                "loaded" to loaded
                "total" to total
            }
        }

        return false
    }

    private fun tickRenderIdle(mc: Minecraft): Boolean {
        val lightingDone =
            !mc.level!!
                .chunkSource.lightEngine
                .hasLightWork()
        val renderingIdle = isRenderingIdle(mc)

        if (lightingDone && renderingIdle) {
            advancePhase(Phase.Settling)
            return false
        }

        if (ticksInPhase >= RENDER_IDLE_TIMEOUT_TICKS) {
            log.warn("RenderIdle timed out, advancing") {
                "ticks" to ticksInPhase
                "timeout" to RENDER_IDLE_TIMEOUT_TICKS
                "lighting_done" to lightingDone
                "rendering_idle" to renderingIdle
            }
            advancePhase(Phase.Settling)
            return false
        }

        if (ticksInPhase % LOG_INTERVAL_TICKS == 0) {
            log.debug("Stabilize [RenderIdle]: waiting for render pipeline") {
                "ticks" to ticksInPhase
                "lighting_done" to lightingDone
                "rendering_idle" to renderingIdle
                "sodium_queued" to (SodiumIntegration.getScheduledJobCount() ?: -1)
                "sodium_busy" to (SodiumIntegration.getBusyThreadCount() ?: -1)
            }
        }

        return false
    }

    private fun tickSettling(mc: Minecraft): Boolean {
        // Re-verify all conditions still hold during settle period.
        // If anything regresses (e.g. late chunk triggers render work), reset settling.
        val lightingDone =
            !mc.level!!
                .chunkSource.lightEngine
                .hasLightWork()
        val renderingIdle = isRenderingIdle(mc)

        if (!lightingDone || !renderingIdle) {
            settlingTicks = 0

            if (ticksInPhase % LOG_INTERVAL_TICKS == 0) {
                log.debug("Stabilize [Settling]: regression, resetting settle counter") {
                    "lighting_done" to lightingDone
                    "rendering_idle" to renderingIdle
                }
            }

            return false
        }

        settlingTicks++

        if (settlingTicks >= SETTLING_TICKS) {
            val (loaded, total) = countClientChunks(mc)
            log.info("Stabilization complete") {
                "total_ticks" to totalTicks
                "chunks" to "$loaded/$total"
            }
            return true
        }

        return false
    }

    // -- Helpers --

    private fun advancePhase(next: Phase) {
        log.info("Stabilization phase: $phase → $next") {
            "phase_ticks" to ticksInPhase
            "total_ticks" to totalTicks
        }
        phase = next
        ticksInPhase = 0
    }

    /** Returns (loaded, total) chunk counts within render distance. */
    private fun countClientChunks(mc: Minecraft): Pair<Int, Int> =
        withMinecraftContext {
            val chunkSource = level.chunkSource

            val chunkX = player.blockPosition().x shr 4
            val chunkZ = player.blockPosition().z shr 4
            val renderDistance = mc.options.effectiveRenderDistance
            val radiusSquared = renderDistance * renderDistance

            var total = 0
            var loaded = 0

            for (dx in -renderDistance..renderDistance) {
                for (dz in -renderDistance..renderDistance) {
                    if (dx * dx + dz * dz > radiusSquared) continue
                    total++
                    val chunk = chunkSource.getChunk(chunkX + dx, chunkZ + dz, ChunkStatus.FULL, false)
                    if (chunk != null && chunk !is EmptyLevelChunk) {
                        loaded++
                    }
                }
            }

            Pair(loaded, total)
        } ?: Pair(0, 0)

    /**
     * Is Sodium's rendering pipeline idle?
     *
     * Checks the build queue, scheduled jobs, busy threads, and graph update flag.
     * Falls back to vanilla's [net.minecraft.client.renderer.LevelRenderer.hasRenderedAllSections]
     * if Sodium is not available.
     */
    private fun isRenderingIdle(mc: Minecraft): Boolean {
        if (!SodiumIntegration.isAvailable()) {
            return mc.levelRenderer.hasRenderedAllSections()
        }

        val needsUpdate = SodiumIntegration.needsGraphUpdate() ?: return true
        val scheduledJobs = SodiumIntegration.getScheduledJobCount() ?: 0
        val busyThreads = SodiumIntegration.getBusyThreadCount() ?: 0
        val queueEmpty = SodiumIntegration.isBuildQueueEmpty() ?: true

        return !needsUpdate && scheduledJobs == 0 && busyThreads == 0 && queueEmpty
    }

    private enum class Phase {
        ServerGeneration,
        ChunkReceipt,
        RenderIdle,
        Settling,
    }

    companion object {
        private const val LOG_INTERVAL_TICKS = 10

        private const val SERVER_GEN_TIMEOUT_TICKS = 600 // 30s
        private const val CHUNK_RECEIPT_TIMEOUT_TICKS = 200 // 10s
        private const val RENDER_IDLE_TIMEOUT_TICKS = 400 // 20s
        private const val SETTLING_TICKS = 5 // 250ms of continuous idle
    }
}
