package com.xevion.glint.api

import com.xevion.glint.Loggers
import com.xevion.glint.scene.BackendSyncState
import com.xevion.glint.scene.LocalSceneMetadata
import com.xevion.glint.scene.LocalSceneStore
import com.xevion.glint.scene.ScenePackageMeta
import com.xevion.glint.scene.SceneState
import com.xevion.glint.scene.scenePackageJson
import java.io.File
import java.security.MessageDigest
import java.time.Instant
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Executors
import java.util.zip.ZipFile

/**
 * Reconciles local scene packages with the Glint backend scene list.
 * Replaces the old world-based pull/push model with package-level sync.
 */
object SceneSyncManager {
    private val log = Loggers.Api.get()
    private val executor = Executors.newSingleThreadExecutor()

    /** Pull scene list from backend and reconcile with local index. */
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
                val fetchResult = SceneClient.fetchScenes(client)

                fetchResult.fold(
                    onSuccess = { apiScenes ->
                        reconcileIndex(apiScenes)
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

    private fun reconcileIndex(apiScenes: List<ApiSceneListItem>): ReconcileResult {
        val index = LocalSceneStore.loadIndex()
        val localSlugs = index.scenes.keys.toMutableSet()
        val apiBySlug = apiScenes.associateBy { it.slug }

        val now = Instant.now().toString()
        var matched = 0
        var stale = 0
        var remoteOnly = 0

        for ((slug, apiScene) in apiBySlug) {
            val localEntry = index.scenes[slug]
            if (localEntry != null) {
                localSlugs.remove(slug)
                val localMeta = LocalSceneStore.loadMetadata(slug)
                val localHash = localMeta?.packageHash
                val remoteHash = apiScene.version.packageHash

                LocalSceneStore.markSynced(
                    slug,
                    BackendSyncState(
                        sceneId = apiScene.id,
                        latestVersionId = apiScene.version.id,
                        syncedAt = now,
                        syncedVersionHash = remoteHash ?: "",
                    ),
                )
                if (localHash != null && localHash == remoteHash) matched++ else stale++
            } else {
                // Remote-only: exists on backend but not locally.
                // No action needed — user can download later via downloadScene().
                remoteOnly++
            }
        }

        // Remaining localSlugs are local-only (not on backend)
        val localOnly = localSlugs.size

        log.info("Reconciliation complete") {
            "matched" to matched
            "stale" to stale
            "remoteOnly" to remoteOnly
            "localOnly" to localOnly
        }

        return ReconcileResult.Success(
            matched = matched,
            stale = stale,
            remoteOnly = remoteOnly,
            localOnly = localOnly,
        )
    }

    /** Upload a local scene to the backend. */
    @Suppress("LongMethod")
    fun uploadScene(
        slug: String,
        config: ApiConfig,
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

                // Initiate upload — new scene or new version
                val initiateResult =
                    if (metadata.backend == null) {
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

                // Update local sync state
                LocalSceneStore.markSynced(
                    slug,
                    BackendSyncState(
                        sceneId = completed.sceneId,
                        latestVersionId = completed.sceneVersionId,
                        syncedAt = Instant.now().toString(),
                        syncedVersionHash = fileHash,
                    ),
                )

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
                        state = SceneState.SYNCED,
                        exportedAt = now,
                        camera = meta.camera,
                        fov = meta.fov,
                        renderDistance = meta.renderDistance,
                        environment = meta.environment,
                        chunkBounds = meta.chunkBounds,
                        entityCount = 0,
                        packageHash = fileHash,
                        packageSizeBytes = targetFile.length(),
                        backend =
                            BackendSyncState(
                                sceneId = detail.id,
                                latestVersionId = detail.version.id,
                                syncedAt = now,
                                syncedVersionHash = detail.version.packageHash ?: fileHash,
                            ),
                    )

                LocalSceneStore.registerExport(slug, localMeta)
                // registerExport sets state to LOCAL; mark as SYNCED
                LocalSceneStore.markSynced(slug, localMeta.backend!!)

                log.info("Scene downloaded successfully") {
                    "slug" to slug
                    "sceneId" to detail.id
                }

                DownloadResult.Success(slug = slug, sceneId = detail.id)
            },
            executor,
        )
    }

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
        val matched: Int,
        val stale: Int,
        val remoteOnly: Int,
        val localOnly: Int,
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
