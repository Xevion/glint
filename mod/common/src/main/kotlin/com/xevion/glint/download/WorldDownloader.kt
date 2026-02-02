package com.xevion.glint.download

import com.xevion.glint.Glint
import net.minecraft.client.Minecraft
import java.io.File
import java.io.FileOutputStream
import java.io.IOException
import java.net.HttpURLConnection
import java.net.URI
import java.security.MessageDigest
import java.util.concurrent.CompletableFuture
import java.util.zip.ZipInputStream
import javax.net.ssl.SSLException

/**
 * Handles downloading and extracting world files from the API.
 * Downloads to .minecraft/glint/worlds/{slug}_{random}/
 */
object WorldDownloader {
    private val downloadedWorlds = mutableSetOf<String>()

    /**
     * Downloads and extracts a world from the given URL.
     * Returns the folder name to load (relative to .minecraft).
     *
     * @param worldSlug Human-readable world slug
     * @param worldId UUID from API
     * @param fileUrl Download URL from API
     * @param expectedHash SHA256 hash with "sha256:" prefix
     * @param progressCallback Called with download progress
     * @return CompletableFuture with relative path to world folder
     */
    fun downloadWorld(
        worldSlug: String,
        worldId: String,
        fileUrl: String,
        expectedHash: String?,
        progressCallback: (DownloadProgress) -> Unit,
    ): CompletableFuture<String> = CompletableFuture.supplyAsync {
        var worldDir: File? = null
        try {
            val folderName = generateFolderName(worldSlug)
            val worldsDir = getWorldsDirectory()
            worldDir = File(worldsDir, folderName)

            worldDir.mkdirs()

            Glint.LOGGER.info("Downloading world {} to {}", worldSlug, worldDir.absolutePath)

            val zipFile = File(worldDir, "world.zip")

            downloadFile(fileUrl, zipFile, progressCallback)

            if (expectedHash != null) {
                verifyHash(zipFile, expectedHash)
            }

            progressCallback(DownloadProgress.extracting())
            extractZip(zipFile, worldDir)

            zipFile.delete()

            progressCallback(DownloadProgress.complete())

            downloadedWorlds.add(folderName)

            "glint/worlds/$folderName"
        } catch (e: WorldDownloadException) {
            Glint.LOGGER.error("World download failed", e)
            progressCallback(DownloadProgress.failed(e.message ?: "Unknown error"))
            worldDir?.deleteRecursively()
            throw e
        } catch (e: Exception) {
            val message = "Unexpected error downloading world: ${e.message}"
            Glint.LOGGER.error(message, e)
            progressCallback(DownloadProgress.failed(message))
            worldDir?.deleteRecursively()
            throw WorldDownloadException.DownloadInterrupted(message, e)
        }
    }

    /**
     * Checks if a world has been downloaded this session.
     */
    fun isWorldDownloaded(folderName: String): Boolean = downloadedWorlds.contains(folderName)

    /**
     * Cleans up a specific downloaded world.
     */
    fun cleanupWorld(folderName: String): Boolean {
        val worldsDir = getWorldsDirectory()
        val worldDir = File(worldsDir, folderName)

        return if (worldDir.exists() && worldDir.isDirectory) {
            val deleted = worldDir.deleteRecursively()
            if (deleted) {
                downloadedWorlds.remove(folderName)
                Glint.LOGGER.info("Cleaned up world: {}", folderName)
            }
            deleted
        } else {
            false
        }
    }

    /**
     * Cleans up all worlds downloaded this session.
     */
    fun cleanupAllDownloads() {
        val worldsToCleanup = downloadedWorlds.toList()
        worldsToCleanup.forEach { folderName ->
            cleanupWorld(folderName)
        }
        Glint.LOGGER.info("Cleaned up {} downloaded worlds", worldsToCleanup.size)
    }

