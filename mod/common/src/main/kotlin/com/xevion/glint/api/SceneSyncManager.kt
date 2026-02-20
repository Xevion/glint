package com.xevion.glint.api

import com.xevion.glint.Loggers
import com.xevion.glint.scene.LocalSceneMetadata
import com.xevion.glint.scene.LocalSceneStore
import com.xevion.glint.scene.ScenePackageMeta
import com.xevion.glint.scene.scenePackageJson
import java.io.File
import java.security.MessageDigest
import java.time.Instant
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Executors
import java.util.zip.ZipFile

/**
 * Reconciles local scene packages with the Glint backend.
 * Sync status is computed live by the server — never persisted locally.
 */
object SceneSyncManager {
    private val log = Loggers.Api.get()
    private val executor = Executors.newSingleThreadExecutor()

    /** Reconcile local scenes against the backend, returning per-scene sync status. */
    fun reconcile(config: ApiConfig): CompletableFuture<ReconcileResult> {
        if (!config.isValid()) {
            return CompletableFuture.completedFuture(
                ReconcileResult.Failure(ApiError.ConfigError("API config not valid")),
            )
        }

        return CompletableFuture.supplyAsync(
            {
                log.info("Reconciling scenes with backend")

                val client = HttpClient(config.apiUrl, token = config.accessToken)

                // Build local manifest: slug → package hash
                val index = LocalSceneStore.loadIndex()
                val localScenes =
                    index.scenes.mapValues { (slug, _) ->
                        val meta = LocalSceneStore.loadMetadata(slug)
                        ReconcileLocalScene(packageHash = meta?.packageHash?.normalizeHash())
                    }

                val result = SceneClient.reconcile(client, localScenes)

                result.fold(
                    onSuccess = { response ->
                        val statusCounts =
                            response.scenes.values
                                .groupingBy { it.status }
                                .eachCount()
                        log.info("Reconciliation complete") {
                            "synced" to (statusCounts[SyncStatus.SYNCED] ?: 0)
                            "localOnly" to (statusCounts[SyncStatus.LOCAL_ONLY] ?: 0)
                            "remoteOnly" to (statusCounts[SyncStatus.REMOTE_ONLY] ?: 0)
                            "localAhead" to (statusCounts[SyncStatus.LOCAL_AHEAD] ?: 0)
                            "remoteAhead" to (statusCounts[SyncStatus.REMOTE_AHEAD] ?: 0)
                        }
                        ReconcileResult.Success(response.scenes)
                    },
                    onFailure = { throwable ->
                        val error = ApiError.from(throwable)
                        log.error("Failed to reconcile scenes") { "error" to error.message }
                        ReconcileResult.Failure(error)
                    },
                )
            },
            executor,
        )
    }

