package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.WorkItem
import java.io.File
import java.io.IOException
import java.security.MessageDigest

/** Result of preparing all assets for a set of work items. */
sealed class PrepResult {
    data class Ready(
        val items: List<WorkItem>,
        /** Map of package hash → local ZIP file for downloaded scene packages. */
        val scenePackages: Map<String, File>,
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
 * Prepares assets (shaders, scene packages) for autonomous capture.
 *
 * Downloads all unique shaders and scene packages from the work item list,
 * caching scene packages by hash to avoid redundant downloads.
 */
class AssetPreparer(
    private val gameDirectory: File,
) {
    private val log = Loggers.Orchestration.get()

    /**
     * Prepares all assets needed for the given work items.
     *
     * Downloads shaders and scene packages, returning the items that are
     * ready for capture alongside a map of scene package paths.
     */
    fun prepareAll(items: List<WorkItem>): PrepResult {
        if (items.isEmpty()) return PrepResult.Failed("No work items")

        // 1. Download all unique shaders
        val shaderResults = downloadShaders(items)

        // 2. Download all unique scene packages
        val packageResults = downloadScenePackages(items)

        // 3. Filter items to only those with successfully downloaded shaders and packages
        val readyItems =
            items.filter { item ->
                val shaderReady =
                    if (item.shader.slug == "vanilla") {
                        true
                    } else {
                        shaderResults[item.shader.versionId] != null
                    }
                val packageReady =
                    item.scenePackage == null || packageResults.containsKey(item.scenePackage.hash)
                shaderReady && packageReady
            }

        if (readyItems.isEmpty()) {
            return PrepResult.Failed("No items ready after asset preparation")
        }

        log.info("Asset preparation complete") {
            "total_items" to items.size
            "ready_items" to readyItems.size
            "shaders" to shaderResults.size
            "packages" to packageResults.size
        }

        return PrepResult.Ready(readyItems, packageResults)
    }

    /**
     * Downloads all unique shaders from the work items.
     * Returns a map of shader version ID → filename for successfully downloaded shaders.
     */
    private fun downloadShaders(items: List<WorkItem>): Map<String, String> {
        val results = mutableMapOf<String, String>()

        val uniqueShaders =
            items
                .filter { it.shader.slug != "vanilla" }
                .distinctBy { it.shader.versionId }

        for (item in uniqueShaders) {
            val filename = downloadShader(item)
            if (filename != null) {
                results[item.shader.versionId] = filename
            } else {
                log.error("Failed to download shader") {
                    "shader" to item.shader.name
                    "version" to item.shader.version
                }
            }
        }

        return results
    }

    /** Downloads a single shader pack and returns its filename, or null on failure. */
    private fun downloadShader(item: WorkItem): String? {
        val shaderpacksDir = File(gameDirectory, "shaderpacks")
        shaderpacksDir.mkdirs()

        val hash8 = item.shader.fileHash?.take(8)
        val filename =
            if (hash8 != null) {
                "${item.shader.slug}-${item.shader.version}-$hash8.zip"
            } else {
                "${item.shader.slug}-${item.shader.version}.zip"
            }
        val targetFile = File(shaderpacksDir, filename)

        // Check if file already exists
        if (targetFile.exists()) {
            if (item.shader.fileHash != null) {
                val existingHash = sha1Hex(targetFile)
                if (existingHash == item.shader.fileHash) {
                    log.debug("Shader already present and verified") { "file" to filename }
                    return filename
                }
                log.warn("Shader file exists but hash mismatch, re-downloading") {
                    "file" to filename
                    "expected" to item.shader.fileHash
                    "actual" to existingHash
                }
                targetFile.delete()
            } else {
                log.debug("Shader already present") { "file" to filename }
                return filename
            }
        }

        val downloadUrl = item.shader.downloadUrl
        if (downloadUrl == null) {
            log.error("No download URL for shader") { "name" to item.shader.name }
            return null
        }

        log.info("Downloading shader") {
            "name" to item.shader.name
            "file" to filename
        }
        return downloadFile(downloadUrl, targetFile, item.shader.fileHash, "shader")?.let { filename }
    }

    /**
     * Downloads all unique scene packages from the work items.
     * Cached by package hash — unchanged packages are not re-downloaded.
     * Returns a map of package hash → local file path.
     */
    private fun downloadScenePackages(items: List<WorkItem>): Map<String, File> {
        val results = mutableMapOf<String, File>()
        val packagesDir = File(gameDirectory, "glint/packages")
        packagesDir.mkdirs()

        val uniquePackages =
            items
                .filter { it.scenePackage != null }
                .distinctBy { it.scenePackage!!.hash }

        for (item in uniquePackages) {
            val pkg = item.scenePackage!!
            val hash = pkg.hash
            val url = pkg.url
            val targetFile = File(packagesDir, "$hash.zip")

            // Cache hit: file exists with matching name (hash-based naming ensures correctness)
            if (targetFile.exists()) {
                log.debug("Scene package cached") {
                    "hash" to hash
                    "scene" to item.scene.name
                    "bytes" to targetFile.length()
                }
                results[hash] = targetFile
                continue
            }

            log.info("Downloading scene package") {
                "scene" to item.scene.name
                "hash" to hash
                "size" to (pkg.sizeBytes ?: -1)
            }

            val downloaded = downloadFile(url, targetFile, expectedHash = null, label = "scene package")
            if (downloaded != null) {
                results[hash] = targetFile
                log.info("Scene package downloaded") {
                    "hash" to hash
                    "bytes" to targetFile.length()
                }
            } else {
                log.error("Failed to download scene package") {
                    "scene" to item.scene.name
                    "hash" to hash
                }
            }
        }

        return results
    }

    /**
     * Downloads a file from a URL to a target file.
     * Optionally verifies SHA-1 hash after download.
     * Returns the target file on success, or null on failure.
     */
    private fun downloadFile(
        url: String,
        targetFile: File,
        expectedHash: String?,
        label: String,
    ): File? {
        try {
            val connection =
                java.net
                    .URI(url)
                    .toURL()
                    .openConnection() as java.net.HttpURLConnection
            connection.requestMethod = "GET"
            connection.connectTimeout = 10000
            connection.readTimeout = 120000

            if (connection.responseCode !in 200..299) {
                log.error("Download failed") {
                    "label" to label
                    "status" to connection.responseCode
                }
                return null
            }

            connection.inputStream.use { input ->
                targetFile.outputStream().use { output ->
                    input.copyTo(output)
                }
            }

            // Verify downloaded file hash if expected
            if (expectedHash != null) {
                val downloadedHash = sha1Hex(targetFile)
                if (downloadedHash != expectedHash) {
                    log.error("Downloaded file hash mismatch") {
                        "label" to label
                        "expected" to expectedHash
                        "actual" to downloadedHash
                    }
                    targetFile.delete()
                    return null
                }
            }

            return targetFile
        } catch (e: IOException) {
            log.error(e, "Failed to download file") { "label" to label }
            targetFile.delete()
            return null
        } catch (e: SecurityException) {
            log.error(e, "Failed to download file") { "label" to label }
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
}
