package com.xevion.glint.capture

import com.xevion.glint.Loggers
import net.minecraft.client.Minecraft
import net.minecraft.world.level.ChunkPos
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Utility for force loading chunks within render distance.
 *
 * Minecraft's integrated server doesn't always send all chunks within render distance to the client.
 * Additionally, Sodium's ChunkTracker requires a 9-chunk neighborhood (chunk + 8 neighbors) to have
 * complete status before rendering a chunk. This means edge chunks at the render distance boundary
 * will never satisfy the neighbor requirement.
 *
 * This utility forces the server to load all chunks in render distance, ensuring:
 * 1. The server sends all chunk data to the client
 * 2. Chunks have proper neighbors for Sodium's rendering requirements
 */
object ChunkForceLoader {
    private val log = Loggers.Capture.get()
    private val forcedChunks: MutableSet<Long> = ConcurrentHashMap.newKeySet()

    @Volatile
    private var forcedDimension: net.minecraft.resources.ResourceKey<net.minecraft.world.level.Level>? = null
    private val generationComplete = AtomicBoolean(false)

    /** Whether blocking server-side chunk generation has finished. */
    fun isGenerationComplete(): Boolean = generationComplete.get()

    /**
     * Force load all chunks in render distance around the player.
     * Must be called from render thread, executes on server thread.
     *
     * @return true if force loading was initiated, false if not in singleplayer or already loading
     */
    fun forceLoadRenderDistance(): Boolean {
        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer ?: return false
        val player = mc.player ?: return false

        val chunkX = player.blockPosition().x shr 4
        val chunkZ = player.blockPosition().z shr 4
        val renderDistance = mc.options.effectiveRenderDistance
        val dimension = player.level().dimension()

        generationComplete.set(false)

        // Execute on server thread
        server.execute {
            val serverLevel = server.getLevel(dimension) ?: return@execute

            forcedDimension = dimension
            var count = 0
            val allPositions = mutableListOf<Long>()

            // Phase 1: Mark all chunks as force-loaded (prevents unloading, starts async generation)
            for (dx in -renderDistance..renderDistance) {
                for (dz in -renderDistance..renderDistance) {
                    if (dx * dx + dz * dz > renderDistance * renderDistance) continue

                    val cx = chunkX + dx
                    val cz = chunkZ + dz
                    val chunkPos = ChunkPos.asLong(cx, cz)
                    allPositions.add(chunkPos)

                    if (serverLevel.setChunkForced(cx, cz, true)) {
                        forcedChunks.add(chunkPos)
                        count++
                    }
                }
            }
            log.debug("Force loaded chunks") {
                "count" to count
                "render_distance" to renderDistance
            }

            // Phase 2: Block until every chunk reaches FULL status on the server.
            // Level.getChunk(x, z) calls getChunk(x, z, FULL, true) which blocks via
            // managedBlock — this processes pending server tasks (including sending
            // completed chunk packets to the client) while waiting, so chunk generation
            // workers continue in parallel and the client receives data during the wait.
            log.debug("Blocking on server-side chunk generation") {
                "total" to allPositions.size
            }
            for (chunkPos in allPositions) {
                serverLevel.getChunk(ChunkPos.getX(chunkPos), ChunkPos.getZ(chunkPos))
            }
            log.info("Server-side chunk generation complete") {
                "total" to allPositions.size
            }
            generationComplete.set(true)
        }
        return true
    }

    /**
     * Release all force-loaded chunks.
     * Should be called when capture session ends to avoid memory leaks.
     */
    fun releaseAll() {
        if (forcedChunks.isEmpty()) return

        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer ?: return
        val dimension = forcedDimension ?: return

        val chunksToRelease = forcedChunks.toSet()
        val count = chunksToRelease.size

        server.execute {
            val serverLevel = server.getLevel(dimension) ?: return@execute

            for (chunkPos in chunksToRelease) {
                serverLevel.setChunkForced(ChunkPos.getX(chunkPos), ChunkPos.getZ(chunkPos), false)
            }
            log.debug("Released force-loaded chunks") {
                "count" to count
            }
        }

        forcedChunks.clear()
        forcedDimension = null
        generationComplete.set(false)
    }

    /**
     * Check if force loading has been initiated.
     */
    fun isForceLoadingActive(): Boolean = forcedChunks.isNotEmpty()

    /**
     * Get the number of force-loaded chunks.
     */
    fun getForcedChunkCount(): Int = forcedChunks.size
}
