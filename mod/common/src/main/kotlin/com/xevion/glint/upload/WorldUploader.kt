package com.xevion.glint.upload

import com.xevion.glint.Loggers
import com.xevion.glint.api.CompleteWorldUploadRequest
import com.xevion.glint.api.CompleteWorldVersionUploadRequest
import com.xevion.glint.api.CreateWorldUploadRequest
import com.xevion.glint.api.CreateWorldVersionUploadRequest
import com.xevion.glint.api.GlintApi
import com.xevion.glint.api.WorldInfo
import net.minecraft.client.Minecraft
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.io.IOException
import java.net.HttpURLConnection
import java.net.URI
import java.security.MessageDigest
import java.util.concurrent.CompletableFuture
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream

/**
 * Handles packaging and uploading world files to the Glint backend.
 * Supports uploading the currently loaded world (force-save first)
 * or any world directory from disk.
 */
object WorldUploader {
    private val log = Loggers.Download.get()

    private const val MAX_UPLOAD_BYTES = 512L * 1024 * 1024
    private val EXCLUDED_FILES = setOf("session.lock")

    /**
     * Uploads a world directory to the backend.
     *
     * @param worldDir The world save directory on disk
     * @param name Display name for the world
     * @param slug URL-friendly slug
     * @param description Optional description
     * @param minecraftVersion Minecraft version string
     * @param apiUrl Backend API URL
     * @param token Auth token
     * @param forceSave If true, force-save the currently loaded world before packaging
     * @param progressCallback Called with upload progress updates
     * @return CompletableFuture with the created WorldInfo on success
     */
    fun uploadWorld(
        worldDir: File,
        name: String,
        slug: String,
        description: String?,
        minecraftVersion: String,
        apiUrl: String,
        token: String,
        forceSave: Boolean,
        progressCallback: (UploadProgress) -> Unit,
    ): CompletableFuture<WorldInfo> =
        CompletableFuture.supplyAsync {
            var zipFile: File? = null
            try {
                // Step 1: Force-save if uploading the currently loaded world
                if (forceSave) {
                    progressCallback(UploadProgress.saving())
                    forceSaveWorld()
                }

                // Step 2: Package world into ZIP
                progressCallback(UploadProgress.packaging(0, 0))
                val totalSize = calculateDirectorySize(worldDir)
                zipFile = File.createTempFile("glint-upload-", ".zip")

                log.info("Packaging world") {
                    "path" to worldDir.absolutePath
                    "totalSize" to totalSize
                }

                val hash = packageWorld(worldDir, zipFile, totalSize, progressCallback)
                val zipSize = zipFile.length()

                log.info("World packaged") {
                    "zipSize" to zipSize
                    "hash" to hash
                }

                if (zipSize > MAX_UPLOAD_BYTES) {
                    throw UploadException.FileTooLarge(zipSize, MAX_UPLOAD_BYTES)
                }

                // Step 3: Initiate upload (get presigned URL)
                progressCallback(UploadProgress.finalizing())
                val fileHash = "sha256:$hash"

                val createResult =
                    GlintApi.createWorldUpload(
                        apiUrl = apiUrl,
                        request =
                            CreateWorldUploadRequest(
                                name = name,
                                slug = slug,
                                description = description,
                                minecraftVersion = minecraftVersion,
                                fileHash = fileHash,
                                fileSizeBytes = zipSize,
                            ),
                        token = token,
                    )

                val uploadResponse =
                    createResult.getOrElse { error ->
                        throw UploadException.FinalizationFailed(error.message ?: "Failed to initiate upload")
                    }

                log.info("Upload initiated") {
                    "uploadId" to uploadResponse.uploadId
                }

                // Step 4: Upload ZIP to presigned URL
                uploadToPresignedUrl(
                    zipFile = zipFile,
                    presignedUrl = uploadResponse.presignedUrl,
                    hash = hash,
                    progressCallback = progressCallback,
                )

                // Step 5: Complete upload
                progressCallback(UploadProgress.finalizing())
                val completeResult =
                    GlintApi.completeWorldUpload(
                        apiUrl = apiUrl,
                        worldSlug = slug,
                        request = CompleteWorldUploadRequest(uploadId = uploadResponse.uploadId),
                        token = token,
                    )

                val worldInfo =
                    completeResult.getOrElse { error ->
                        throw UploadException.FinalizationFailed(error.message ?: "Failed to complete upload")
                    }

                progressCallback(UploadProgress.complete())
                log.info("World uploaded") {
                    "worldId" to worldInfo.id
                    "slug" to worldInfo.slug
                }

                worldInfo
            } catch (e: UploadException) {
                log.error(e, "World upload failed")
                progressCallback(UploadProgress.failed(e.message ?: "Unknown error"))
                throw e
            } catch (e: Exception) {
                log.error(e, "Unexpected error uploading world")
                val userMessage = "Unexpected error: ${e.message}"
                progressCallback(UploadProgress.failed(userMessage))
                throw UploadException.UploadInterrupted(userMessage, e)
            } finally {
                zipFile?.delete()
            }
        }

