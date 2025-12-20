package com.xevion.glint.screenshot

import kotlinx.serialization.Serializable

@Serializable
data class ScreenshotMetadata(
    val timestamp: String,
    val screenshot: ScreenshotInfo,
    val minecraft: MinecraftInfo,
    val shader: ShaderInfo? = null,
)

@Serializable
data class ScreenshotInfo(
    val file: String,
    val width: Int,
    val height: Int,
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

@Serializable
data class ShaderInfo(
    val pack: String,
    val enabled: Boolean,
)
