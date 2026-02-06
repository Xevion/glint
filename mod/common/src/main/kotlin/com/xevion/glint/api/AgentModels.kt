package com.xevion.glint.api

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** Full job payload returned when claiming a job from the backend. */
@Serializable
data class JobPayload(
    val id: String,
    val shader: JobShaderInfo,
    val scenes: List<JobSceneInfo>,
    val worlds: List<JobWorldInfo>,
    val profiles: List<String>,
    val priority: Int,
)

/** Shader information within a job payload. */
@Serializable
data class JobShaderInfo(
    val id: String,
    val slug: String,
    val name: String,
    @SerialName("version_id") val versionId: String,
    val version: String,
    @SerialName("download_url") val downloadUrl: String? = null,
    @SerialName("file_hash") val fileHash: String? = null,
)

/** Scene information within a job payload. */
@Serializable
data class JobSceneInfo(
    val id: String,
    val slug: String,
    val name: String,
    @SerialName("world_id") val worldId: String,
    @SerialName("definition_json") val definitionJson: String,
)

/** World information within a job payload. */
@Serializable
data class JobWorldInfo(
    val id: String,
    val slug: String,
    val name: String,
    @SerialName("file_url") val fileUrl: String? = null,
    @SerialName("file_hash") val fileHash: String? = null,
    @SerialName("size_bytes") val sizeBytes: Long? = null,
)

/** A file entry in a prepare-upload request, with metadata for R2 key generation. */
@Serializable
data class PrepareUploadFile(
    @SerialName("local_path") val localPath: String,
    @SerialName("scene_id") val sceneId: String,
    val profile: String? = null,
)

/** Request to prepare pre-signed upload URLs. */
@Serializable
data class PrepareUploadRequest(
    val files: List<PrepareUploadFile>,
)

/** A single upload target returned from prepare-upload. */
@Serializable
data class UploadTarget(
    @SerialName("presigned_url") val presignedUrl: String,
    @SerialName("r2_key") val r2Key: String,
    @SerialName("capture_id") val captureId: String,
)

/** Response containing pre-signed upload URLs and capture metadata. */
@Serializable
data class PrepareUploadResponse(
    val uploads: Map<String, UploadTarget>,
)

/** Request to mark a job as complete. */
@Serializable
data class CompleteJobRequest(
    val captures: List<CaptureRecord>,
    @SerialName("discovered_profiles") val discoveredProfiles: List<String>? = null,
)

/** A single capture result within a completion request. */
@Serializable
data class CaptureRecord(
    @SerialName("capture_id") val captureId: String,
    @SerialName("scene_id") val sceneId: String,
    val profile: String? = null,
    @SerialName("screenshot_path") val screenshotPath: String,
    @SerialName("resolution_width") val resolutionWidth: Int,
    @SerialName("resolution_height") val resolutionHeight: Int,
    @SerialName("captured_at") val capturedAt: String,
)

/** Request to mark a job as failed. */
@Serializable
data class FailJobRequest(
    @SerialName("error_message") val errorMessage: String,
)

/** Request to force-create a job for testing. */
@Serializable
data class ForceJobRequest(
    val scenes: String? = null,
    val shaders: String? = null,
)
