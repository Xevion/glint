package com.xevion.glint.capture

import com.xevion.glint.Loggers
import java.util.function.Consumer
import java.util.function.Function

/**
 * Integration with the FREX Flawless Frames API.
 *
 * When enabled, tells rendering mods (Sodium, Canvas, Bobby) to prioritize visual
 * correctness over performance — ensuring all chunks load at full LOD, all builds
 * complete before rendering, etc.
 *
 * The toggle is provided by Sodium at startup via platform-specific entrypoints
 * (Fabric: `frex_flawless_frames`, NeoForge: `frex:flawless_frames_handler`).
 * If Sodium doesn't provide a toggle (old version, not installed), calls to
 * [setEnabled] are no-ops with a warning.
 */
object FlawlessFrames {
    private val log = Loggers.Capture.get()
    private var toggle: Consumer<Boolean>? = null
    private var warnedMissing = false

    /**
     * Called by platform-specific entrypoints when Sodium provides its Flawless Frames controller.
     *
     * @param provider FREX provider function: takes a mod ID string, returns a boolean consumer
     */
    fun registerToggle(provider: Function<String, Consumer<Boolean>>) {
        toggle = provider.apply("glint")
        log.info("Flawless Frames registered")
    }

    /** Whether a Flawless Frames toggle was registered by a rendering mod. */
    val isAvailable: Boolean get() = toggle != null

    /**
     * Enable or disable Flawless Frames mode.
     *
     * Called at capture session start/end. When enabled, Sodium will wait for all chunks
     * to be fully built before rendering each frame, eliminating LOD and deferral artifacts.
     */
    fun setEnabled(enabled: Boolean) {
        val t = toggle
        if (t != null) {
            t.accept(enabled)
            log.debug("Flawless Frames") { "enabled" to enabled }
        } else if (enabled && !warnedMissing) {
            warnedMissing = true
            log.warn(
                "Flawless Frames not available — Sodium may use reduced LOD during capture. " +
                    "Install a Sodium version with FREX support for best results.",
            )
        }
    }
}