    /**
     * Uploads the current world as a new version of an existing world.
     * Uses the same two-phase upload flow as world creation:
     * initiate → upload to staging → complete (verify + finalize).
     *
     * @param worldDir The world save directory on disk
     * @param worldId Backend world ID to create a version for
     * @param apiUrl Backend API URL
     * @param token Auth token
     * @param forceSave If true, force-save the currently loaded world before packaging
     * @param progressCallback Called with upload progress updates
     * @return CompletableFuture with the upload ID on success
     */
    fun uploadWorldVersion(
        worldDir: File,
        worldId: String,
        apiUrl: String,
        token: String,
        forceSave: Boolean,
        progressCallback: (UploadProgress) -> Unit,
    ): CompletableFuture<String> =
        CompletableFuture.supplyAsync {
            var zipFile: File? = null
            try {
                // Step 1: Force-save if uploading the currently loaded world
                if (forceSave) {
                    progressCallback(UploadProgress.saving())
                    forceSaveWorld()
                }

                // Step 2: Package world into ZIP
                progressCallback(UploadProgress.packaging(0, 0))
                val totalSize = calculateDirectorySize(worldDir)
                zipFile = File.createTempFile("glint-upload-", ".zip")

                log.info("Packaging world for version upload") {
                    "path" to worldDir.absolutePath
                    "worldId" to worldId
                    "totalSize" to totalSize
                }

                val hash = packageWorld(worldDir, zipFile, totalSize, progressCallback)
                val zipSize = zipFile.length()

                log.info("World packaged for version upload") {
                    "zipSize" to zipSize
                    "hash" to hash
                }

                if (zipSize > MAX_UPLOAD_BYTES) {
                    throw UploadException.FileTooLarge(zipSize, MAX_UPLOAD_BYTES)
                }

                // Step 3: Create version upload (get presigned URL)
                progressCallback(UploadProgress.finalizing())
                val fileHash = "sha256:$hash"

                val createResult =
                    GlintApi.createWorldVersionUpload(
                        apiUrl = apiUrl,
                        worldId = worldId,
                        request =
                            CreateWorldVersionUploadRequest(
                                fileHash = fileHash,
                                fileSizeBytes = zipSize,
                            ),
                        token = token,
                    )

                val response =
                    createResult.getOrElse { error ->
                        throw UploadException.FinalizationFailed(
                            error.message ?: "Failed to create version upload",
                        )
                    }

                log.info("Version upload initiated") {
                    "uploadId" to response.uploadId
                    "worldId" to worldId
                }

                // Step 4: Upload ZIP to presigned URL
                uploadToPresignedUrl(
                    zipFile = zipFile,
                    presignedUrl = response.presignedUrl,
                    hash = hash,
                    progressCallback = progressCallback,
                )

                // Step 5: Complete upload (verify + create version record)
                progressCallback(UploadProgress.finalizing())
                val completeResult =
                    GlintApi.completeWorldVersionUpload(
                        apiUrl = apiUrl,
                        worldId = worldId,
                        request = CompleteWorldVersionUploadRequest(uploadId = response.uploadId),
                        token = token,
                    )

                completeResult.getOrElse { error ->
                    throw UploadException.FinalizationFailed(
                        error.message ?: "Failed to complete version upload",
                    )
                }

                progressCallback(UploadProgress.complete())
                log.info("World version uploaded") {
                    "uploadId" to response.uploadId
                    "worldId" to worldId
                }

                response.uploadId
            } catch (e: UploadException) {
                log.error(e, "World version upload failed")
                progressCallback(UploadProgress.failed(e.message ?: "Unknown error"))
                throw e
            } catch (e: Exception) {
                log.error(e, "Unexpected error uploading world version")
                val userMessage = "Unexpected error: ${e.message}"
                progressCallback(UploadProgress.failed(userMessage))
                throw UploadException.UploadInterrupted(userMessage, e)
            } finally {
                zipFile?.delete()
            }
        }

