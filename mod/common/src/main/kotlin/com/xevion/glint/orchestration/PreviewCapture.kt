package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.WorkItem
import com.xevion.glint.api.WorkPackage
import com.xevion.glint.api.WorkPreset
import com.xevion.glint.api.WorkScene
import com.xevion.glint.api.WorkShader
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
                shader =
                    WorkShader(
                        versionId = "vanilla",
                        id = "vanilla",
                        slug = "vanilla",
                        name = "Vanilla",
                        version = "0",
                    ),
                scene =
                    WorkScene(
                        id = slug,
                        slug = slug,
                        name = metadata.name,
                        dimension = metadata.dimension,
                        x = metadata.camera.x,
                        y = metadata.camera.y,
                        z = metadata.camera.z,
                        yaw = metadata.camera.yaw.toDouble(),
                        pitch = metadata.camera.pitch.toDouble(),
                        timeOfDayTicks = metadata.environment.time,
                        weather = metadata.environment.weather,
                        weatherIntensity = metadata.environment.weatherIntensity.toDouble(),
                        moonPhase = metadata.environment.moonPhase,
                        fov = metadata.fov,
                        renderDistance = metadata.renderDistance,
                        minecraftVersion = metadata.minecraftVersion,
                        versionId = null,
                    ),
                preset =
                    preset?.let {
                        WorkPreset(
                            // Unsynced local presets use "local" as a sentinel ID.
                            // Preview captures don't flow through RunUploader, so
                            // this never reaches the backend API.
                            id = "local",
                            name = it.name,
                            slug = it.slug,
                            timeOfDayTicks = it.timeOfDayTicks,
                            weather = it.weather,
                            weatherIntensity = it.weatherIntensity,
                            moonPhase = it.moonPhase,
                        )
                    },
                scenePackage =
                    WorkPackage(
                        url = "",
                        hash = metadata.packageHash,
                    ),
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