    /** Upload a local scene to the backend. */
    @Suppress("LongMethod")
    fun uploadScene(
        slug: String,
        config: ApiConfig,
        syncStatus: SyncStatus?,
        onProgress: ((bytesUploaded: Long, totalBytes: Long) -> Unit)? = null,
    ): CompletableFuture<UploadResult> {
        if (!config.isValid()) {
            return CompletableFuture.completedFuture(
                UploadResult.Failure(ApiError.ConfigError("API config not valid")),
            )
        }

        return CompletableFuture.supplyAsync(
            {
                log.info("Uploading scene") { "slug" to slug }

                val metadata = LocalSceneStore.loadMetadata(slug)
                if (metadata == null) {
                    log.error("Scene metadata not found") { "slug" to slug }
                    return@supplyAsync UploadResult.Failure(
                        ApiError.ConfigError("Scene metadata not found for slug: $slug"),
                    )
                }

                val packageFile = LocalSceneStore.packagePath(slug)
                if (!packageFile.exists()) {
                    log.error("Package file not found") {
                        "slug" to slug
                        "path" to packageFile.absolutePath
                    }
                    return@supplyAsync UploadResult.Failure(
                        ApiError.ConfigError("Package file not found: ${packageFile.absolutePath}"),
                    )
                }

                val fileHash = computeFileHash(packageFile)
                val sizeBytes = packageFile.length()
                val client = HttpClient(config.apiUrl, token = config.accessToken)

                // Decide new scene vs new version based on live sync status
                val isNewScene = syncStatus == null || syncStatus == SyncStatus.LOCAL_ONLY || syncStatus == SyncStatus.UNKNOWN
                val initiateResult =
                    if (isNewScene) {
                        SceneUploadClient.initiateNewScene(
                            client,
                            InitiateNewSceneUploadRequest(
                                name = metadata.name,
                                slug = slug,
                                description = metadata.description,
                                dimension = metadata.dimension,
                                minecraftVersion = metadata.minecraftVersion,
                                fileHash = fileHash,
                                sizeBytes = sizeBytes,
                            ),
                        )
                    } else {
                        SceneUploadClient.initiateVersion(
                            client,
                            slug,
                            InitiateVersionUploadRequest(
                                fileHash = fileHash,
                                sizeBytes = sizeBytes,
                                minecraftVersion = metadata.minecraftVersion,
                            ),
                        )
                    }

                val initiated =
                    initiateResult.getOrElse { throwable ->
                        val error = ApiError.from(throwable)
                        log.error("Failed to initiate upload") {
                            "slug" to slug
                            "error" to error.message
                        }
                        return@supplyAsync UploadResult.Failure(error)
                    }

                log.info("Upload initiated, uploading package") {
                    "slug" to slug
                    "uploadId" to initiated.uploadId
                    "sizeBytes" to sizeBytes
                }

                // Upload the package file
                val uploadResult =
                    SceneUploadClient.uploadPackage(initiated.uploadUrl, packageFile) { bytesUploaded, totalBytes ->
                        val percent = if (totalBytes > 0) ((bytesUploaded * 100) / totalBytes).toInt() else 0
                        log.debug("Upload progress") {
                            "slug" to slug
                            "percent" to percent
                        }
                        onProgress?.invoke(bytesUploaded, totalBytes)
                    }

                uploadResult.getOrElse { throwable ->
                    val error = ApiError.from(throwable)
                    log.error("Failed to upload package") {
                        "slug" to slug
                        "error" to error.message
                    }
                    return@supplyAsync UploadResult.Failure(error)
                }

                // Complete the upload
                val completeResult =
                    SceneUploadClient.completeUpload(
                        client,
                        initiated.uploadId,
                        CompleteUploadRequest(
                            fileHash = fileHash,
                            camera =
                                UploadCamera(
                                    yaw = metadata.camera.yaw.toDouble(),
                                    pitch = metadata.camera.pitch.toDouble(),
                                ),
                            environment =
                                UploadEnvironment(
                                    timeOfDayTicks = metadata.environment.time,
                                    weather = metadata.environment.weather,
                                    weatherIntensity = metadata.environment.weatherIntensity.toDouble(),
                                    moonPhase = metadata.environment.moonPhase,
                                ),
                            fov = metadata.fov,
                            renderDistance = metadata.renderDistance,
                            position =
                                UploadPosition(
                                    x = metadata.camera.x,
                                    y = metadata.camera.y,
                                    z = metadata.camera.z,
                                ),
                        ),
                    )

                val completed =
                    completeResult.getOrElse { throwable ->
                        val error = ApiError.from(throwable)
                        log.error("Failed to complete upload") {
                            "slug" to slug
                            "error" to error.message
                        }
                        return@supplyAsync UploadResult.Failure(error)
                    }

                log.info("Scene uploaded successfully") {
                    "slug" to slug
                    "sceneId" to completed.sceneId
                    "versionId" to completed.sceneVersionId
                }

                UploadResult.Success(
                    sceneId = completed.sceneId,
                    versionId = completed.sceneVersionId,
                )
            },
            executor,
        )
    }