    /**
     * Force-saves the currently loaded singleplayer world.
     * Must be called from a thread that can submit to the server thread.
     */
    private fun forceSaveWorld() {
        val mc = Minecraft.getInstance()
        val server =
            mc.singleplayerServer
                ?: throw UploadException.SaveFailed(IllegalStateException("Not in singleplayer"))

        try {
            // saveEverything(suppressLog, flush, forced)
            server.saveEverything(false, true, true)
            log.info("Force-saved world")
        } catch (e: Exception) {
            throw UploadException.SaveFailed(e)
        }
    }

    /**
     * Packages a world directory into a ZIP file while simultaneously computing SHA256.
     * Returns the hex SHA256 hash of the ZIP file.
     */
    private fun packageWorld(
        worldDir: File,
        zipFile: File,
        totalSize: Long,
        progressCallback: (UploadProgress) -> Unit,
    ): String {
        val digest = MessageDigest.getInstance("SHA-256")
        var bytesProcessed = 0L

        try {
            ZipOutputStream(FileOutputStream(zipFile)).use { zos ->
                val basePath = worldDir.toPath()

                worldDir.walkTopDown().forEach { file ->
                    if (file == worldDir) return@forEach

                    val relativePath = basePath.relativize(file.toPath()).toString()

                    if (file.name in EXCLUDED_FILES) return@forEach

                    if (file.isDirectory) {
                        zos.putNextEntry(ZipEntry("$relativePath/"))
                        zos.closeEntry()
                    } else {
                        zos.putNextEntry(ZipEntry(relativePath))
                        FileInputStream(file).use { input ->
                            val buffer = ByteArray(8192)
                            while (true) {
                                val bytesRead = input.read(buffer)
                                if (bytesRead == -1) break
                                zos.write(buffer, 0, bytesRead)
                                bytesProcessed += bytesRead
                                progressCallback(UploadProgress.packaging(bytesProcessed, totalSize))
                            }
                        }
                        zos.closeEntry()
                    }
                }
            }

            // Compute SHA256 of the final ZIP file
            FileInputStream(zipFile).use { input ->
                val buffer = ByteArray(8192)
                while (true) {
                    val bytesRead = input.read(buffer)
                    if (bytesRead == -1) break
                    digest.update(buffer, 0, bytesRead)
                }
            }

            return digest.digest().joinToString("") { "%02x".format(it) }
        } catch (e: UploadException) {
            throw e
        } catch (e: Exception) {
            throw UploadException.PackagingFailed(e)
        }
    }

