package com.xevion.glint.capture

import com.xevion.glint.Loggers

/**
 * Overrides the time values fed to Iris shader uniforms during capture.
 *
 * When activated, provides deterministic values for all three Iris system time
 * uniforms: `frameTimeCounter` (cumulative seconds), `frameTime` (per-frame delta),
 * and `frameCounter` (integer frame count). Values start from zero at activation
 * and advance at a fixed 60fps rate.
 *
 * The key insight: Iris's `Timer.frameTimeCounter` is **cumulative** — it sums all
 * frame deltas since game start. Overriding the input to `beginFrame(long)` doesn't
 * help because the accumulated base is already non-deterministic. Instead, we override
 * the **getter methods** that shaders actually read.
 *
 * Mixins:
 * - [com.xevion.glint.mixin.SystemTimeUniformsMixin] intercepts `Timer.getFrameTimeCounter()`
 *   and `Timer.getLastFrameTime()`
 * - [com.xevion.glint.mixin.FrameCounterMixin] intercepts `FrameCounter.getAsInt()`
 *
 * Lifecycle:
 * - [activate] at the start of the pre-capture settle phase (after stabilization)
 * - [advanceFrame] each rendered frame during settle and capture
 * - [deactivate] after capture completes or on session teardown
 */
object CaptureTimeOverride {
    private val log = Loggers.Capture.get()

    private var active = false
    private var frameCount: Int = 0

    /** Seconds per frame at 60fps. */
    private const val SECONDS_PER_FRAME = 1.0f / 60.0f

    /**
     * Begin feeding deterministic time to Iris shader uniforms.
     * Resets the frame counter so all time values start from zero.
     */
    fun activate() {
        frameCount = 0
        active = true
        log.debug("Capture time override activated")
    }

    /**
     * Stop overriding time. Iris resumes using real system time.
     */
    fun deactivate() {
        if (active) {
            log.debug("Capture time override deactivated") {
                "frames_rendered" to frameCount
                "final_frameTimeCounter" to (frameCount * SECONDS_PER_FRAME)
            }
        }
        active = false
        frameCount = 0
    }

    /** Whether the override is currently active. */
    val isActive: Boolean get() = active

    /**
     * Advance the synthetic clock by one frame. Call once per rendered frame
     * during the pre-capture settle and capture phases.
     */
    fun advanceFrame() {
        if (active) frameCount++
    }

    /**
     * Deterministic `frameTimeCounter` value (cumulative seconds from zero).
     * Returns null when inactive — mixin passes through to real value.
     *
     * Called by [com.xevion.glint.mixin.SystemTimeUniformsMixin].
     */
    fun getOverrideFrameTimeCounter(): Float? {
        if (!active) return null
        return frameCount * SECONDS_PER_FRAME
    }

    /**
     * Deterministic `frameTime` value (per-frame delta in seconds).
     * Returns a constant 1/60s when active, null otherwise.
     *
     * Called by [com.xevion.glint.mixin.SystemTimeUniformsMixin].
     */
    fun getOverrideLastFrameTime(): Float? {
        if (!active) return null
        return SECONDS_PER_FRAME
    }

    /**
     * Deterministic `frameCounter` value (integer frame count from zero).
     * Returns null when inactive — mixin passes through to real value.
     *
     * Called by [com.xevion.glint.mixin.FrameCounterMixin].
     */
    fun getOverrideFrameCounter(): Int? {
        if (!active) return null
        return frameCount
    }
}
