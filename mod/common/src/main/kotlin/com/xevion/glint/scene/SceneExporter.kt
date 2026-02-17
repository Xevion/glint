package com.xevion.glint.scene

import com.xevion.glint.Loggers
import kotlinx.serialization.encodeToString
import net.minecraft.SharedConstants
import net.minecraft.client.Minecraft
import net.minecraft.nbt.NbtIo
import net.minecraft.world.level.ChunkPos
import net.minecraft.world.level.chunk.storage.RegionFile
import net.minecraft.world.level.chunk.storage.RegionStorageInfo
import java.io.DataOutputStream
import java.nio.file.Files
import java.nio.file.Path
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream

private val log = Loggers.Scene.get()

/**
 * Exports the current player state as a scene package ZIP.
 * Captures region files, entity data, and metadata for later injection.
 */
object SceneExporter {
    /**
     * Export current player state as a scene package.
     * Must be called from the game thread with an active integrated server.
     *
     * **Blocking:** Calls [CompletableFuture.join] to read chunk NBT from the
     * chunk storage I/O thread (see [extractRegions]). Acceptable for the
     * interactive `/glint export` command but must not be used on the autonomous
     * capture path — autonomous export should use an async alternative.
     */
    fun export(name: String): Result<Path> =
        runCatching {
            val mc = Minecraft.getInstance()
            val player = mc.player ?: error("No player")
            val server = mc.singleplayerServer ?: error("No integrated server")
            val serverLevel =
                server.getLevel(player.level().dimension())
                    ?: error("No server level for ${player.level().dimension()}")

            val renderDistance = mc.options.renderDistance().get()
            val fov = mc.options.fov().get()

            val camera =
                CameraPosition(
                    x = player.x,
                    y = player.y,
                    z = player.z,
                    yaw = player.yRot,
                    pitch = player.xRot,
                )

            val cameraChunkX = player.blockPosition().x shr 4
            val cameraChunkZ = player.blockPosition().z shr 4
            val bounds =
                ChunkBounds(
                    min = listOf(cameraChunkX - renderDistance, cameraChunkZ - renderDistance),
                    max = listOf(cameraChunkX + renderDistance, cameraChunkZ + renderDistance),
                )

            val environment =
                PackageEnvironment(
                    time = serverLevel.dayTime(),
                    weather =
                        when {
                            serverLevel.isThundering -> "thunder"
                            serverLevel.isRaining -> "rain"
                            else -> "clear"
                        },
                    weatherIntensity = serverLevel.getRainLevel(1.0f),
                    moonPhase = serverLevel.getMoonPhase(),
                )

            val dimension =
                player
                    .level()
                    .dimension()
                    .location()
                    .toString()

            val meta =
                ScenePackageMeta(
                    minecraftVersion = SharedConstants.VERSION_STRING,
                    dimension = dimension,
                    camera = camera,
                    fov = fov,
                    renderDistance = renderDistance,
                    environment = environment,
                    chunkBounds = bounds,
                )

            val tempDir = Files.createTempDirectory("glint-export-$name")
            try {
                val regionOutDir = tempDir.resolve("region")
                Files.createDirectories(regionOutDir)

                extractRegions(serverLevel, bounds, regionOutDir)

                val entityTags = collectEntities(serverLevel, bounds)
                writeEntityNbt(tempDir.resolve("entities.nbt"), entityTags)

                val metaJson = scenePackageJson.encodeToString(meta)
                Files.writeString(tempDir.resolve("meta.json"), metaJson)

                val outputDir =
                    mc.gameDirectory
                        .toPath()
                        .resolve("glint")
                        .resolve("exports")
                Files.createDirectories(outputDir)
                val zipPath = outputDir.resolve("$name.zip")
                zipDirectory(tempDir, zipPath)

                log.info("Exported scene package") {
                    "name" to name
                    "path" to zipPath.toString()
                    "chunks" to ((bounds.maxChunkX - bounds.minChunkX + 1) * (bounds.maxChunkZ - bounds.minChunkZ + 1))
                    "entities" to entityTags.size
                }

                zipPath
            } finally {
                tempDir.toFile().deleteRecursively()
            }
        }

    private fun extractRegions(
        level: net.minecraft.server.level.ServerLevel,
        bounds: ChunkBounds,
        outputDir: Path,
    ) {
        val minRegionX = bounds.minChunkX shr 5
        val minRegionZ = bounds.minChunkZ shr 5
        val maxRegionX = bounds.maxChunkX shr 5
        val maxRegionZ = bounds.maxChunkZ shr 5

        for (regionX in minRegionX..maxRegionX) {
            for (regionZ in minRegionZ..maxRegionZ) {
                val regionPath = outputDir.resolve("r.$regionX.$regionZ.mca")
                val info =
                    RegionStorageInfo(
                        "glint-export",
                        level.dimension(),
                        "chunk",
                    )
                RegionFile(info, regionPath, outputDir, true).use { outRegion ->
                    val chunkMinX = maxOf(regionX shl 5, bounds.minChunkX)
                    val chunkMaxX = minOf((regionX shl 5) + 31, bounds.maxChunkX)
                    val chunkMinZ = maxOf(regionZ shl 5, bounds.minChunkZ)
                    val chunkMaxZ = minOf((regionZ shl 5) + 31, bounds.maxChunkZ)

                    var written = 0
                    for (cx in chunkMinX..chunkMaxX) {
                        for (cz in chunkMinZ..chunkMaxZ) {
                            val pos = ChunkPos(cx, cz)
                            val nbt =
                                level.chunkSource.chunkMap
                                    .read(pos)
                                    .join()
                                    .orElse(null)
                                    ?: continue
                            val dos: DataOutputStream = outRegion.getChunkDataOutputStream(pos)
                            dos.use { NbtIo.write(nbt, it) }
                            written++
                        }
                    }

                    log.debug("Extracted region") {
                        "region" to "r.$regionX.$regionZ.mca"
                        "chunks" to written
                    }
                }
            }
        }
    }

    private fun zipDirectory(
        sourceDir: Path,
        zipPath: Path,
    ) {
        ZipOutputStream(Files.newOutputStream(zipPath).buffered()).use { zos ->
            Files.walk(sourceDir).use { paths ->
                paths.filter { Files.isRegularFile(it) }.forEach { file ->
                    val relativePath = sourceDir.relativize(file).toString()
                    zos.putNextEntry(ZipEntry(relativePath))
                    Files.copy(file, zos)
                    zos.closeEntry()
                }
            }
        }
    }
}
