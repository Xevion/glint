package com.xevion.glint.api

import com.xevion.glint.api.HttpClient.Method
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/**
 * Scene CRUD and preset management against the Glint backend API.
 */
object SceneClient {
    fun fetchScenes(client: HttpClient): Result<List<ApiSceneListItem>> =
        client.request {
            path = "/api/scenes"
        }

    fun fetchScene(
        client: HttpClient,
        slug: String,
    ): Result<List<ApiSceneDetail>> =
        client.request {
            path = "/api/scenes/by-slug/$slug"
        }

    fun updateSceneMetadata(
        client: HttpClient,
        id: String,
        name: String?,
        description: String?,
    ): Result<ApiScene> =
        client.request {
            method = Method.PUT
            path = "/api/scenes/$id"
            jsonBody(
                UpdateSceneMetadataRequest.serializer(),
                UpdateSceneMetadataRequest(name = name, description = description),
            )
            onStatus(404) { ApiError.HttpError(404, "Scene not found") }
        }

    fun deleteScene(
        client: HttpClient,
        slug: String,
    ): Result<Unit> =
        client.requestUnit {
            method = Method.DELETE
            path = "/api/scenes/by-slug/$slug"
            expectedStatus = setOf(204)
            onStatus(404) { ApiError.HttpError(404, "Scene not found") }
        }

    fun createPreset(
        client: HttpClient,
        sceneSlug: String,
        request: CreatePresetRequest,
    ): Result<ApiPreset> =
        client.request {
            method = Method.POST
            path = "/api/scenes/by-slug/$sceneSlug/presets"
            jsonBody(CreatePresetRequest.serializer(), request)
            expectedStatus = setOf(201)
            onStatus(404) { ApiError.HttpError(404, "Scene not found") }
            onStatus(409) { ApiError.HttpError(409, "Preset already exists") }
        }

    fun updatePreset(
        client: HttpClient,
        sceneSlug: String,
        presetSlug: String,
        request: UpdatePresetRequest,
    ): Result<ApiPreset> =
        client.request {
            method = Method.PUT
            path = "/api/scenes/by-slug/$sceneSlug/presets/$presetSlug"
            jsonBody(UpdatePresetRequest.serializer(), request)
            onStatus(404) { ApiError.HttpError(404, "Preset not found") }
        }

    fun deletePreset(
        client: HttpClient,
        sceneSlug: String,
        presetSlug: String,
    ): Result<Unit> =
        client.requestUnit {
            method = Method.DELETE
            path = "/api/scenes/by-slug/$sceneSlug/presets/$presetSlug"
            expectedStatus = setOf(204)
            onStatus(404) { ApiError.HttpError(404, "Preset not found") }
        }

    fun reorderPresets(
        client: HttpClient,
        sceneSlug: String,
        presetIds: List<String>,
    ): Result<List<ApiPreset>> =
        client.request {
            method = Method.PUT
            path = "/api/scenes/by-slug/$sceneSlug/presets/reorder"
            jsonBody(ReorderPresetsRequest.serializer(), ReorderPresetsRequest(presetIds))
        }

    /** POST /api/scenes/reconcile — compute sync status for local scenes against the server. */
    fun reconcile(
        client: HttpClient,
        localScenes: Map<String, ReconcileLocalScene>,
    ): Result<ReconcileResponse> =
        client.request {
            method = Method.POST
            path = "/api/scenes/reconcile"
            jsonBody(ReconcileRequest.serializer(), ReconcileRequest(localScenes))
        }
}

@Serializable
data class ApiScene(
    val id: String,
    val name: String,
    val slug: String,
    val description: String?,
    val dimension: String,
    val active: Boolean,
    val createdAt: String,
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
    val fov: Int,
    val renderDistance: Int,
    val timeOfDayTicks: Int,
    val weather: String,
    val weatherIntensity: Double,
    val moonPhase: Int?,
    val biome: String?,
    val minecraftVersion: String?,
    val packageUrl: String?,
    val packageHash: String?,
    val packageSizeBytes: Long?,
    val createdAt: String,
)

@Serializable
data class ApiPreset(
    val id: String,
    val sceneId: String,
    val name: String,
    val slug: String,
    val timeOfDayTicks: Int,
    val weather: String,
    val weatherIntensity: Double,
    val moonPhase: Int?,
    val sortOrder: Int,
    val createdAt: String,
    val updatedAt: String,
)

@Serializable
data class ApiTag(
    val id: String,
    val name: String,
    val slug: String,
)

/** GET /api/scenes list item — Scene flattened + version + enrichment fields. */
@Serializable
data class ApiSceneListItem(
    val id: String,
    val name: String,
    val slug: String,
    val description: String?,
    val dimension: String,
    val active: Boolean,
    val createdAt: String,
    val version: ApiSceneVersion,
    val tags: List<ApiTag> = emptyList(),
    val thumbhash: String? = null,
    val captureCount: Long = 0,
)

/** GET /api/scenes/by-slug/{slug} detail — Scene flattened + version + presets. */
@Serializable
data class ApiSceneDetail(
    val id: String,
    val name: String,
    val slug: String,
    val description: String?,
    val dimension: String,
    val active: Boolean,
    val createdAt: String,
    val version: ApiSceneVersion,
    val presets: List<ApiPreset> = emptyList(),
)

@Serializable
data class UpdateSceneMetadataRequest(
    val name: String?,
    val description: String?,
)

@Serializable
data class CreatePresetRequest(
    val name: String,
    val slug: String,
    val timeOfDayTicks: Int,
    val weather: String,
    val weatherIntensity: Double = 0.0,
    val moonPhase: Int? = null,
)

@Serializable
data class UpdatePresetRequest(
    val name: String? = null,
    val timeOfDayTicks: Int? = null,
    val weather: String? = null,
    val weatherIntensity: Double? = null,
    val moonPhase: Int? = null,
)

@Serializable
data class ReorderPresetsRequest(
    val presetIds: List<String>,
)

// Reconcile types

@Serializable
data class ReconcileRequest(
    val scenes: Map<String, ReconcileLocalScene>,
)

@Serializable
data class ReconcileLocalScene(
    val packageHash: String? = null,
)

@Serializable
data class ReconcileResponse(
    val scenes: Map<String, ReconcileSceneStatus>,
)

@Serializable
data class ReconcileSceneStatus(
    val status: SyncStatus,
    val scene: ReconcileSceneInfo? = null,
)

@Serializable
data class ReconcileSceneInfo(
    val id: String,
    val name: String,
    val slug: String,
    val description: String? = null,
    val dimension: String,
    val versionId: String,
    val packageHash: String? = null,
    val packageUrl: String? = null,
)

/** Bidirectional sync status computed by the server. */
@Serializable
enum class SyncStatus {
    /** No server connection — status unknown */
    @SerialName("unknown")
    UNKNOWN,

    /** Local scene exists, server doesn't have it */
    @SerialName("local_only")
    LOCAL_ONLY,

    /** Server scene exists, mod doesn't have it */
    @SerialName("remote_only")
    REMOTE_ONLY,

    /** Both exist, hashes match */
    @SerialName("synced")
    SYNCED,

    /** Both exist, local hash differs from server */
    @SerialName("local_ahead")
    LOCAL_AHEAD,

    /** Both exist, server has a newer version */
    @SerialName("remote_ahead")
    REMOTE_AHEAD,
}
