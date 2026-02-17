package com.xevion.glint.scene

import com.xevion.glint.Loggers
import net.minecraft.nbt.CompoundTag
import net.minecraft.server.level.ServerLevel
import net.minecraft.server.level.ServerPlayer
import net.minecraft.server.level.TicketType
import net.minecraft.world.level.ChunkPos
import java.nio.file.Files
import java.nio.file.Path
import java.util.zip.ZipFile
import kotlin.math.floor

private val log = Loggers.Scene.get()

/**
 * A loaded scene package ready for injection.
 * Created from an extracted ZIP, holds region files open.
 */
class LoadedScene(
    val meta: ScenePackageMeta,
    val provider: SceneChunkProvider,
    val entityTags: List<CompoundTag>,
    private val extractDir: Path,
) : AutoCloseable {
    override fun close() {
        provider.close()
        extractDir.toFile().deleteRecursively()
    }
}

/**
 * Manages scene injection lifecycle: loading, activation, switching, cleanup.
 */
class SceneInjector {
    private var currentScene: LoadedScene? = null

    /** Chunk offset used by the current/last injection, for ticket cleanup */
    private var activeChunkOffsetX = 0
    private var activeChunkOffsetZ = 0

    /**
     * Load a scene package from a ZIP file.
     * Extracts to temp directory, opens region files, parses metadata.
     */
    fun load(zipPath: Path): LoadedScene {
        val extractDir = Files.createTempDirectory("glint-scene").normalize()
        try {
            ZipFile(zipPath.toFile()).use { zip ->
                zip.entries().asSequence().forEach { entry ->
                    val target = extractDir.resolve(entry.name).normalize()
                    require(target.startsWith(extractDir)) {
                        "ZipSlip: entry '${entry.name}' escapes extract directory"
                    }
                    if (entry.isDirectory) {
                        Files.createDirectories(target)
                    } else {
                        Files.createDirectories(target.parent)
                        zip.getInputStream(entry).use { input ->
                            Files.copy(input, target)
                        }
                    }
                }
            }

            val metaJson = Files.readString(extractDir.resolve("meta.json"))
            val meta = scenePackageJson.decodeFromString<ScenePackageMeta>(metaJson)
            val entityTags = readEntityNbt(extractDir.resolve("entities.nbt"))
            val provider = SceneChunkProvider(meta, extractDir.resolve("region"))

            val currentVersion =
                net.minecraft.SharedConstants
                    .getCurrentVersion()
                    .id
            if (meta.minecraftVersion != currentVersion) {
                log.warn("Scene was created for a different Minecraft version") {
                    "scene_version" to meta.minecraftVersion
                    "current_version" to currentVersion
                }
            }

            val bounds = meta.chunkBounds
            log.info("Loaded scene package") {
                "dimension" to meta.dimension
                "chunks" to
                    ((bounds.maxChunkX - bounds.minChunkX + 1) * (bounds.maxChunkZ - bounds.minChunkZ + 1))
                "entities" to entityTags.size
            }

            return LoadedScene(meta, provider, entityTags, extractDir)
        } catch (e: Exception) {
            extractDir.toFile().deleteRecursively()
            throw e
        }
    }

    /**
     * Inject a loaded scene into a server level with a chunk offset for relative positioning.
     * The offset translates scene coordinates to world coordinates:
     *   world chunk = scene chunk + offset
     *
     * Returns a tick-driven state machine — call tick() each server tick until complete.
     */
    fun inject(
        scene: LoadedScene,
        level: ServerLevel,
        chunkOffsetX: Int,
        chunkOffsetZ: Int,
        cameraTarget: CameraPosition,
    ): InjectionProcess {
        activeChunkOffsetX = chunkOffsetX
        activeChunkOffsetZ = chunkOffsetZ
        return InjectionProcess(scene, level, this, chunkOffsetX, chunkOffsetZ, cameraTarget)
    }

