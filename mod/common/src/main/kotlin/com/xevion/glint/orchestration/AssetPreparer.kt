package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.GlintJson
import com.xevion.glint.api.WorkItem
import com.xevion.glint.scene.SceneManager
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonObject
import net.minecraft.client.Minecraft
import java.io.File
import java.io.IOException
import java.security.MessageDigest
import java.util.concurrent.CompletionException

data class ShaderGroup(
    val shaderVersionId: String,
    val shaderId: String,
    val shaderSlug: String,
    val shaderName: String,
    val version: String,
    val downloadUrl: String?,
    val fileHash: String?,
    val items: List<WorkItem>,
)

sealed class PrepResult {
    data class Ready(
        val spec: CaptureSpec,
    ) : PrepResult()

    data class Failed(
        val reason: String,
        val isShaderFailure: Boolean,
    ) : PrepResult()
}

class AssetPreparer(
    private val gameDirectory: File,
) {
    private val log = Loggers.Orchestration.get()
    private val json = GlintJson

    fun prepare(
        group: ShaderGroup,
        runId: String?,
    ): PrepResult {
        // 1. Download worlds
        val worldFolders = downloadWorlds(group)
        if (worldFolders.isEmpty()) {
            return PrepResult.Failed("Failed to download any worlds", isShaderFailure = false)
        }

        // 2. Download shader (if not vanilla)
        val shaderFilename =
            if (group.shaderSlug != "vanilla") {
                val filename = downloadShader(group)
                if (filename == null) {
                    return PrepResult.Failed("Failed to download shader: ${group.shaderName}", isShaderFailure = true)
                }
                filename
            } else {
                null
            }

        // 3. Write scene definitions
        writeSceneDefinitions(group)

        // 4. Build capture spec
        val spec =
            buildCaptureSpec(group, shaderFilename, runId)
                ?: return PrepResult.Failed("No valid scenes found", isShaderFailure = false)

        return PrepResult.Ready(spec)
    }

    private fun downloadWorlds(group: ShaderGroup): Map<String, String> {
        val savesDir = File(gameDirectory, "saves")
        val worldFolders = mutableMapOf<String, String>()

        val uniqueWorlds =
            group.items
                .distinctBy { it.worldId }
                .map { item ->
                    WorldTarget(
                        id = item.worldId,
                        slug = item.worldSlug,
                        name = item.worldName,
                        fileUrl = item.worldFileUrl,
                        fileHash = item.worldFileHash,
                    )
                }

        for (world in uniqueWorlds) {
            val existingDir = File(savesDir, world.slug)
            if (existingDir.exists() && existingDir.isDirectory) {
                log.debug("World already present") { "slug" to world.slug }
                worldFolders[world.id] = world.slug
                continue
            }

            val fileUrl = world.fileUrl
            if (fileUrl == null) {
                log.error("No download URL for world") { "name" to world.name }
                continue
            }

            log.info("Downloading world") {
                "name" to world.name
                "slug" to world.slug
            }
            try {
                var lastProgressLog = 0L
                val folderPath =
                    com.xevion.glint.download.WorldDownloader
                        .downloadWorld(
                            worldSlug = world.slug,
                            worldId = world.id,
                            fileUrl = fileUrl,
                            expectedHash = world.fileHash,
                            progressCallback = { progress ->
                                val now = System.currentTimeMillis()
                                val isTerminal = progress.state != com.xevion.glint.download.DownloadProgress.State.DOWNLOADING
                                if (isTerminal || now - lastProgressLog >= 1000) {
                                    lastProgressLog = now
                                    log.debug("World download progress") {
                                        "slug" to world.slug
                                        "progress" to progress
                                    }
                                }
                            },
                        ).join()

                val downloadedDir = File(gameDirectory, folderPath)
                val levelDat = File(downloadedDir, "level.dat")

                if (levelDat.exists()) {
                    val targetDir = File(savesDir, world.slug)
                    downloadedDir.copyRecursively(targetDir, overwrite = true)
                    worldFolders[world.id] = world.slug
                    log.info("World installed") { "slug" to world.slug }
                } else {
                    val subfolders = downloadedDir.listFiles()?.filter { it.isDirectory } ?: emptyList()
                    val worldSubfolder = subfolders.find { File(it, "level.dat").exists() }
                    if (worldSubfolder != null) {
                        val targetDir = File(savesDir, world.slug)
                        worldSubfolder.copyRecursively(targetDir, overwrite = true)
                        worldFolders[world.id] = world.slug
                        log.info("World installed") {
                            "slug" to world.slug
                            "subfolder" to worldSubfolder.name
                        }
                    } else {
                        log.error("Downloaded world has no level.dat") { "path" to downloadedDir.absolutePath }
                    }
                }
            } catch (e: CompletionException) {
                log.error(e.cause ?: e, "Failed to download world") { "slug" to world.slug }
            } catch (e: IOException) {
                log.error(e, "Failed to download world") { "slug" to world.slug }
            } catch (e: SecurityException) {
                log.error(e, "Failed to download world") { "slug" to world.slug }
            }
        }

        return worldFolders
    }

    private fun downloadShader(group: ShaderGroup): String? {
        val shaderpacksDir = File(gameDirectory, "shaderpacks")
        shaderpacksDir.mkdirs()

        val hash8 = group.fileHash?.take(8)
        val filename =
            if (hash8 != null) {
                "${group.shaderSlug}-${group.version}-$hash8.zip"
            } else {
                "${group.shaderSlug}-${group.version}.zip"
            }
        val targetFile = File(shaderpacksDir, filename)

        // Check if file already exists
        if (targetFile.exists()) {
            if (group.fileHash != null) {
                val existingHash = sha1Hex(targetFile)
                if (existingHash == group.fileHash) {
                    log.debug("Shader already present and verified") { "file" to filename }
                    return filename
                }
                log.warn("Shader file exists but hash mismatch, re-downloading") {
                    "file" to filename
                    "expected" to group.fileHash
                    "actual" to existingHash
                }
                targetFile.delete()
            } else {
                log.debug("Shader already present") { "file" to filename }
                return filename
            }
        }

        val downloadUrl = group.downloadUrl
        if (downloadUrl == null) {
            log.error("No download URL for shader") { "name" to group.shaderName }
            return null
        }

        log.info("Downloading shader") {
            "name" to group.shaderName
            "file" to filename
        }
        try {
            val connection =
                java.net
                    .URI(downloadUrl)
                    .toURL()
                    .openConnection() as java.net.HttpURLConnection
            connection.requestMethod = "GET"
            connection.connectTimeout = 10000
            connection.readTimeout = 120000

            if (connection.responseCode !in 200..299) {
                log.error("Shader download failed") { "status" to connection.responseCode }
                return null
            }

            connection.inputStream.use { input ->
                targetFile.outputStream().use { output ->
                    input.copyTo(output)
                }
            }

            // Verify downloaded file hash
            if (group.fileHash != null) {
                val downloadedHash = sha1Hex(targetFile)
                if (downloadedHash != group.fileHash) {
                    log.error("Downloaded shader hash mismatch") {
                        "file" to filename
                        "expected" to group.fileHash
                        "actual" to downloadedHash
                    }
                    targetFile.delete()
                    return null
                }
            }

            log.info("Shader downloaded") {
                "file" to filename
                "bytes" to targetFile.length()
            }
            return filename
        } catch (e: IOException) {
            log.error(e, "Failed to download shader")
            targetFile.delete()
            return null
        } catch (e: SecurityException) {
            log.error(e, "Failed to download shader")
            targetFile.delete()
            return null
        }
    }

    private fun sha1Hex(file: File): String {
        val digest = MessageDigest.getInstance("SHA-1")
        file.inputStream().use { input ->
            val buffer = ByteArray(8192)
            var bytesRead: Int
            while (input.read(buffer).also { bytesRead = it } != -1) {
                digest.update(buffer, 0, bytesRead)
            }
        }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }

    private fun writeSceneDefinitions(group: ShaderGroup) {
        val scenesDir = File(gameDirectory, "glint/scenes")
        scenesDir.mkdirs()

        val itemsByWorld = group.items.groupBy { it.worldSlug }

        for ((worldSlug, items) in itemsByWorld) {
            val collectionFile = File(scenesDir, "$worldSlug.json")

            val sceneElements =
                items
                    .distinctBy { it.sceneId }
                    .map { item ->
                        buildJsonObject {
                            put("id", item.sceneId)
                            put("name", item.sceneName)
                            putJsonObject("position") {
                                put("x", item.sceneX)
                                put("y", item.sceneY)
                                put("z", item.sceneZ)
                            }
                            putJsonObject("camera") {
                                put("yaw", item.sceneYaw)
                                put("pitch", item.scenePitch)
                            }
                            put("timeOfDay", item.sceneTimeOfDayTicks)
                            put("dimension", item.sceneDimension)
                            put("weather", item.sceneWeather)
                            put("weatherIntensity", item.sceneWeatherIntensity)
                            if (item.sceneBiome != null) put("biome", item.sceneBiome)
                            if (item.sceneMoonPhase != null) put("moonPhase", item.sceneMoonPhase)
                        }
                    }

            val mcVersion = Minecraft.getInstance().launchedVersion
            val collection =
                buildJsonObject {
                    put("world", worldSlug)
                    put("folder", worldSlug)
                    put("version", mcVersion)
                    put("scenes", JsonArray(sceneElements))
                }

            collectionFile.writeText(
                json.encodeToString(
                    kotlinx.serialization.json.JsonElement
                        .serializer(),
                    collection,
                ),
            )
            log.info("Wrote scene collection") {
                "file" to collectionFile.name
                "scene_count" to sceneElements.size
            }
        }

        SceneManager.clearCache()
    }

    private fun buildCaptureSpec(
        group: ShaderGroup,
        shaderFilename: String?,
        runId: String?,
    ): CaptureSpec? {
        val allSceneIds = group.items.map { it.sceneId }.distinct()
        if (allSceneIds.isEmpty()) return null

        // Collect distinct (profileId, profileName) pairs for Iris application and API lookups
        val profilePairs =
            group.items
                .filter { it.profileId != null && it.profileName != null }
                .map { it.profileId!! to it.profileName!! }
                .distinct()
        val shaders =
            buildList {
                if (shaderFilename == null) {
                    add(ShaderSpec(filename = null))
                } else if (profilePairs.isEmpty()) {
                    add(ShaderSpec(filename = shaderFilename))
                } else {
                    for ((profileId, profileName) in profilePairs) {
                        add(ShaderSpec(filename = shaderFilename, profile = profileName, profileId = profileId))
                    }
                }
            }

        return CaptureSpec(
            sceneIds = allSceneIds,
            shaders = shaders,
            outputDir = "glint/runs/$runId",
            shutdownOnComplete = false,
            runId = runId,
        )
    }

    private data class WorldTarget(
        val id: String,
        val slug: String,
        val name: String,
        val fileUrl: String?,
        val fileHash: String?,
    )
}
