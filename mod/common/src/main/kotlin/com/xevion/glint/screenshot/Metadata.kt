package com.xevion.glint.screenshot

import kotlinx.serialization.Serializable

/**
 * Capture session data returned by CaptureSession for aggregation into master manifest.
 * Contains all metadata about a single scene capture (all shaders for one scene).
 */
@Serializable
data class CaptureSessionData(
    val worldName: String,
    val sceneId: String,
    val sessionDir: String,
    val startedAt: String,
    val completedAt: String,
    val totalScreenshots: Int,
    val shaderPacks: List<String>,
    val minecraft: MinecraftInfo,
    val screenshots: List<ScreenshotEntry>,
)

@Serializable
data class MinecraftInfo(
    val version: String,
    val dimension: String? = null,
    val position: Position? = null,
    val camera: Camera? = null,
)

@Serializable
data class Position(
    val x: Double,
    val y: Double,
    val z: Double,
)

@Serializable
data class Camera(
    val yaw: Float,
    val pitch: Float,
)

/**
 * Metadata for a single screenshot within a session.
 */
@Serializable
data class ScreenshotEntry(
    val file: String,
    val timestamp: String,
    val shader: ShaderMetadata?,
    val resolution: Resolution,
)

@Serializable
data class ShaderMetadata(
    val packFile: String,
    val id: String,
    val version: String,
    val profile: String? = null,
)

@Serializable
data class Resolution(
    val width: Int,
    val height: Int,
)
