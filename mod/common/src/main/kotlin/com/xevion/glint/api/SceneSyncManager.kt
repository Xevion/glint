package com.xevion.glint.api

import com.xevion.glint.Loggers
import com.xevion.glint.capture.Camera
import com.xevion.glint.capture.Position
import com.xevion.glint.scene.Scene
import com.xevion.glint.scene.SceneManager
import com.xevion.glint.scene.Weather
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Executors
import kotlin.math.abs

/**
 * Manages synchronization of scenes between local files and the Glint backend.
 * Server-authoritative model: pull replaces local scenes, push sends local changes to API.
 */
object SceneSyncManager {
    private val log = Loggers.Api.get()
    private val executor = Executors.newSingleThreadExecutor()

    private const val FLOAT_EPSILON = 0.001

    /**
     * Pulls scenes from the API and replaces local collection scenes.
     */
    fun pullScenes(
        worldId: String,
        collectionFileName: String,
        config: ApiConfig,
    ): CompletableFuture<PullResult> {
        if (!config.isValid()) {
            return CompletableFuture.completedFuture(
                PullResult.Failure(ApiError.ConfigError("API config not valid")),
            )
        }

        return CompletableFuture.supplyAsync(
            {
                log.info("Pulling scenes from API") {
                    "world_id" to worldId
                    "collection" to collectionFileName
                }

                val client = HttpClient(config.apiUrl, token = config.accessToken)
                val fetchResult = SceneClient.fetchScenes(client, worldId)

                fetchResult.fold(
                    onSuccess = { apiScenes ->
                        val localScenes = apiScenes.map { it.toLocalScene() }

                        val collection = SceneManager.loadCollection(collectionFileName)
                        val updatedCollection =
                            if (collection != null) {
                                collection.copy(scenes = localScenes)
                            } else {
                                com.xevion.glint.scene.SceneCollection(
                                    world = collectionFileName,
                                    apiWorldId = worldId,
                                    scenes = localScenes,
                                )
                            }

                        val saved = SceneManager.saveCollection(collectionFileName, updatedCollection)
                        if (!saved) {
                            log.error("Failed to save pulled scenes") { "collection" to collectionFileName }
                            return@supplyAsync PullResult.Failure(
                                ApiError.ConfigError("Failed to save collection to disk"),
                            )
                        }

                        log.info("Pulled scenes from API") {
                            "count" to localScenes.size
                            "collection" to collectionFileName
                        }
                        PullResult.Success(localScenes, localScenes.size)
                    },
                    onFailure = { throwable ->
                        val error =
                            throwable as? ApiError
                                ?: ApiError.UnknownError(throwable.message ?: "Unknown", throwable)
                        log.error("Failed to pull scenes") {
                            "world_id" to worldId
                            "error" to error.message
                        }
                        PullResult.Failure(error)
                    },
                )
            },
            executor,
        )
    }

    /**
     * Computes the diff between local scenes and API scenes.
     */
    fun computeDiff(
        localScenes: List<Scene>,
        apiScenes: List<ApiScene>,
    ): SceneDiff {
        val localBySlug = localScenes.associateBy { it.id }
        val apiBySlug = apiScenes.associateBy { it.slug }

        val toCreate = mutableListOf<Scene>()
        val toUpdate = mutableListOf<SceneUpdate>()
        val unchanged = mutableListOf<String>()

        for ((slug, local) in localBySlug) {
            val remote = apiBySlug[slug]
            if (remote == null) {
                toCreate.add(local)
            } else if (sceneDiffers(local, remote)) {
                toUpdate.add(SceneUpdate(local, remote))
            } else {
                unchanged.add(slug)
            }
        }

        val toRemove = apiScenes.filter { it.slug !in localBySlug }

        return SceneDiff(
            toCreate = toCreate,
            toUpdate = toUpdate,
            toRemove = toRemove,
            unchanged = unchanged,
        )
    }