    /** Download a remote-only scene's package to local cache. */
    fun downloadScene(
        slug: String,
        config: ApiConfig,
    ): CompletableFuture<DownloadResult> {
        if (!config.isValid()) {
            return CompletableFuture.completedFuture(
                DownloadResult.Failure(ApiError.ConfigError("API config not valid")),
            )
        }

        return CompletableFuture.supplyAsync(
            {
                log.info("Downloading scene") { "slug" to slug }

                val client = HttpClient(config.apiUrl, token = config.accessToken)
                val detailResult = SceneClient.fetchScene(client, slug)

                val details =
                    detailResult.getOrElse { throwable ->
                        val error = ApiError.from(throwable)
                        log.error("Failed to fetch scene details") {
                            "slug" to slug
                            "error" to error.message
                        }
                        return@supplyAsync DownloadResult.Failure(error)
                    }

                val detail = details.firstOrNull()
                if (detail == null) {
                    log.error("Scene not found on backend") { "slug" to slug }
                    return@supplyAsync DownloadResult.Failure(
                        ApiError.HttpError(404, "Scene not found: $slug"),
                    )
                }

                val packageUrl = detail.version.packageUrl
                if (packageUrl == null) {
                    log.error("Scene has no package URL") { "slug" to slug }
                    return@supplyAsync DownloadResult.Failure(
                        ApiError.ConfigError("Scene has no package URL: $slug"),
                    )
                }

                // Download the package
                val targetFile = LocalSceneStore.packagePath(slug)
                targetFile.parentFile.mkdirs()

                val downloadResult = downloadFile(packageUrl, targetFile)
                if (downloadResult.isFailure) {
                    val error =
                        ApiError.from(
                            downloadResult.exceptionOrNull() ?: Exception("Download failed"),
                        )
                    log.error("Failed to download package") {
                        "slug" to slug
                        "error" to error.message
                    }
                    return@supplyAsync DownloadResult.Failure(error)
                }

                // Extract meta.json from the package
                val meta = extractPackageMeta(targetFile)
                if (meta == null) {
                    targetFile.delete()
                    log.error("Failed to extract meta.json from package") { "slug" to slug }
                    return@supplyAsync DownloadResult.Failure(
                        ApiError.ConfigError("Failed to extract meta.json from package"),
                    )
                }

                val fileHash = computeFileHash(targetFile)
                val now = Instant.now().toString()

                val localMeta =
                    LocalSceneMetadata(
                        slug = slug,
                        name = detail.name,
                        description = detail.description,
                        dimension = detail.dimension,
                        minecraftVersion = meta.minecraftVersion,
                        exportedAt = now,
                        camera = meta.camera,
                        fov = meta.fov,
                        renderDistance = meta.renderDistance,
                        environment = meta.environment,
                        chunkBounds = meta.chunkBounds,
                        entityCount = 0,
                        packageHash = fileHash,
                        packageSizeBytes = targetFile.length(),
                    )

                LocalSceneStore.registerExport(slug, localMeta)

                log.info("Scene downloaded successfully") {
                    "slug" to slug
                    "sceneId" to detail.id
                }

                DownloadResult.Success(slug = slug, sceneId = detail.id)
            },
            executor,
        )
    }

    /** Ensure hash has `sha256:` prefix for comparison with server-stored hashes. */
    private fun String.normalizeHash(): String = if (startsWith("sha256:")) this else "sha256:$this"

    private fun computeFileHash(file: File): String {
        val digest = MessageDigest.getInstance("SHA-256")
        file.inputStream().use { input ->
            val buffer = ByteArray(8192)
            while (true) {
                val bytesRead = input.read(buffer)
                if (bytesRead == -1) break
                digest.update(buffer, 0, bytesRead)
            }
        }
        val hash = digest.digest().joinToString("") { "%02x".format(it) }
        return "sha256:$hash"
    }

    private fun extractPackageMeta(packageFile: File): ScenePackageMeta? =
        try {
            ZipFile(packageFile).use { zip ->
                val entry = zip.getEntry("meta.json") ?: return null
                val content = zip.getInputStream(entry).bufferedReader().readText()
                scenePackageJson.decodeFromString(ScenePackageMeta.serializer(), content)
            }
        } catch (e: Exception) {
            log.error(e, "Failed to parse meta.json from package") { "path" to packageFile.absolutePath }
            null
        }

    private fun downloadFile(
        url: String,
        target: File,
    ): Result<Unit> {
        val connection =
            try {
                java.net
                    .URI(url)
                    .toURL()
                    .openConnection() as java.net.HttpURLConnection
            } catch (e: Exception) {
                return Result.failure(ApiError.fromException(e))
            }

        connection.requestMethod = "GET"
        connection.connectTimeout = 10000
        connection.readTimeout = 120000

        return try {
            if (connection.responseCode !in 200..299) {
                return Result.failure(
                    ApiError.HttpError(connection.responseCode, "Failed to download file"),
                )
            }

            connection.inputStream.use { input ->
                target.outputStream().use { output ->
                    input.copyTo(output, bufferSize = 8192)
                }
            }
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(ApiError.fromException(e))
        } finally {
            connection.disconnect()
        }
    }

    fun shutdown() {
        executor.shutdown()
    }
}

// Result types

sealed class ReconcileResult {
    data class Success(
        val scenes: Map<String, ReconcileSceneStatus>,
    ) : ReconcileResult()

    data class Failure(
        val error: ApiError,
    ) : ReconcileResult()
}

sealed class UploadResult {
    data class Success(
        val sceneId: String,
        val versionId: String,
    ) : UploadResult()

    data class Failure(
        val error: ApiError,
    ) : UploadResult()
}

sealed class DownloadResult {
    data class Success(
        val slug: String,
        val sceneId: String,
    ) : DownloadResult()

    data class Failure(
        val error: ApiError,
    ) : DownloadResult()
}
