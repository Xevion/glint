package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.WorkItem
import com.xevion.glint.scene.LocalSceneMetadata
import com.xevion.glint.scene.LocalSceneStore
import com.xevion.glint.session.SessionRegistry
import java.util.UUID

/**
 * Starts a quick preview capture for a scene using the injection pipeline.
 *
 * Builds synthetic [WorkItem]s from [LocalSceneMetadata] with vanilla shader
 * and default preset, then delegates to [LinearOrchestrator] via [SessionRegistry].
 */
object PreviewCapture {
    private val log = Loggers.Orchestration.get()

    /**
     * Starts a preview capture for a single scene.
     *
     * @param slug Scene slug (key in LocalSceneStore)
     * @param presetSlug Optional preset to use (null = scene defaults)
     * @return true if capture started, false if failed or already running
     */
    fun start(
        slug: String,
        presetSlug: String? = null,
    ): Boolean {
        val metadata = LocalSceneStore.loadMetadata(slug)
        if (metadata == null) {
            log.error("Cannot preview: no metadata for scene") { "slug" to slug }
            return false
        }

        val packageFile = LocalSceneStore.packagePath(slug)
        if (!packageFile.exists()) {
            log.error("Cannot preview: package not found") {
                "slug" to slug
                "path" to packageFile.absolutePath
            }
            return false
        }

        val preset =
            if (presetSlug != null) {
                metadata.presets.find { it.slug == presetSlug }
            } else {
                null
            }

        val runId = "preview-${UUID.randomUUID().toString().take(8)}"

        val item =
            WorkItem(
                // Shader: vanilla
                shaderVersionId = "vanilla",
                shaderId = "vanilla",
                shaderSlug = "vanilla",
                shaderName = "Vanilla",
                version = "0",
                // Scene: from metadata
                sceneId = slug,
                sceneSlug = slug,
                sceneName = metadata.name,
                sceneDimension = metadata.dimension,
                sceneX = metadata.camera.x,
                sceneY = metadata.camera.y,
                sceneZ = metadata.camera.z,
                sceneYaw = metadata.camera.yaw.toDouble(),
                scenePitch = metadata.camera.pitch.toDouble(),
                sceneTimeOfDayTicks = preset?.timeOfDayTicks ?: metadata.environment.time,
                sceneWeather = preset?.weather ?: metadata.environment.weather,
                sceneWeatherIntensity = preset?.weatherIntensity ?: metadata.environment.weatherIntensity.toDouble(),
                sceneMoonPhase = preset?.moonPhase ?: metadata.environment.moonPhase,
                sceneFov = metadata.fov,
                sceneRenderDistance = metadata.renderDistance,
                sceneMinecraftVersion = metadata.minecraftVersion,
                // Package
                packageHash = metadata.packageHash,
                // Preset (if specified)
                presetId = preset?.backendPresetId,
                presetName = preset?.name,
                presetSlug = preset?.slug,
                presetTimeOfDayTicks = preset?.timeOfDayTicks,
                presetWeather = preset?.weather,
                presetWeatherIntensity = preset?.weatherIntensity,
                presetMoonPhase = preset?.moonPhase,
                // Scene version
                sceneVersionId = metadata.versions.firstOrNull()?.backendVersionId,
            )

        val scenePackages = mapOf(metadata.packageHash to packageFile)

        log.info("Starting preview capture") {
            "slug" to slug
            "preset" to (preset?.name ?: "default")
            "run_id" to runId
        }

        return SessionRegistry.startLinearOrchestration(
            items = listOf(item),
            runId = runId,
            scenePackages = scenePackages,
        )
    }
}