    private fun getWorldsDirectory(): File {
        val mc = Minecraft.getInstance()
        val worldsDir = File(mc.gameDirectory, "glint/worlds")
        worldsDir.mkdirs()
        return worldsDir
    }

    private fun generateFolderName(slug: String): String {
        val random = (1..6).map { ('a'..'z').random() }.joinToString("")
        return "${slug}_$random"
    }

    private fun downloadFile(
        url: String,
        outputFile: File,
        progressCallback: (DownloadProgress) -> Unit,
    ) {
        val connection =
            try {
                URI(url).toURL().openConnection() as HttpURLConnection
            } catch (e: SSLException) {
                throw WorldDownloadException.NetworkError(url, e)
            } catch (e: IOException) {
                throw WorldDownloadException.NetworkError(url, e)
            }

        try {
            connection.requestMethod = "GET"
            connection.connectTimeout = 10000
            connection.readTimeout = 30000

            val responseCode = connection.responseCode
            if (responseCode !in 200..299) {
                throw WorldDownloadException.HttpError(responseCode, url)
            }

            connection.inputStream.use { input ->
                FileOutputStream(outputFile).use { output ->
                    val totalBytes = connection.contentLengthLong
                    var bytesDownloaded = 0L
                    val buffer = ByteArray(8192)

                    while (true) {
                        val bytesRead =
                            try {
                                input.read(buffer)
                            } catch (e: IOException) {
                                throw WorldDownloadException.DownloadInterrupted(
                                    "Connection lost after downloading $bytesDownloaded bytes",
                                    e,
                                )
                            }

                        if (bytesRead == -1) break

                        output.write(buffer, 0, bytesRead)
                        bytesDownloaded += bytesRead

                        progressCallback(DownloadProgress.downloading(bytesDownloaded, totalBytes))
                    }

                    if (totalBytes > 0 && bytesDownloaded != totalBytes) {
                        throw WorldDownloadException.DownloadInterrupted(
                            "Incomplete download: got $bytesDownloaded of $totalBytes bytes",
                        )
                    }
                }
            }
        } catch (e: WorldDownloadException) {
            outputFile.delete()
            throw e
        } catch (e: SSLException) {
            outputFile.delete()
            throw WorldDownloadException.NetworkError(url, e)
        } catch (e: IOException) {
            outputFile.delete()
            throw WorldDownloadException.NetworkError(url, e)
        } finally {
            connection.disconnect()
        }
    }

    private fun verifyHash(
        file: File,
        expectedHash: String,
    ) {
        val hash = computeSha256(file)
        val expected = expectedHash.removePrefix("sha256:")

        if (!hash.equals(expected, ignoreCase = true)) {
            throw WorldDownloadException.HashMismatch(expected, hash)
        }

        Glint.LOGGER.debug("Hash verification passed for {}", file.name)
    }

    private fun computeSha256(file: File): String {
        val digest = MessageDigest.getInstance("SHA-256")
        file.inputStream().use { input ->
            val buffer = ByteArray(8192)
            while (true) {
                val bytesRead = input.read(buffer)
                if (bytesRead == -1) break
                digest.update(buffer, 0, bytesRead)
            }
        }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }

    private fun extractZip(
        zipFile: File,
        targetDir: File,
    ) {
        try {
            ZipInputStream(zipFile.inputStream()).use { zis ->
                var entry = zis.nextEntry

                while (entry != null) {
                    val file = File(targetDir, entry.name)

                    if (entry.isDirectory) {
                        file.mkdirs()
                    } else {
                        file.parentFile.mkdirs()
                        FileOutputStream(file).use { output ->
                            zis.copyTo(output)
                        }
                    }

                    zis.closeEntry()
                    entry = zis.nextEntry
                }
            }

            Glint.LOGGER.info("Extracted world to {}", targetDir.absolutePath)
        } catch (e: Exception) {
            throw WorldDownloadException.ExtractionFailed(e)
        }
    }
}
