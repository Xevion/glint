package com.xevion.glint.scene

import com.xevion.glint.Loggers
import net.minecraft.nbt.CompoundTag
import net.minecraft.nbt.ListTag
import net.minecraft.nbt.NbtIo
import net.minecraft.server.level.ServerLevel
import net.minecraft.server.level.ServerPlayer
import net.minecraft.world.entity.EntitySpawnReason
import net.minecraft.world.entity.EntityType
import java.io.DataInputStream
import java.io.DataOutputStream
import java.nio.file.Path
import java.util.UUID

private val log = Loggers.Scene.get()

/**
 * Discard all non-player entities in the level.
 * Returns the count of discarded entities.
 */
fun clearEntities(level: ServerLevel): Int {
    // Snapshot to avoid ConcurrentModificationException — discard() mutates the live collection
    val toRemove = level.allEntities.filter { it !is ServerPlayer }.toList()
    toRemove.forEach { it.discard() }
    log.info("Cleared entities") { "count" to toRemove.size }
    return toRemove.size
}

/**
 * Spawn entities from scene package NBT data with optional position offset.
 * Each entry is a CompoundTag from Entity.save() — full round-trip.
 * Block offsets translate scene coordinates to world coordinates for relative injection.
 * Returns the count of successfully spawned entities.
 */
fun spawnEntities(
    level: ServerLevel,
    entityTags: List<CompoundTag>,
    blockOffsetX: Int = 0,
    blockOffsetZ: Int = 0,
): Int {
    var count = 0
    for (tag in entityTags) {
        val spawnTag = tag.copy()
        prepareEntityTag(spawnTag, blockOffsetX, blockOffsetZ)

        val entity =
            EntityType.loadEntityRecursive(spawnTag, level, EntitySpawnReason.LOAD) { e ->
                if (level.addFreshEntity(e)) {
                    count++
                    e
                } else {
                    null
                }
            }
        if (entity == null) {
            val type = tag.getString("id")
            log.warn("Failed to spawn entity") { "type" to type }
        }
    }
    log.info("Spawned entities") {
        "count" to count
        "total" to entityTags.size
    }
    return count
}

/**
 * Offset an entity's position in its NBT tag.
 * Handles the "Pos" double list tag (x, y, z) used by Entity.save().
 */
private fun offsetEntityPosition(
    tag: CompoundTag,
    blockOffsetX: Int,
    blockOffsetZ: Int,
) {
    val posList = tag.getList("Pos", 6) // 6 = DoubleTag type ID
    if (posList.size == 3) {
        val x = posList.getDouble(0) + blockOffsetX
        val z = posList.getDouble(2) + blockOffsetZ
        posList.set(
            0,
            net.minecraft.nbt.DoubleTag
                .valueOf(x),
        )
        posList.set(
            2,
            net.minecraft.nbt.DoubleTag
                .valueOf(z),
        )
    }
}

/**
 * Recursively prepare an entity tag for injection:
 * - Assign fresh UUID to prevent collisions on re-injection
 * - Offset position for relative injection
 * Walks the Passengers NBT tree so nested passengers are also fixed.
 */
private fun prepareEntityTag(
    tag: CompoundTag,
    blockOffsetX: Int,
    blockOffsetZ: Int,
) {
    tag.putUUID("UUID", UUID.randomUUID())
    if (blockOffsetX != 0 || blockOffsetZ != 0) {
        offsetEntityPosition(tag, blockOffsetX, blockOffsetZ)
    }

    // Recurse into passengers (tag type 9 = ListTag, type 10 = CompoundTag)
    if (tag.contains("Passengers", 9)) {
        val passengers = tag.getList("Passengers", 10)
        for (i in 0 until passengers.size) {
            prepareEntityTag(passengers.getCompound(i), blockOffsetX, blockOffsetZ)
        }
    }
}

/**
 * Read entities.nbt from a scene package.
 * Format: root compound with "entities" ListTag of CompoundTags.
 */
fun readEntityNbt(path: Path): List<CompoundTag> {
    if (!path.toFile().exists()) return emptyList()
    val root =
        DataInputStream(path.toFile().inputStream().buffered()).use { stream ->
            NbtIo.read(stream)
        }
    val list = root.getList("entities", 10) // 10 = CompoundTag type ID
    return (0 until list.size).map { list.getCompound(it) }
}

/**
 * Write entities.nbt for a scene package.
 * Format: root compound with "entities" ListTag of CompoundTags.
 */
fun writeEntityNbt(
    path: Path,
    tags: List<CompoundTag>,
) {
    val root = CompoundTag()
    val list = ListTag()
    tags.forEach { list.add(it) }
    root.put("entities", list)
    DataOutputStream(path.toFile().outputStream().buffered()).use { stream ->
        NbtIo.write(root, stream)
    }
}

/**
 * Collect all non-player entities within chunk bounds and save their NBT.
 * Returns the entity tags as CompoundTags.
 */
fun collectEntities(
    level: ServerLevel,
    bounds: ChunkBounds,
): List<CompoundTag> {
    val tags = mutableListOf<CompoundTag>()
    level.allEntities.forEach { entity ->
        if (entity is ServerPlayer) return@forEach
        val chunkX = entity.blockPosition().x shr 4
        val chunkZ = entity.blockPosition().z shr 4
        if (!bounds.contains(chunkX, chunkZ)) return@forEach

        val tag = CompoundTag()
        if (entity.save(tag)) {
            tags.add(tag)
        }
    }
    log.info("Collected entities") { "count" to tags.size }
    return tags
}
