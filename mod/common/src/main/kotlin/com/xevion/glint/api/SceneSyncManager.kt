package com.xevion.glint.api

import com.xevion.glint.Glint
import com.xevion.glint.scene.Scene
import com.xevion.glint.scene.SceneCollection
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Executors

/**
 * Manages synchronization of scenes between local files and the Glint backend.
 */
object SceneSyncManager {
    private val executor = Executors.newSingleThreadExecutor()

    /**
     * Syncs a single scene to the backend.
     * Automatically determines whether to create or update based on existence.
     *
     * @return CompletableFuture with SyncResult
     */
    fun syncScene(
        scene: Scene,
        config: ApiConfig,
    ): CompletableFuture<SyncResult> {
        if (!config.isValid()) {
            return CompletableFuture.completedFuture(
                SyncResult.Failure(scene.id, ApiError.ConfigError("API config not valid")),
            )
        }

        return CompletableFuture.supplyAsync(
            {
                Glint.LOGGER.info("Syncing scene: ${scene.id}")

                // Try to create first - backend will return 409 if exists
                val createResult = GlintApi.createScene(config.apiUrl, config.worldId, scene, config.accessToken)

                createResult.fold(
                    onSuccess = {
                        Glint.LOGGER.info("Created scene '${scene.id}' on backend")
                        SyncResult.Success(scene.id, created = true)
                    },
                    onFailure = { throwable ->
                        val error = throwable as? ApiError ?: ApiError.UnknownError(throwable.message ?: "Unknown", throwable)

                        if (error is ApiError.HttpError && error.statusCode == 409) {
                            // Scene exists, try updating
                            Glint.LOGGER.debug("Scene '${scene.id}' exists, updating instead")
                            val updateResult = GlintApi.updateScene(config.apiUrl, config.worldId, scene, config.accessToken)

                            updateResult.fold(
                                onSuccess = {
                                    Glint.LOGGER.info("Updated scene '${scene.id}' on backend")
                                    SyncResult.Success(scene.id, created = false)
                                },
                                onFailure = { updateThrowable ->
                                    val updateError =
                                        updateThrowable as? ApiError
                                            ?: ApiError.UnknownError(updateThrowable.message ?: "Unknown", updateThrowable)
                                    Glint.LOGGER.error("Failed to update scene '${scene.id}': ${updateError.message}")
                                    SyncResult.Failure(scene.id, updateError)
                                },
                            )
                        } else {
                            // Connection error or other issue
                            val logLevel =
                                when (error) {
                                    is ApiError.NetworkError -> "warn"
                                    is ApiError.HttpError -> if (error.statusCode >= 500) "warn" else "error"
                                    else -> "error"
                                }

                            if (logLevel == "warn") {
                                Glint.LOGGER.warn("Failed to create scene '${scene.id}': ${error.userMessage}")
                            } else {
                                Glint.LOGGER.error("Failed to create scene '${scene.id}': ${error.message}")
                            }

                            SyncResult.Failure(scene.id, error)
                        }
                    },
                )
            },
            executor,
        )
    }

    /**
     * Syncs all scenes in a collection to the backend.
     *
     * @return CompletableFuture with list of SyncResults
     */
    fun syncCollection(
        collection: SceneCollection,
        config: ApiConfig,
    ): CompletableFuture<List<SyncResult>> {
        if (!config.isValid()) {
            val configError = ApiError.ConfigError("API config not valid")
            return CompletableFuture.completedFuture(
                collection.scenes.map { SyncResult.Failure(it.id, configError) },
            )
        }

        val futures = collection.scenes.map { scene -> syncScene(scene, config) }

        return CompletableFuture.allOf(*futures.toTypedArray()).thenApply {
            val results = futures.map { it.join() }

            // Log summary
            val successCount = results.count { it is SyncResult.Success }
            val failureCount = results.count { it is SyncResult.Failure }

            if (failureCount > 0) {
                Glint.LOGGER.warn("Synced $successCount/${collection.scenes.size} scenes ($failureCount failed)")
            } else {
                Glint.LOGGER.info("Successfully synced all ${collection.scenes.size} scenes")
            }

            results
        }
    }

    /**
     * Disables a scene on the backend (called when scene is deleted locally).
     */
    fun disableScene(
        sceneSlug: String,
        config: ApiConfig,
    ): CompletableFuture<Boolean> {
        if (!config.isValid()) {
            Glint.LOGGER.debug("Cannot disable scene '$sceneSlug': API config not valid")
            return CompletableFuture.completedFuture(false)
        }

        return CompletableFuture.supplyAsync(
            {
                Glint.LOGGER.info("Disabling scene on backend: $sceneSlug")
                val result = GlintApi.disableScene(config.apiUrl, config.worldId, sceneSlug, config.accessToken)

                result.fold(
                    onSuccess = {
                        Glint.LOGGER.info("Successfully disabled scene '$sceneSlug' on backend")
                        true
                    },
                    onFailure = { throwable ->
                        val error = throwable as? ApiError ?: ApiError.UnknownError(throwable.message ?: "Unknown", throwable)

                        when (error) {
                            is ApiError.NetworkError -> {
                                Glint.LOGGER.warn("Cannot disable scene '$sceneSlug': ${error.userMessage}")
                            }

                            is ApiError.HttpError -> {
                                if (error.statusCode == 404) {
                                    Glint.LOGGER.debug("Scene '$sceneSlug' not found on backend (already deleted?)")
                                } else {
                                    Glint.LOGGER.error("Failed to disable scene '$sceneSlug': ${error.message}")
                                }
                            }

                            else -> {
                                Glint.LOGGER.error("Failed to disable scene '$sceneSlug': ${error.message}")
                            }
                        }
                        false
                    },
                )
            },
            executor,
        )
    }

    /**
     * Shuts down the sync executor (call on mod shutdown).
     */
    fun shutdown() {
        executor.shutdown()
    }
}

/**
 * Result of a scene sync operation.
 */
sealed class SyncResult {
    abstract val sceneId: String

    data class Success(
        override val sceneId: String,
        val created: Boolean,
    ) : SyncResult()

    data class Failure(
        override val sceneId: String,
        val error: ApiError,
    ) : SyncResult() {
        val userMessage: String get() = error.userMessage
    }
}
