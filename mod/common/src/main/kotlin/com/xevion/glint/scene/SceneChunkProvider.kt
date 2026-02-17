package com.xevion.glint.scene

import com.xevion.glint.Loggers
import io.netty.buffer.Unpooled
import net.minecraft.core.SectionPos
import net.minecraft.core.registries.Registries
import net.minecraft.nbt.CompoundTag
import net.minecraft.nbt.NbtIo
import net.minecraft.network.FriendlyByteBuf
import net.minecraft.network.protocol.game.ClientboundBundlePacket
import net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket
import net.minecraft.network.protocol.game.ClientboundLightUpdatePacket
import net.minecraft.resources.ResourceKey
import net.minecraft.resources.ResourceLocation
import net.minecraft.server.level.ServerLevel
import net.minecraft.server.level.ServerPlayer
import net.minecraft.world.level.ChunkPos
import net.minecraft.world.level.LightLayer
import net.minecraft.world.level.block.entity.BlockEntity
import net.minecraft.world.level.chunk.LevelChunk
import net.minecraft.world.level.chunk.status.ChunkStatus
import net.minecraft.world.level.chunk.storage.RegionFile
import net.minecraft.world.level.chunk.storage.RegionStorageInfo
import net.minecraft.world.level.chunk.storage.SerializableChunkData
import net.minecraft.world.level.levelgen.Heightmap
import net.minecraft.world.level.lighting.LevelLightEngine
import java.io.Closeable
import java.nio.file.Path
import java.util.BitSet
import java.util.concurrent.ConcurrentHashMap

private val log = Loggers.Scene.get()

/**
 * Serves chunk NBT data from scene package region files.
 * Thread-safe: ChunkStorage.read() may be called from IO threads.
 */
class SceneChunkProvider(
    private val meta: ScenePackageMeta,
    private val regionDir: Path,
) : Closeable {
    private val regionFiles = ConcurrentHashMap<Long, RegionFile>()
    private val missingRegions = ConcurrentHashMap.newKeySet<Long>()
    private val openLock = Any()
    private val bounds = meta.chunkBounds

    /** Check if a chunk position (in scene coordinates) is within the scene bounds. */
    fun isInBounds(
        chunkX: Int,
        chunkZ: Int,
    ): Boolean = bounds.contains(chunkX, chunkZ)

    fun getChunk(pos: ChunkPos): CompoundTag? {
        if (!bounds.contains(pos.x, pos.z)) return null

        val regionFile = getOrOpenRegionFile(pos) ?: return null
        val stream = regionFile.getChunkDataInputStream(pos) ?: return null
        return stream.use { NbtIo.read(it) }
    }

    private fun getOrOpenRegionFile(pos: ChunkPos): RegionFile? {
        val regionKey = ChunkPos.asLong(pos.regionX, pos.regionZ)

        // Fast path: already cached
        regionFiles[regionKey]?.let { return it }

        // Fast path: known missing
        if (regionKey in missingRegions) return null

        // Slow path: open or record miss
        return synchronized(openLock) {
            // Re-check after acquiring lock
            regionFiles[regionKey]?.let { return it }
            if (regionKey in missingRegions) return null

            val regionPath = regionDir.resolve("r.${pos.regionX}.${pos.regionZ}.mca")
            if (!regionPath.toFile().exists()) {
                missingRegions.add(regionKey)
                return null
            }

            val dimensionKey =
                ResourceKey.create(
                    Registries.DIMENSION,
                    ResourceLocation.parse(meta.dimension),
                )
            val info = RegionStorageInfo("glint-scene", dimensionKey, "chunk")
            val regionFile = RegionFile(info, regionPath, regionDir, false)
            regionFiles[regionKey] = regionFile
            regionFile
        }
    }

    override fun close() {
        regionFiles.values.forEach { regionFile ->
            runCatching { regionFile.close() }
                .onFailure { log.warn(it, "Failed to close region file") }
        }
        regionFiles.clear()
        missingRegions.clear()
    }
}