    /**
     * Executes a push based on a pre-computed diff.
     */
    fun executePush(
        diff: SceneDiff,
        worldId: String,
        config: ApiConfig,
    ): CompletableFuture<PushResult> {
        if (!config.isValid()) {
            return CompletableFuture.completedFuture(
                PushResult.Failure(ApiError.ConfigError("API config not valid")),
            )
        }

        return CompletableFuture.supplyAsync(
            {
                log.info("Pushing scene changes to API") {
                    "world_id" to worldId
                    "create" to diff.toCreate.size
                    "update" to diff.toUpdate.size
                    "remove" to diff.toRemove.size
                }

                val client = HttpClient(config.apiUrl, token = config.accessToken)
                var created = 0
                var updated = 0
                var removed = 0

                for (scene in diff.toCreate) {
                    val result = SceneClient.createScene(client, worldId, scene)
                    result.fold(
                        onSuccess = {
                            log.debug("Created scene") { "slug" to scene.id }
                            created++
                        },
                        onFailure = { throwable ->
                            val error =
                                throwable as? ApiError
                                    ?: ApiError.UnknownError(throwable.message ?: "Unknown", throwable)
                            log.error("Failed to create scene") {
                                "slug" to scene.id
                                "error" to error.message
                            }
                            return@supplyAsync PushResult.Failure(error)
                        },
                    )
                }

                for (sceneUpdate in diff.toUpdate) {
                    val result =
                        SceneClient.updateScene(client, worldId, sceneUpdate.local)
                    result.fold(
                        onSuccess = {
                            log.debug("Updated scene") { "slug" to sceneUpdate.local.id }
                            updated++
                        },
                        onFailure = { throwable ->
                            val error =
                                throwable as? ApiError
                                    ?: ApiError.UnknownError(throwable.message ?: "Unknown", throwable)
                            log.error("Failed to update scene") {
                                "slug" to sceneUpdate.local.id
                                "error" to error.message
                            }
                            return@supplyAsync PushResult.Failure(error)
                        },
                    )
                }

                if (diff.toRemove.isNotEmpty()) {
                    val slugs = diff.toRemove.map { it.slug }
                    val result =
                        SceneClient.batchDisableScenes(client, worldId, slugs)
                    result.fold(
                        onSuccess = {
                            log.debug("Batch disabled scenes") { "count" to slugs.size }
                            removed = slugs.size
                        },
                        onFailure = { throwable ->
                            val error =
                                throwable as? ApiError
                                    ?: ApiError.UnknownError(throwable.message ?: "Unknown", throwable)
                            log.error("Failed to batch disable scenes") { "error" to error.message }
                            return@supplyAsync PushResult.Failure(error)
                        },
                    )
                }

                log.info("Push completed") {
                    "created" to created
                    "updated" to updated
                    "removed" to removed
                }
                PushResult.Success(created, updated, removed)
            },
            executor,
        )
    }

    private fun sceneDiffers(
        local: Scene,
        remote: ApiScene,
    ): Boolean {
        val v = remote.version
        if (local.dimension != remote.dimension) return true
        if (local.timeOfDay != v.timeOfDayTicks) return true
        if (local.weather.toMinecraftString() != v.weather) return true
        if (abs(local.weatherIntensity.toDouble() - v.weatherIntensity) > FLOAT_EPSILON) return true
        if (local.moonPhase != v.moonPhase) return true
        if (local.biome != v.biome) return true
        if (abs(local.position.x - v.x) > FLOAT_EPSILON) return true
        if (abs(local.position.y - v.y) > FLOAT_EPSILON) return true
        if (abs(local.position.z - v.z) > FLOAT_EPSILON) return true
        if (abs(local.camera.pitch.toDouble() - v.pitch) > FLOAT_EPSILON) return true
        if (abs(local.camera.yaw.toDouble() - v.yaw) > FLOAT_EPSILON) return true
        return false
    }

    fun shutdown() {
        executor.shutdown()
    }
}

fun ApiScene.toLocalScene(): Scene =
    Scene(
        id = slug,
        name = name,
        description = description,
        dimension = dimension,
        position = Position(x = version.x, y = version.y, z = version.z),
        camera = Camera(yaw = version.yaw.toFloat(), pitch = version.pitch.toFloat()),
        timeOfDay = version.timeOfDayTicks,
        weather = Weather.fromString(version.weather),
        weatherIntensity = version.weatherIntensity.toFloat(),
        moonPhase = version.moonPhase,
        biome = version.biome,
    )

data class SceneDiff(
    val toCreate: List<Scene>,
    val toUpdate: List<SceneUpdate>,
    val toRemove: List<ApiScene>,
    val unchanged: List<String>,
) {
    val hasChanges: Boolean get() = toCreate.isNotEmpty() || toUpdate.isNotEmpty() || toRemove.isNotEmpty()
    val changeCount: Int get() = toCreate.size + toUpdate.size + toRemove.size
}

data class SceneUpdate(
    val local: Scene,
    val remote: ApiScene,
)

sealed class PullResult {
    data class Success(
        val scenes: List<Scene>,
        val count: Int,
    ) : PullResult()

    data class Failure(
        val error: ApiError,
    ) : PullResult()
}

sealed class PushResult {
    data class Success(
        val created: Int,
        val updated: Int,
        val removed: Int,
    ) : PushResult()

    data class Failure(
        val error: ApiError,
    ) : PushResult()
}
