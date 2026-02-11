package com.xevion.glint.api

import com.xevion.glint.api.HttpClient.Method
import com.xevion.glint.capture.Camera
import com.xevion.glint.capture.Position
import com.xevion.glint.scene.Scene
import kotlinx.serialization.Serializable

/**
 * Scene CRUD operations against the Glint backend API.
 */
object SceneClient {
    fun fetchScenes(
        client: HttpClient,
        worldId: String,
    ): Result<List<ApiScene>> =
        client.request {
            path = "/api/scenes?worldId=$worldId"
        }

    fun createScene(
        client: HttpClient,
        worldId: String,
        scene: Scene,
    ): Result<ApiScene> =
        client.request {
            method = Method.POST
            path = "/api/scenes"
            jsonBody(
                CreateSceneRequest.serializer(),
                CreateSceneRequest(
                    worldId = worldId,
                    slug = scene.id,
                    name = scene.name,
                    position = scene.position,
                    camera = scene.camera,
                    dimension = scene.dimension,
                    timeOfDay = scene.timeOfDay,
                    weather = scene.weather.toMinecraftString(),
                    weatherIntensity = scene.weatherIntensity.toDouble(),
                    moonPhase = scene.moonPhase,
                    biome = scene.biome,
                ),
            )
            expectedStatus = setOf(201)
            onStatus(409) { ApiError.HttpError(409, "Scene already exists") }
        }

    fun updateScene(
        client: HttpClient,
        worldId: String,
        scene: Scene,
    ): Result<ApiScene> =
        client.request {
            method = Method.PUT
            path = "/api/scenes/by-slug/${scene.id}"
            jsonBody(
                UpdateSceneRequest.serializer(),
                UpdateSceneRequest(
                    worldId = worldId,
                    position = scene.position,
                    camera = scene.camera,
                    dimension = scene.dimension,
                    timeOfDay = scene.timeOfDay,
                    weather = scene.weather.toMinecraftString(),
                    weatherIntensity = scene.weatherIntensity.toDouble(),
                    moonPhase = scene.moonPhase,
                    biome = scene.biome,
                ),
            )
            onStatus(404) { ApiError.HttpError(404, "Scene not found") }
        }

    fun disableScene(
        client: HttpClient,
        worldId: String,
        sceneSlug: String,
    ): Result<Unit> =
        client.requestUnit {
            method = Method.DELETE
            path = "/api/scenes/by-slug/$sceneSlug?worldId=$worldId"
            expectedStatus = setOf(204)
            onStatus(404) { ApiError.HttpError(404, "Scene not found") }
        }

    fun batchDisableScenes(
        client: HttpClient,
        worldId: String,
        slugs: List<String>,
    ): Result<Unit> =
        client.requestUnit {
            method = Method.DELETE
            path = "/api/scenes/batch?worldId=$worldId"
            jsonBody(BatchDisableRequest.serializer(), BatchDisableRequest(slugs))
            expectedStatus = setOf(204)
        }
}

@Serializable
data class CreateSceneRequest(
    val worldId: String,
    val slug: String,
    val name: String,
    val position: Position,
    val camera: Camera,
    val dimension: String,
    val timeOfDay: Int,
    val weather: String,
    val weatherIntensity: Double,
    val moonPhase: Int?,
    val biome: String?,
)

@Serializable
data class UpdateSceneRequest(
    val worldId: String,
    val position: Position,
    val camera: Camera,
    val dimension: String,
    val timeOfDay: Int,
    val weather: String,
    val weatherIntensity: Double,
    val moonPhase: Int?,
    val biome: String?,
)

@Serializable
data class BatchDisableRequest(
    val slugs: List<String>,
)

@Serializable
data class ApiSceneVersion(
    val id: String,
    val sceneId: String,
    val x: Double,
    val y: Double,
    val z: Double,
    val pitch: Double,
    val yaw: Double,
    val timeOfDayTicks: Int,
    val weather: String,
    val weatherIntensity: Double,
    val moonPhase: Int?,
    val biome: String?,
    val createdAt: String,
)

@Serializable
data class ApiScene(
    val id: String,
    val name: String,
    val slug: String,
    val description: String?,
    val worldId: String,
    val dimension: String,
    val parentSceneId: String? = null,
    val active: Boolean,
    val createdAt: String,
    val version: ApiSceneVersion,
)
