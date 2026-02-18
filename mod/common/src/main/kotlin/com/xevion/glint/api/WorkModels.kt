package com.xevion.glint.api

import com.xevion.glint.orchestration.CaptureItemKey
import com.xevion.glint.orchestration.ShaderSpec
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject

/** Shader metadata within a work item. */
@Serializable
data class WorkShader(
    val versionId: String,
    val id: String,
    val slug: String,
    val name: String,
    val version: String,
    val downloadUrl: String? = null,
    val fileHash: String? = null,
    val profileId: String? = null,
    val profileName: String? = null,
)

/** Scene positioning and environment within a work item. */
@Serializable
data class WorkScene(
    val id: String,
    val slug: String,
    val name: String,
    val dimension: String,
    val x: Double,
    val y: Double,
    val z: Double,
    val yaw: Double,
    val pitch: Double,
    val timeOfDayTicks: Int,
    val weather: String,
    val weatherIntensity: Double,
    val moonPhase: Int? = null,
    val biome: String? = null,
    val fov: Int = 70,
    val renderDistance: Int = 16,
    val minecraftVersion: String? = null,
    val versionId: String? = null,
)

/** Preset overrides within a work item. */
@Serializable
data class WorkPreset(
    val id: String,
    val name: String,
    val slug: String,
    val timeOfDayTicks: Int? = null,
    val weather: String? = null,
    val weatherIntensity: Double? = null,
    val moonPhase: Int? = null,
)

/** Scene package download info within a work item. */
@Serializable
data class WorkPackage(
    val url: String,
    val hash: String,
    val sizeBytes: Long? = null,
)

/** A single work item: one (shader_version, scene, preset, profile) tuple to capture. */
@Serializable
data class WorkItem(
    val shader: WorkShader,
    val scene: WorkScene,
    val preset: WorkPreset? = null,
    val scenePackage: WorkPackage? = null,
)

// -- WorkItem extensions: preset-over-scene environment overrides --

/** Effective time of day: preset overrides scene. */
val WorkItem.effectiveTimeOfDayTicks: Int
    get() = preset?.timeOfDayTicks ?: scene.timeOfDayTicks

/** Effective weather: preset overrides scene. */
val WorkItem.effectiveWeather: String
    get() = preset?.weather ?: scene.weather

/** Effective weather intensity: preset overrides scene. */
val WorkItem.effectiveWeatherIntensity: Double
    get() = preset?.weatherIntensity ?: scene.weatherIntensity

/** Effective moon phase: preset overrides scene. */
val WorkItem.effectiveMoonPhase: Int?
    get() = preset?.moonPhase ?: scene.moonPhase

/** Composite key for capture tracking. */
fun WorkItem.captureKey() =
    CaptureItemKey(
        shaderVersionId = shader.versionId,
        sceneId = scene.id,
        profileId = shader.profileId,
        presetId = preset?.id,
    )

/** Build a ShaderSpec from the shader metadata. */
fun WorkShader.toShaderSpec(): ShaderSpec {
    if (slug == "vanilla") return ShaderSpec(filename = null)
    val hash8 = fileHash?.take(8)
    val filename = if (hash8 != null) "$slug-$version-$hash8.zip" else "$slug-$version.zip"
    return ShaderSpec(filename = filename, profile = profileName, profileId = profileId)
}

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
    val profileId: String? = null,
    val presetId: String? = null,
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
    val profileId: String? = null,
    val profileName: String? = null,
    val presetId: String? = null,
    val status: String,
    val captureId: String? = null,
    val errorMessage: String? = null,
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
    val presetId: String? = null,
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