/**
 * Build a [ClientboundLightUpdatePacket] from scene section data, bypassing the
 * light engine entirely.
 *
 * [ThreadedLevelLightEngine][net.minecraft.server.level.ThreadedLevelLightEngine]
 * schedules all light engine calls as async tasks, and its internal `queuedSections`
 * map is iterated without per-operation synchronization on worker threads. Writing
 * to it from the main thread causes concurrent modification crashes.
 *
 * Instead, we serialize the scene's [DataLayer]s directly into the packet's wire
 * format and decode it via the packet's [StreamCodec].
 */
private fun buildSceneLightPacket(
    chunkPos: ChunkPos,
    lightEngine: LevelLightEngine,
    sectionData: List<SerializableChunkData.SectionData>,
): ClientboundLightUpdatePacket {
    val sectionMap = sectionData.associateBy { it.y() }

    val skyYMask = BitSet()
    val blockYMask = BitSet()
    val emptySkyYMask = BitSet()
    val emptyBlockYMask = BitSet()
    val skyUpdates = mutableListOf<ByteArray>()
    val blockUpdates = mutableListOf<ByteArray>()

    for (i in 0 until lightEngine.lightSectionCount) {
        val sectionY = lightEngine.minLightSection + i
        val section = sectionMap[sectionY]

        section?.skyLight()?.let { skyLight ->
            if (skyLight.isEmpty) {
                emptySkyYMask.set(i)
            } else {
                skyYMask.set(i)
                skyUpdates.add(skyLight.copy().getData())
            }
        }

        section?.blockLight()?.let { blockLight ->
            if (blockLight.isEmpty) {
                emptyBlockYMask.set(i)
            } else {
                blockYMask.set(i)
                blockUpdates.add(blockLight.copy().getData())
            }
        }
    }

    // Serialize in the ClientboundLightUpdatePacket wire format:
    // VarInt x, VarInt z, then ClientboundLightUpdatePacketData fields
    val buf = FriendlyByteBuf(Unpooled.buffer())
    try {
        buf.writeVarInt(chunkPos.x)
        buf.writeVarInt(chunkPos.z)
        buf.writeBitSet(skyYMask)
        buf.writeBitSet(blockYMask)
        buf.writeBitSet(emptySkyYMask)
        buf.writeBitSet(emptyBlockYMask)
        buf.writeVarInt(skyUpdates.size)
        for (data in skyUpdates) buf.writeByteArray(data)
        buf.writeVarInt(blockUpdates.size)
        for (data in blockUpdates) buf.writeByteArray(data)

        return ClientboundLightUpdatePacket.STREAM_CODEC.decode(buf)
    } finally {
        buf.release()
    }
}

/**
 * Patch a chunk's NBT to match a new world position after relocation.
 *
 * Chunk NBT contains absolute positions that Minecraft validates on load:
 * - xPos/zPos: chunk coordinates (validated by SerializableChunkData.read())
 * - block_entities: block entity x/z (used to place entities at correct block)
 * - block_ticks/fluid_ticks: scheduled tick x/z (filtered by chunk position on load)
 * - structures: removed entirely (complex format, not meaningful after relocation)
 *
 * Modifies the tag in-place.
 */
fun relocateChunkNbt(
    tag: CompoundTag,
    worldChunkX: Int,
    worldChunkZ: Int,
    blockOffsetX: Int,
    blockOffsetZ: Int,
) {
    tag.putInt("xPos", worldChunkX)
    tag.putInt("zPos", worldChunkZ)

    offsetPositionList(tag, "block_entities", blockOffsetX, blockOffsetZ)
    offsetPositionList(tag, "block_ticks", blockOffsetX, blockOffsetZ)
    offsetPositionList(tag, "fluid_ticks", blockOffsetX, blockOffsetZ)

    // Structure references are position-dependent and complex — just remove them
    tag.remove("structures")
}

/**
 * Offset x/z integer fields in every CompoundTag within a named ListTag.
 * Handles block_entities, block_ticks, and fluid_ticks which all use {x, y, z} format.
 */
private fun offsetPositionList(
    parent: CompoundTag,
    listName: String,
    blockOffsetX: Int,
    blockOffsetZ: Int,
) {
    if (!parent.contains(listName)) return
    val list = parent.getList(listName, 10) // 10 = CompoundTag type ID
    for (i in 0 until list.size) {
        val entry = list.getCompound(i)
        if (entry.contains("x")) entry.putInt("x", entry.getInt("x") + blockOffsetX)
        if (entry.contains("z")) entry.putInt("z", entry.getInt("z") + blockOffsetZ)
    }
}

