package com.xevion.glint.api

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject

/** A single work item: one (shader_version, scene, profile) triple to capture. */
@Serializable
data class WorkItem(
    val shaderVersionId: String,
    val shaderId: String,
    val shaderSlug: String,
    val shaderName: String,
    val version: String,
    val downloadUrl: String? = null,
    val fileHash: String? = null,
    val sceneId: String,
    val sceneSlug: String,
    val sceneName: String,
    val sceneDimension: String,
    val sceneX: Double,
    val sceneY: Double,
    val sceneZ: Double,
    val sceneYaw: Double,
    val scenePitch: Double,
    val sceneTimeOfDayTicks: Int,
    val sceneWeather: String,
    val sceneWeatherIntensity: Double,
    val sceneMoonPhase: Int? = null,
    val sceneBiome: String? = null,
    val worldId: String,
    val worldSlug: String,
    val worldName: String,
    val worldFileUrl: String? = null,
    val worldFileHash: String? = null,
    val worldSizeBytes: Long? = null,
    val worldVersionId: String? = null,
    val sceneVersionId: String? = null,
    val profile: String? = null,
)

/** Request to create a capture run. */
@Serializable
data class CreateRunRequest(
    val agentId: String? = null,
    val items: List<CreateRunItemRequest>,
    val metadataJson: JsonObject? = null,
)

/** A single item within a create-run request. */
@Serializable
data class CreateRunItemRequest(
    val shaderVersionId: String,
    val sceneId: String,
    val profile: String? = null,
)

/** Response from creating or completing a capture run. */
@Serializable
data class CaptureRun(
    val id: String,
    val agentId: String? = null,
    val startedAt: String,
    val completedAt: String? = null,
    val status: String,
    val totalItems: Int,
    val completedItems: Int,
    val failedItems: Int,
    val skippedItems: Int,
    val metadataJson: JsonObject? = null,
)

/** A single item within a capture run. */
@Serializable
data class CaptureRunItem(
    val id: String,
    val runId: String,
    val shaderVersionId: String,
    val sceneId: String,
    val profile: String? = null,
    val status: String,
    val captureId: String? = null,
    val errorMessage: String? = null,
)

/** Request for a presigned upload URL. */
@Serializable
data class UploadUrlRequest(
    val shaderId: String,
    val sceneId: String,
)

/** Response with presigned upload URL and capture metadata. */
@Serializable
data class UploadUrlResponse(
    val captureId: String,
    val r2Key: String,
    val presignedUrl: String,
    val imageUrl: String,
)

/** Request to mark a run item as completed. */
@Serializable
data class CompleteItemRequest(
    val captureId: String,
    val imagePath: String,
    val imageUrl: String,
    val resolutionWidth: Int,
    val resolutionHeight: Int,
    val capturedAt: String,
    val durationMs: Int? = null,
)

/** Request to mark a run item as failed. */
@Serializable
data class FailItemRequest(
    val errorMessage: String,
    val errorLog: String? = null,
    val durationMs: Int? = null,
)

/** Request to claim a run item for upload. */
@Serializable
data class ClaimItemRequest(
    val resolutionWidth: Int,
    val resolutionHeight: Int,
    val capturedAt: String,
    val worldVersionId: String,
    val sceneVersionId: String,
)

/** Response from claiming a run item. */
@Serializable
data class ClaimItemResponse(
    val captureId: String,
    val presignedUrl: String,
    val imageUrl: String,
)

/** Request to confirm an upload has completed. */
@Serializable
data class ConfirmUploadRequest(
    val imagePath: String? = null,
)

/** Request to report a persistent shader failure. */
@Serializable
data class ReportFailureRequest(
    val shaderVersionId: String,
    val errorMessage: String,
)