    /**
     * Deactivate any active scene, unload chunks, clean up.
     * Also accepts a LoadedScene directly for cases where injection was in progress
     * but the scene hadn't been set as "current" yet.
     */
    fun deactivate(
        level: ServerLevel,
        pendingScene: LoadedScene? = null,
    ) {
        val scene = currentScene ?: pendingScene
        val offsetX = activeChunkOffsetX
        val offsetZ = activeChunkOffsetZ

        // Always clean up global state regardless of whether we have a scene reference
        SceneInjection.unfreezeEntityTick()
        SceneInjection.deactivate()
        clearEntities(level)

        if (scene != null) {
            // Remove chunk tickets using offset world coordinates
            val bounds = scene.meta.chunkBounds
            for (cx in bounds.minChunkX..bounds.maxChunkX) {
                for (cz in bounds.minChunkZ..bounds.maxChunkZ) {
                    val worldPos = ChunkPos(cx + offsetX, cz + offsetZ)
                    level.chunkSource.removeRegionTicket(TicketType.FORCED, worldPos, 0, worldPos)
                }
            }
            scene.close()
        }

        // If pendingScene differs from the scene we cleaned up, close it too
        if (pendingScene != null && pendingScene !== scene) {
            pendingScene.close()
        }

        activeChunkOffsetX = 0
        activeChunkOffsetZ = 0
        currentScene = null
        log.info("Deactivated scene")
    }

    fun setCurrentScene(scene: LoadedScene) {
        currentScene = scene
    }

    fun getCurrentScene(): LoadedScene? = currentScene
}

/**
 * Tick-driven state machine for scene injection.
 * Handles the full lifecycle: activate mixin → replace loaded chunks in-place →
 * wait for remaining chunks to load via mixin → spawn entities → teleport to camera.
 *
 * Chunk offset translates scene coordinates to world coordinates:
 *   world chunk = scene chunk + (chunkOffsetX, chunkOffsetZ)
 *
 * Hybrid approach: chunks already in RAM are patched in-place via [replaceLoadedChunk],
 * while chunks not yet loaded are intercepted by [ChunkStorageMixin] during disk reads.
 */
