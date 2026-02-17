package com.xevion.glint.scene

import com.xevion.glint.Loggers
import net.minecraft.server.level.ServerLevel
import java.nio.file.Path

/**
 * Manages scene injection lifecycle. Owns all injection state
 * so both commands and the autonomous runner can drive injection
 * without coupling to the command layer.
 */
object InjectionManager {
    private val log = Loggers.Scene.get()
    private val injector = SceneInjector()
    private var activeProcess: InjectionProcess? = null
    private var activeLevel: ServerLevel? = null
    private var activeLoadedScene: LoadedScene? = null

    val isActive: Boolean
        get() =
            activeProcess != null ||
                injector.getCurrentScene() != null ||
                SceneInjection.getActiveScene() != null

    /**
     * Tick the active injection process. Must be called on the server thread.
     */
    fun tickInjection() {
        val process = activeProcess ?: return
        val level = activeLevel
        try {
            val done = process.tick()
            if (done) {
                if (process.isComplete) {
                    log.info("Scene injection complete (from tick driver)")
                } else {
                    log.error("Scene injection failed") { "error" to (process.error ?: "unknown") }
                    if (level != null) forceDeactivate(level)
                }
                activeProcess = null
            }
        } catch (e: Exception) {
            log.error(e, "Exception during scene injection tick")
            activeProcess = null
            if (level != null) forceDeactivate(level)
        }
    }

    /**
     * Load a scene package from a ZIP file.
     */
    fun load(zipPath: Path): LoadedScene = injector.load(zipPath)

    /**
     * Start injecting a loaded scene into a level.
     */
    fun startInjection(
        scene: LoadedScene,
        level: ServerLevel,
        chunkOffsetX: Int,
        chunkOffsetZ: Int,
        cameraTarget: CameraPosition,
    ) {
        // Deactivate any existing scene first
        forceDeactivate(level)

        val process = injector.inject(scene, level, chunkOffsetX, chunkOffsetZ, cameraTarget)
        activeProcess = process
        activeLevel = level
        activeLoadedScene = scene
    }

    /**
     * Force-cleanup all injection state regardless of what phase we're in.
     * Uses the provided level, or falls back to the stored active level.
     */
    fun forceDeactivate(level: ServerLevel? = null) {
        val resolvedLevel = level ?: activeLevel ?: return
        activeProcess = null
        val pending = activeLoadedScene
        activeLoadedScene = null
        injector.deactivate(resolvedLevel, pending)
        activeLevel = null
    }
}
