package com.xevion.glint.screenshot

import kotlinx.serialization.Serializable

/**
 * Session manifest containing metadata for all screenshots in a capture session.
 * Written to `session.json` in the session directory.
 */
@Serializable
data class SessionManifest(
    val session: SessionInfo,
    val minecraft: MinecraftInfo,
    val screenshots: List<ScreenshotEntry>,
)

@Serializable
data class SessionInfo(
    val id: String,
    val sceneId: String,
    val startedAt: String,
    val completedAt: String,
    val totalScreenshots: Int,
    val shaderPacks: List<String>,
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
)

@Serializable
data class Resolution(
    val width: Int,
    val height: Int,
)