class InjectionProcess(
    private val scene: LoadedScene,
    private val level: ServerLevel,
    private val injector: SceneInjector,
    private val chunkOffsetX: Int,
    private val chunkOffsetZ: Int,
    private val cameraTarget: CameraPosition,
) {
    enum class State {
        /** Activate the mixin provider, freeze entities, clear existing entities. */
        ACTIVATING,

        /** Replace data in already-loaded chunks and add tickets for all chunks. */
        REPLACING_CHUNKS,

        /** Wait for remaining (previously unloaded) chunks to load via mixin. */
        LOADING_REMAINING,

        /** Spawn entities and teleport player to the camera position. */
        SPAWNING_ENTITIES,
        COMPLETE,
        FAILED,
    }

    var state = State.ACTIVATING
        private set
    var error: String? = null
        private set

    private var ticksWaited = 0
    private val maxLoadTicks = 600 // 30 seconds
    private val blockOffsetX = chunkOffsetX * 16
    private val blockOffsetZ = chunkOffsetZ * 16

    // World-coordinate bounds for the injection area
    private val worldMinCX = scene.meta.chunkBounds.minChunkX + chunkOffsetX
    private val worldMaxCX = scene.meta.chunkBounds.maxChunkX + chunkOffsetX
    private val worldMinCZ = scene.meta.chunkBounds.minChunkZ + chunkOffsetZ
    private val worldMaxCZ = scene.meta.chunkBounds.maxChunkZ + chunkOffsetZ

    /**
     * Advance the injection process one step. Call once per server tick.
     * Returns true when the process is terminal (success or failure).
     */
    fun tick(): Boolean {
        when (state) {
            State.ACTIVATING -> {
                SceneInjection.freezeEntityTick()
                clearEntities(level)
                SceneInjection.activate(scene.provider, chunkOffsetX, chunkOffsetZ)

                state = State.REPLACING_CHUNKS
                log.info("Activated scene injection") {
                    "chunk_offset" to "$chunkOffsetX,$chunkOffsetZ"
                    "world_bounds" to "$worldMinCX,$worldMinCZ -> $worldMaxCX,$worldMaxCZ"
                }
            }

            State.REPLACING_CHUNKS -> {
                val bounds = scene.meta.chunkBounds
                var replaced = 0
                var ticketed = 0

                // Camera chunk in world coordinates for center-outward ordering
                val cameraCX = floor(cameraTarget.x).toInt() shr 4
                val cameraCZ = floor(cameraTarget.z).toInt() shr 4

                val sortedChunks =
                    buildList {
                        for (cx in bounds.minChunkX..bounds.maxChunkX) {
                            for (cz in bounds.minChunkZ..bounds.maxChunkZ) {
                                add(cx to cz)
                            }
                        }
                    }.sortedBy { (cx, cz) ->
                        val worldCX = cx + chunkOffsetX
                        val worldCZ = cz + chunkOffsetZ
                        val dx = worldCX - cameraCX
                        val dz = worldCZ - cameraCZ
                        dx * dx + dz * dz
                    }

                for ((cx, cz) in sortedChunks) {
                    val worldCX = cx + chunkOffsetX
                    val worldCZ = cz + chunkOffsetZ
                    val worldPos = ChunkPos(worldCX, worldCZ)

                    // Add FORCED ticket regardless (drives loading for unloaded chunks,
                    // keeps loaded chunks loaded)
                    level.chunkSource.addRegionTicket(
                        TicketType.FORCED,
                        worldPos,
                        0,
                        worldPos,
                    )
                    ticketed++

                    // If chunk is already in memory, replace its data in-place
                    val existingChunk = level.chunkSource.getChunkNow(worldCX, worldCZ)
                    if (existingChunk != null) {
                        val sceneChunkPos = ChunkPos(cx, cz)
                        val sceneNbt = scene.provider.getChunk(sceneChunkPos)
                        if (sceneNbt != null) {
                            relocateChunkNbt(
                                sceneNbt,
                                worldCX,
                                worldCZ,
                                blockOffsetX,
                                blockOffsetZ,
                            )
                            replaceLoadedChunk(level, existingChunk, sceneNbt)
                            replaced++
                        }
                    }
                }

                ticksWaited = 0
                state = State.LOADING_REMAINING
                log.info("Replaced loaded chunks, waiting for remaining") {
                    "replaced" to replaced
                    "ticketed" to ticketed
                }
            }

            State.LOADING_REMAINING -> {
                ticksWaited++
                if (ticksWaited > maxLoadTicks) {
                    state = State.FAILED
                    error = "Timed out waiting for chunks to load after ${maxLoadTicks / 20}s"
                    log.error("Chunk loading timed out") { "ticks_waited" to ticksWaited }
                    return true
                }

                // Poll: are all chunks at world coordinates loaded?
                var allLoaded = true
                for (cx in worldMinCX..worldMaxCX) {
                    for (cz in worldMinCZ..worldMaxCZ) {
                        if (level.chunkSource.getChunkNow(cx, cz) == null) {
                            allLoaded = false
                            break
                        }
                    }
                    if (!allLoaded) break
                }

                if (allLoaded) {
                    log.info("All chunks loaded") { "ticks_waited" to ticksWaited }
                    state = State.SPAWNING_ENTITIES
                }
            }

            State.SPAWNING_ENTITIES -> {
                spawnEntities(level, scene.entityTags, blockOffsetX, blockOffsetZ)
                injector.setCurrentScene(scene)

                // Teleport player to the offset camera position
                val player = level.players().filterIsInstance<ServerPlayer>().firstOrNull()
                if (player != null) {
                    player.teleportTo(
                        level,
                        cameraTarget.x,
                        cameraTarget.y,
                        cameraTarget.z,
                        setOf(),
                        cameraTarget.yaw,
                        cameraTarget.pitch,
                        false,
                    )
                    log.info("Teleported player to scene camera") {
                        "pos" to "${cameraTarget.x}, ${cameraTarget.y}, ${cameraTarget.z}"
                    }
                }

                state = State.COMPLETE
                log.info("Scene injection complete")
            }

            State.COMPLETE, State.FAILED -> {
                return true
            }
        }

        return state == State.COMPLETE || state == State.FAILED
    }

    val isComplete: Boolean get() = state == State.COMPLETE
    val isFailed: Boolean get() = state == State.FAILED
}
