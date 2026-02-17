package com.xevion.glint.scene

import java.util.concurrent.atomic.AtomicReference

/**
 * Atomic snapshot of active scene state.
 * Read as a single unit from the mixin to avoid TOCTOU races.
 */
data class ActiveScene(
    val provider: SceneChunkProvider,
    val chunkOffsetX: Int,
    val chunkOffsetZ: Int,
)

/**
 * Global state for active scene injection.
 * Accessed from ChunkStorage mixin (Java) — keep the API simple.
 *
 * Chunk offset fields translate between world coordinates (where the scene is injected)
 * and scene coordinates (where the scene data lives in region files).
 * World pos = scene pos + offset. Scene pos = world pos - offset.
 */
object SceneInjection {
    private val activeScene = AtomicReference<ActiveScene?>(null)

    @Volatile
    var entityTickFrozen: Boolean = false
        private set

    fun activate(
        provider: SceneChunkProvider,
        offsetX: Int,
        offsetZ: Int,
    ) {
        activeScene.set(ActiveScene(provider, offsetX, offsetZ))
    }

    fun deactivate() {
        activeScene.set(null)
    }

    /** Atomic snapshot of provider + offsets for mixin use. */
    fun getActiveScene(): ActiveScene? = activeScene.get()

    fun freezeEntityTick() {
        entityTickFrozen = true
    }

    fun unfreezeEntityTick() {
        entityTickFrozen = false
    }
}
