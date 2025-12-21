package com.xevion.glint.capture

import net.minecraft.client.CameraType
import net.minecraft.client.CloudStatus
import net.minecraft.client.GraphicsStatus
import net.minecraft.client.Minecraft
import net.minecraft.server.level.ParticleStatus

/**
 * Tracks and restores client state during capture sessions.
 *
 * Saves original settings before capture begins and restores them when complete.
 * This ensures the game returns to the user's preferred state after automated capture.
 */
object CaptureState {
    @Volatile private var isCapturing = false

    data class SavedState(
        val fov: Int,
        val viewBobbing: Boolean,
        val renderDistance: Int,
        val simulationDistance: Int,
        val entityDistanceScaling: Double,
        val graphicsMode: GraphicsStatus,
        val particles: ParticleStatus,
        val clouds: CloudStatus,
        val ambientOcclusion: Boolean,
        val entityShadows: Boolean,
        val biomeBlend: Int,
        val brightness: Double,
        val guiScale: Int,
        val hideGui: Boolean,
        val screenEffects: Double,
        val fovEffects: Double,
        val mipmapLevels: Int,
        val cameraType: CameraType,
    )

    fun saveState(): SavedState {
        val mc = Minecraft.getInstance()
        val options = mc.options

        return SavedState(
            fov = options.fov().get(),
            viewBobbing = options.bobView().get(),
            renderDistance = options.renderDistance().get(),
            simulationDistance = options.simulationDistance().get(),
            entityDistanceScaling = options.entityDistanceScaling().get(),
            graphicsMode = options.graphicsMode().get(),
            particles = options.particles().get(),
            clouds = options.cloudStatus().get(),
            ambientOcclusion = options.ambientOcclusion().get(),
            entityShadows = options.entityShadows().get(),
            biomeBlend = options.biomeBlendRadius().get(),
            brightness = options.gamma().get(),
            guiScale = options.guiScale().get(),
            hideGui = options.hideGui,
            screenEffects = options.screenEffectScale().get(),
            fovEffects = options.fovEffectScale().get(),
            mipmapLevels = options.mipmapLevels().get(),
            cameraType = options.getCameraType(),
        )
    }

    fun restoreState(state: SavedState) {
        val mc = Minecraft.getInstance()
        val options = mc.options

        options.fov().set(state.fov)
        options.bobView().set(state.viewBobbing)
        options.renderDistance().set(state.renderDistance)
        options.simulationDistance().set(state.simulationDistance)
        options.entityDistanceScaling().set(state.entityDistanceScaling)
        options.graphicsMode().set(state.graphicsMode)
        options.particles().set(state.particles)
        options.cloudStatus().set(state.clouds)
        options.ambientOcclusion().set(state.ambientOcclusion)
        options.entityShadows().set(state.entityShadows)
        options.biomeBlendRadius().set(state.biomeBlend)
        options.gamma().set(state.brightness)
        options.guiScale().set(state.guiScale)
        options.hideGui = state.hideGui
        options.screenEffectScale().set(state.screenEffects)
        options.fovEffectScale().set(state.fovEffects)
        options.mipmapLevels().set(state.mipmapLevels)
        options.setCameraType(state.cameraType)
    }

    fun startCapture() {
        isCapturing = true
    }

    fun endCapture() {
        isCapturing = false
    }

    fun isActive(): Boolean = isCapturing
}
