package com.xevion.glint.api

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject

/** A single work item: one (shader_version, scene, profile) triple to capture. */
@Serializable
data class WorkItem(
    @SerialName("shader_version_id") val shaderVersionId: String,
    @SerialName("shader_id") val shaderId: String,
    @SerialName("shader_slug") val shaderSlug: String,
    @SerialName("shader_name") val shaderName: String,
    val version: String,
    @SerialName("download_url") val downloadUrl: String? = null,
    @SerialName("file_hash") val fileHash: String? = null,
    @SerialName("scene_id") val sceneId: String,
    @SerialName("scene_slug") val sceneSlug: String,
    @SerialName("scene_name") val sceneName: String,
    @SerialName("scene_dimension") val sceneDimension: String,
    @SerialName("scene_x") val sceneX: Double,
    @SerialName("scene_y") val sceneY: Double,
    @SerialName("scene_z") val sceneZ: Double,
    @SerialName("scene_yaw") val sceneYaw: Double,
    @SerialName("scene_pitch") val scenePitch: Double,
    @SerialName("scene_time_of_day_ticks") val sceneTimeOfDayTicks: Int,
    @SerialName("scene_weather") val sceneWeather: String,
    @SerialName("scene_weather_intensity") val sceneWeatherIntensity: Double,
    @SerialName("scene_moon_phase") val sceneMoonPhase: Int? = null,
    @SerialName("scene_biome") val sceneBiome: String? = null,
    @SerialName("world_id") val worldId: String,
    @SerialName("world_slug") val worldSlug: String,
    @SerialName("world_name") val worldName: String,
    @SerialName("world_file_url") val worldFileUrl: String? = null,
    @SerialName("world_file_hash") val worldFileHash: String? = null,
    @SerialName("world_size_bytes") val worldSizeBytes: Long? = null,
    @SerialName("world_version_id") val worldVersionId: String? = null,
    @SerialName("scene_version_id") val sceneVersionId: String? = null,
    val profile: String? = null,
)

/** Request to create a capture run. */
@Serializable
data class CreateRunRequest(
    @SerialName("agent_id") val agentId: String? = null,
    val items: List<CreateRunItemRequest>,
    @SerialName("metadata_json") val metadataJson: JsonObject? = null,
)

/** A single item within a create-run request. */
@Serializable
data class CreateRunItemRequest(
    @SerialName("shader_version_id") val shaderVersionId: String,
    @SerialName("scene_id") val sceneId: String,
    val profile: String? = null,
)

/** Response from creating or completing a capture run. */
@Serializable
data class CaptureRun(
    val id: String,
    @SerialName("agent_id") val agentId: String? = null,
    @SerialName("started_at") val startedAt: String,
    @SerialName("completed_at") val completedAt: String? = null,
    val status: String,
    @SerialName("total_items") val totalItems: Int,
    @SerialName("completed_items") val completedItems: Int,
    @SerialName("failed_items") val failedItems: Int,
    @SerialName("skipped_items") val skippedItems: Int,
    @SerialName("metadata_json") val metadataJson: JsonObject? = null,
)

/** A single item within a capture run. */
@Serializable
data class CaptureRunItem(
    val id: String,
    @SerialName("run_id") val runId: String,
    @SerialName("shader_version_id") val shaderVersionId: String,
    @SerialName("scene_id") val sceneId: String,
    val profile: String? = null,
    val status: String,
    @SerialName("capture_id") val captureId: String? = null,
    @SerialName("error_message") val errorMessage: String? = null,
)

/** Request for a presigned upload URL. */
@Serializable
data class UploadUrlRequest(
    @SerialName("shader_id") val shaderId: String,
    @SerialName("scene_id") val sceneId: String,
)

/** Response with presigned upload URL and capture metadata. */
@Serializable
data class UploadUrlResponse(
    @SerialName("capture_id") val captureId: String,
    @SerialName("r2_key") val r2Key: String,
    @SerialName("presigned_url") val presignedUrl: String,
    @SerialName("image_url") val imageUrl: String,
)

/** Request to mark a run item as completed. */
@Serializable
data class CompleteItemRequest(
    @SerialName("capture_id") val captureId: String,
    @SerialName("image_path") val imagePath: String,
    @SerialName("image_url") val imageUrl: String,
    @SerialName("resolution_width") val resolutionWidth: Int,
    @SerialName("resolution_height") val resolutionHeight: Int,
    @SerialName("captured_at") val capturedAt: String,
    @SerialName("duration_ms") val durationMs: Int? = null,
)

/** Request to mark a run item as failed. */
@Serializable
data class FailItemRequest(
    @SerialName("error_message") val errorMessage: String,
    @SerialName("error_log") val errorLog: String? = null,
    @SerialName("duration_ms") val durationMs: Int? = null,
)

/** Request to claim a run item for upload. */
@Serializable
data class ClaimItemRequest(
    @SerialName("resolution_width") val resolutionWidth: Int,
    @SerialName("resolution_height") val resolutionHeight: Int,
    @SerialName("captured_at") val capturedAt: String,
    @SerialName("world_version_id") val worldVersionId: String,
    @SerialName("scene_version_id") val sceneVersionId: String,
)

/** Response from claiming a run item. */
@Serializable
data class ClaimItemResponse(
    @SerialName("capture_id") val captureId: String,
    @SerialName("presigned_url") val presignedUrl: String,
    @SerialName("image_url") val imageUrl: String,
)

/** Request to confirm an upload has completed. */
@Serializable
data class ConfirmUploadRequest(
    @SerialName("image_path") val imagePath: String? = null,
)

/** Request to report a persistent shader failure. */
@Serializable
data class ReportFailureRequest(
    @SerialName("shader_version_id") val shaderVersionId: String,
    @SerialName("error_message") val errorMessage: String,
)
