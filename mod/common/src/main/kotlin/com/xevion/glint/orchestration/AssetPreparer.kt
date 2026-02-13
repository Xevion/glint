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

/** Result of preparing all assets for a set of work items. */
sealed class PrepResult {
    data class Ready(
        val items: List<WorkItem>,
    ) : PrepResult()

    data class Failed(
        val reason: String,
        /** True if a shader download failed (report to backend). */
        val isShaderFailure: Boolean = false,
        /** Shader version ID if this is a shader-specific failure. */
        val failedShaderVersionId: String? = null,
    ) : PrepResult()
}

/**
 * Prepares assets (worlds, shaders, scene definitions) for autonomous capture.
 *
 * Downloads all unique worlds and shaders from the work item list, writes scene
 * definitions to disk, and reports which items are ready for capture.
 */
class AssetPreparer(
    private val gameDirectory: File,
) {
    private val log = Loggers.Orchestration.get()
    private val json = GlintJson

    /**
     * Prepares all assets needed for the given work items.
     *
     * Downloads worlds and shaders, writes scene definitions, and returns
     * the items that are ready for capture (items whose worlds/shaders
     * downloaded successfully).
     */
    fun prepareAll(items: List<WorkItem>): PrepResult {
        if (items.isEmpty()) return PrepResult.Failed("No work items")

        // 1. Download all unique worlds
        val worldFolders = downloadWorlds(items)
        if (worldFolders.isEmpty()) {
            return PrepResult.Failed("Failed to download any worlds")
        }

        // 2. Download all unique shaders
        val shaderResults = downloadShaders(items)

        // 3. Write scene definitions
        writeSceneDefinitions(items)

        // 4. Filter items to only those with successfully downloaded assets
        val readyItems =
            items.filter { item ->
                val worldReady = item.worldId in worldFolders
                val shaderReady =
                    if (item.shaderSlug == "vanilla") {
                        true
                    } else {
                        shaderResults[item.shaderVersionId] != null
                    }
                worldReady && shaderReady
            }

        if (readyItems.isEmpty()) {
            return PrepResult.Failed("No items ready after asset preparation")
        }

        log.info("Asset preparation complete") {
            "total_items" to items.size
            "ready_items" to readyItems.size
            "worlds" to worldFolders.size
            "shaders" to shaderResults.size
        }

        return PrepResult.Ready(readyItems)
    }

    /**
     * Downloads all unique worlds from the work items.
     * Returns a map of world ID → world folder name for successfully downloaded worlds.
     */
    private fun downloadWorlds(items: List<WorkItem>): Map<String, String> {
        val savesDir = File(gameDirectory, "saves")
        val worldFolders = mutableMapOf<String, String>()

        val uniqueWorlds =
            items
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

    /**
     * Downloads all unique shaders from the work items.
     * Returns a map of shader version ID → filename for successfully downloaded shaders.
     */
    private fun downloadShaders(items: List<WorkItem>): Map<String, String> {
        val results = mutableMapOf<String, String>()

        val uniqueShaders =
            items
                .filter { it.shaderSlug != "vanilla" }
                .distinctBy { it.shaderVersionId }

        for (item in uniqueShaders) {
            val filename = downloadShader(item)
            if (filename != null) {
                results[item.shaderVersionId] = filename
            } else {
                log.error("Failed to download shader") {
                    "shader" to item.shaderName
                    "version" to item.version
                }
            }
        }

        return results
    }

    /** Downloads a single shader pack and returns its filename, or null on failure. */
    private fun downloadShader(item: WorkItem): String? {
        val shaderpacksDir = File(gameDirectory, "shaderpacks")
        shaderpacksDir.mkdirs()

        val hash8 = item.fileHash?.take(8)
        val filename =
            if (hash8 != null) {
                "${item.shaderSlug}-${item.version}-$hash8.zip"
            } else {
                "${item.shaderSlug}-${item.version}.zip"
            }
        val targetFile = File(shaderpacksDir, filename)

        // Check if file already exists
        if (targetFile.exists()) {
            if (item.fileHash != null) {
                val existingHash = sha1Hex(targetFile)
                if (existingHash == item.fileHash) {
                    log.debug("Shader already present and verified") { "file" to filename }
                    return filename
                }
                log.warn("Shader file exists but hash mismatch, re-downloading") {
                    "file" to filename
                    "expected" to item.fileHash
                    "actual" to existingHash
                }
                targetFile.delete()
            } else {
                log.debug("Shader already present") { "file" to filename }
                return filename
            }
        }

        val downloadUrl = item.downloadUrl
        if (downloadUrl == null) {
            log.error("No download URL for shader") { "name" to item.shaderName }
            return null
        }

        log.info("Downloading shader") {
            "name" to item.shaderName
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
            if (item.fileHash != null) {
                val downloadedHash = sha1Hex(targetFile)
                if (downloadedHash != item.fileHash) {
                    log.error("Downloaded shader hash mismatch") {
                        "file" to filename
                        "expected" to item.fileHash
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

    /** Writes scene definition JSON files for all scenes referenced by the work items. */
    private fun writeSceneDefinitions(items: List<WorkItem>) {
        val scenesDir = File(gameDirectory, "glint/scenes")
        scenesDir.mkdirs()

        val itemsByWorld = items.groupBy { it.worldSlug }

        for ((worldSlug, worldItems) in itemsByWorld) {
            val collectionFile = File(scenesDir, "$worldSlug.json")

            val sceneElements =
                worldItems
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

    private data class WorldTarget(
        val id: String,
        val slug: String,
        val name: String,
        val fileUrl: String?,
        val fileHash: String?,
    )
}
