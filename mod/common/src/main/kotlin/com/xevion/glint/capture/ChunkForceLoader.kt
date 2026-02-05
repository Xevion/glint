package com.xevion.glint.capture

import com.xevion.glint.Loggers
import com.xevion.glint.debug
import net.minecraft.client.Minecraft
import net.minecraft.world.level.ChunkPos

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
    private val forcedChunks = mutableSetOf<Long>()
    private var forcedDimension: net.minecraft.resources.ResourceKey<net.minecraft.world.level.Level>? = null

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

        // Execute on server thread
        server.execute {
            val serverLevel = server.getLevel(dimension) ?: return@execute

            forcedDimension = dimension
            var count = 0

            for (dx in -renderDistance..renderDistance) {
                for (dz in -renderDistance..renderDistance) {
                    // Use circular pattern matching chunk loading behavior
                    if (dx * dx + dz * dz > renderDistance * renderDistance) continue

                    val cx = chunkX + dx
                    val cz = chunkZ + dz
                    val chunkPos = ChunkPos.asLong(cx, cz)

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