/**
 * Replace a loaded chunk's data in-place with scene chunk data.
 *
 * Deserializes scene NBT via MC's own [SerializableChunkData.parse], then:
 * 1. Swaps section data (block states + biomes) into the live chunk
 * 2. Clears existing block entities and loads new ones from scene data
 * 3. Recalculates heightmaps from the new block data
 * 4. Queues light data and marks lighting for recalculation
 * 5. Sends a full chunk packet to all players who have this chunk loaded
 *
 * Must be called on the server main thread.
 *
 * @param level The server level containing the chunk
 * @param chunk The loaded LevelChunk to replace data in
 * @param sceneNbt The scene's chunk NBT (already relocated via [relocateChunkNbt])
 */
fun replaceLoadedChunk(
    level: ServerLevel,
    chunk: LevelChunk,
    sceneNbt: CompoundTag,
) {
    val parsed =
        SerializableChunkData.parse(level, level.registryAccess(), sceneNbt)
    if (parsed == null) {
        log.warn("Failed to parse scene chunk NBT, skipping") { "chunk" to chunk.pos.toString() }
        return
    }

    // 1. Replace sections
    val sections = chunk.sections
    for (sectionData in parsed.sectionData()) {
        val index = level.getSectionIndexFromSectionY(sectionData.y())
        if (index in sections.indices && sectionData.chunkSection() != null) {
            sections[index] = sectionData.chunkSection()
        }
    }

    // 2. Replace block entities
    chunk.clearAllBlockEntities()
    for (beTag in parsed.blockEntities()) {
        val pos = BlockEntity.getPosFromTag(beTag)
        val blockState = chunk.getBlockState(pos)
        val be = BlockEntity.loadStatic(pos, blockState, beTag, level.registryAccess())
        if (be != null) {
            chunk.addAndRegisterBlockEntity(be)
        }
    }

    // 3. Recalculate heightmaps
    Heightmap.primeHeightmaps(chunk, ChunkStatus.FINAL_HEIGHTMAPS)

    // 4. Handle lighting
    // ThreadedLevelLightEngine makes ALL light engine calls async (scheduled via
    // addTask). We can't write to queuedSections from the main thread because
    // markNewInconsistencies iterates it without synchronization on worker threads.
    // Instead: schedule async tasks for server-side state, and build the client
    // packet's light data directly from the parsed scene data.
    val lightEngine = level.lightEngine
    for (sectionData in parsed.sectionData()) {
        val sectionPos = SectionPos.of(chunk.pos, sectionData.y())
        if (sectionData.blockLight() != null) {
            lightEngine.queueSectionData(LightLayer.BLOCK, sectionPos, sectionData.blockLight())
        }
        if (sectionData.skyLight() != null) {
            lightEngine.queueSectionData(LightLayer.SKY, sectionPos, sectionData.skyLight())
        }
    }
    for (i in 0 until chunk.sectionsCount) {
        val section = chunk.sections[i]
        if (!section.hasOnlyAir()) {
            val sectionY = level.getSectionYFromSectionIndex(i)
            lightEngine.updateSectionStatus(SectionPos.of(chunk.pos, sectionY), false)
        }
    }
    lightEngine.setLightEnabled(chunk.pos, true)
    lightEngine.retainData(chunk.pos, true)
    chunk.initializeLightSources()
    chunk.setLightCorrect(true)

    // 5. Send chunk data + correct light to all players as an atomic bundle.
    // The chunk packet uses empty light masks (the async tasks haven't processed yet,
    // so the light engine has stale data). A separate light update packet built
    // directly from the scene's saved DataLayers provides the correct lighting.
    // Bundling both packets ensures the client processes them in the same tick,
    // avoiding a one-frame dark flash between chunk arrival and light correction.
    val noSections = BitSet()
    val chunkPacket = ClientboundLevelChunkWithLightPacket(chunk, lightEngine, noSections, noSections)
    val lightPacket = buildSceneLightPacket(chunk.pos, lightEngine, parsed.sectionData())
    val bundle = ClientboundBundlePacket(listOf(chunkPacket, lightPacket))
    for (player in level.players()) {
        if (player is ServerPlayer) {
            player.connection.send(bundle)
        }
    }
}
