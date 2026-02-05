package com.xevion.glint.capture

import com.xevion.glint.Loggers
import net.minecraft.client.CameraType
import net.minecraft.client.CloudStatus
import net.minecraft.client.GraphicsStatus
import net.minecraft.client.Minecraft
import net.minecraft.server.level.ParticleStatus
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Thread-safe manager for capture session state.
 *
 * Tracks whether a capture session is active and handles cancellation requests.
 * Uses atomic operations to ensure safe access from both game thread and mixin injection points.
 */
object CaptureStateManager {
    private val log = Loggers.Capture.get()
    private val isCapturing = AtomicBoolean(false)
    private val cancelRequested = AtomicBoolean(false)

    fun startCapture() {
        isCapturing.set(true)
        cancelRequested.set(false)
        log.info("Capture started")
    }

    fun endCapture() {
        isCapturing.set(false)
        cancelRequested.set(false)
        log.info("Capture ended")
    }

    fun isActive(): Boolean = isCapturing.get()

    /**
     * Request cancellation of the current capture session.
     * Called from KeyboardHandlerMixin when ESC is pressed.
     */
    fun requestCancel() {
        if (isCapturing.get()) {
            cancelRequested.set(true)
            log.info("Capture cancellation requested")
        }
    }

    /**
     * Check and clear the cancel request flag atomically.
     * Returns true if cancellation was requested.
     */
    fun consumeCancelRequest(): Boolean = cancelRequested.getAndSet(false)
}

/**
 * Immutable snapshot of Minecraft client state for restoration after capture.
 */
data class CaptureStateSnapshot(
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
) {
    companion object {
        /**
         * Captures the current Minecraft client state.
         */
        fun capture(): CaptureStateSnapshot {
            val mc = Minecraft.getInstance()
            val options = mc.options

            return CaptureStateSnapshot(
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
    }

    /**
     * Restores this state snapshot to the Minecraft client.
     */
    fun restore() {
        val mc = Minecraft.getInstance()
        val options = mc.options

        options.fov().set(fov)
        options.bobView().set(viewBobbing)
        options.renderDistance().set(renderDistance)
        options.simulationDistance().set(simulationDistance)
        options.entityDistanceScaling().set(entityDistanceScaling)
        options.graphicsMode().set(graphicsMode)
        options.particles().set(particles)
        options.cloudStatus().set(clouds)
        options.ambientOcclusion().set(ambientOcclusion)
        options.entityShadows().set(entityShadows)
        options.biomeBlendRadius().set(biomeBlend)
        options.gamma().set(brightness)
        options.guiScale().set(guiScale)
        options.hideGui = hideGui
        options.screenEffectScale().set(screenEffects)
        options.fovEffectScale().set(fovEffects)
        options.mipmapLevels().set(mipmapLevels)
        options.setCameraType(cameraType)
    }
}
