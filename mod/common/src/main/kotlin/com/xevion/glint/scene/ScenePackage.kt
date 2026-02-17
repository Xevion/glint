package com.xevion.glint.scene

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

/** Shared Json instance for scene package serialization (meta.json round-trip). */
internal val scenePackageJson =
    Json {
        prettyPrint = true
        encodeDefaults = true
        ignoreUnknownKeys = true
    }

/**
 * Schema for meta.json inside a scene package ZIP.
 * Only custom format in the package — everything else is Minecraft's own formats.
 */
@Serializable
data class ScenePackageMeta(
    @SerialName("format_version") val formatVersion: Int = 1,
    @SerialName("minecraft_version") val minecraftVersion: String,
    val dimension: String,
    val camera: CameraPosition,
    val fov: Int = 70,
    @SerialName("render_distance") val renderDistance: Int = 12,
    val environment: PackageEnvironment,
    @SerialName("chunk_bounds") val chunkBounds: ChunkBounds,
)

@Serializable
data class CameraPosition(
    val x: Double,
    val y: Double,
    val z: Double,
    val yaw: Float,
    val pitch: Float,
)

@Serializable
data class PackageEnvironment(
    val time: Int,
    val weather: String = "clear",
    @SerialName("weather_intensity") val weatherIntensity: Float = 0f,
    @SerialName("moon_phase") val moonPhase: Int = 0,
)

@Serializable
data class ChunkBounds(
    val min: List<Int>, // [chunkX, chunkZ]
    val max: List<Int>, // [chunkX, chunkZ]
) {
    init {
        require(min.size == 2) { "ChunkBounds.min must be [chunkX, chunkZ], got size ${min.size}" }
        require(max.size == 2) { "ChunkBounds.max must be [chunkX, chunkZ], got size ${max.size}" }
        require(min[0] <= max[0] && min[1] <= max[1]) { "ChunkBounds.min must be <= max: min=$min, max=$max" }
    }

    val minChunkX: Int get() = min[0]
    val minChunkZ: Int get() = min[1]
    val maxChunkX: Int get() = max[0]
    val maxChunkZ: Int get() = max[1]

    fun contains(
        chunkX: Int,
        chunkZ: Int,
    ): Boolean = chunkX in minChunkX..maxChunkX && chunkZ in minChunkZ..maxChunkZ
}