    /**
     * Uploads a ZIP file to a presigned S3/R2 URL with progress tracking.
     */
    private fun uploadToPresignedUrl(
        zipFile: File,
        presignedUrl: String,
        hash: String,
        progressCallback: (UploadProgress) -> Unit,
    ) {
        val totalBytes = zipFile.length()
        progressCallback(UploadProgress.uploading(0, totalBytes))

        val connection =
            try {
                URI(presignedUrl).toURL().openConnection() as HttpURLConnection
            } catch (e: IOException) {
                throw UploadException.NetworkError(presignedUrl, e)
            }

        try {
            connection.requestMethod = "PUT"
            connection.setRequestProperty("Content-Type", "application/zip")
            connection.setRequestProperty("Content-Length", totalBytes.toString())
            connection.setRequestProperty("x-amz-meta-sha256", hash)
            connection.doOutput = true
            connection.connectTimeout = 10000
            // Long timeout for large uploads
            connection.readTimeout = 300000

            // Stream the file to the connection
            connection.outputStream.use { output ->
                FileInputStream(zipFile).use { input ->
                    val buffer = ByteArray(8192)
                    var bytesUploaded = 0L

                    while (true) {
                        val bytesRead =
                            try {
                                input.read(buffer)
                            } catch (e: IOException) {
                                throw UploadException.UploadInterrupted(
                                    "Read failed after $bytesUploaded bytes",
                                    e,
                                )
                            }

                        if (bytesRead == -1) break

                        try {
                            output.write(buffer, 0, bytesRead)
                        } catch (e: IOException) {
                            throw UploadException.UploadInterrupted(
                                "Connection lost after uploading $bytesUploaded bytes",
                                e,
                            )
                        }

                        bytesUploaded += bytesRead
                        progressCallback(UploadProgress.uploading(bytesUploaded, totalBytes))
                    }

                    if (bytesUploaded != totalBytes) {
                        throw UploadException.UploadInterrupted(
                            "Incomplete upload: sent $bytesUploaded of $totalBytes bytes",
                        )
                    }
                }
            }

            val responseCode = connection.responseCode
            if (responseCode !in 200..299) {
                val errorBody = connection.errorStream?.readBytes()?.toString(Charsets.UTF_8)
                throw UploadException.HttpError(responseCode, errorBody)
            }

            log.info("Upload to presigned URL complete")
        } catch (e: UploadException) {
            throw e
        } catch (e: IOException) {
            throw UploadException.NetworkError(presignedUrl, e)
        } finally {
            connection.disconnect()
        }
    }

    private fun calculateDirectorySize(dir: File): Long =
        dir
            .walkTopDown()
            .filter { it.isFile && it.name !in EXCLUDED_FILES }
            .sumOf { it.length() }

    /**
     * Resolves the save directory for the currently loaded singleplayer world.
     * Returns null if not in singleplayer.
     */
    fun resolveCurrentWorldDir(): File? {
        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer ?: return null
        val levelName = server.worldData.levelName

        // Check glint/worlds first, then saves/
        val glintWorld = File(mc.gameDirectory, "glint/worlds/$levelName")
        if (glintWorld.exists() && glintWorld.isDirectory) return glintWorld

        val savesWorld = File(mc.gameDirectory, "saves/$levelName")
        if (savesWorld.exists() && savesWorld.isDirectory) return savesWorld

        return null
    }

    /**
     * Resolves the display name for the currently loaded world.
     * Returns null if not in singleplayer.
     */
    fun resolveCurrentWorldName(): String? {
        val mc = Minecraft.getInstance()
        return mc.singleplayerServer?.worldData?.levelName
    }

    /**
     * Lists all available world directories (saves/ and glint/worlds/).
     * Returns pairs of (directory, display name).
     */
    fun listLocalWorlds(): List<Pair<File, String>> {
        val mc = Minecraft.getInstance()
        val worlds = mutableListOf<Pair<File, String>>()

        // Saves directory
        val savesDir = File(mc.gameDirectory, "saves")
        if (savesDir.exists()) {
            savesDir.listFiles()?.filter { it.isDirectory && File(it, "level.dat").exists() }?.forEach {
                worlds.add(it to it.name)
            }
        }

        // Glint worlds directory
        val glintDir = File(mc.gameDirectory, "glint/worlds")
        if (glintDir.exists()) {
            glintDir.listFiles()?.filter { it.isDirectory && File(it, "level.dat").exists() }?.forEach {
                // Only add if not already in saves
                if (worlds.none { (dir, _) -> dir.name == it.name }) {
                    worlds.add(it to it.name)
                }
            }
        }

        return worlds.sortedBy { it.second.lowercase() }
    }
}
