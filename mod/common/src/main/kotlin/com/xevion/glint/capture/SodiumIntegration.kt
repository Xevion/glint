package com.xevion.glint.capture

import com.xevion.glint.Glint
import net.minecraft.client.Minecraft

/**
 * Integration with Sodium's chunk rendering system.
 *
 * Sodium replaces vanilla's chunk building with its own RenderSectionManager and ChunkBuilder.
 * To properly detect when rendering is complete, we need to check:
 * 1. The render graph doesn't need updating (sections have been scheduled)
 * 2. The build queue is empty (all scheduled builds are complete)
 *
 * IMPORTANT: When `LevelRenderer.allChanged()` is called, Sodium destroys and recreates its
 * RenderSectionManager. The `needsUpdate` flag is set to true initially but gets cleared
 * during the same tick's render pass when `update()` is called. This creates a race condition
 * where we might check the state after Sodium has already processed the graph update but before
 * chunk builds are scheduled.
 *
 * To handle this, we track whether we've seen any rebuild activity (needsUpdate=true OR
 * scheduled jobs > 0) since the last reset. This ensures we don't prematurely consider
 * rendering complete on the same tick that allChanged() was called.
 */
object SodiumIntegration {
    private enum class AvailabilityState {
        UNKNOWN,
        AVAILABLE,
        UNAVAILABLE,
    }

    private var availability: AvailabilityState = AvailabilityState.UNKNOWN
    private var sodiumWorldRendererClass: Class<*>? = null
    private var renderSectionManagerClass: Class<*>? = null
    private var chunkBuilderClass: Class<*>? = null

    private var instanceNullableMethod: java.lang.reflect.Method? = null
    private var renderSectionManagerField: java.lang.reflect.Field? = null
    private var needsUpdateMethod: java.lang.reflect.Method? = null
    private var getBuilderMethod: java.lang.reflect.Method? = null
    private var isBuildQueueEmptyMethod: java.lang.reflect.Method? = null
    private var getScheduledJobCountMethod: java.lang.reflect.Method? = null
    private var getBusyThreadCountMethod: java.lang.reflect.Method? = null
    private var getTotalThreadCountMethod: java.lang.reflect.Method? = null

    private var sawRebuildActivity: Boolean = false

    fun isAvailable(): Boolean {
        if (availability == AvailabilityState.UNKNOWN) {
            availability =
                try {
                    sodiumWorldRendererClass = Class.forName("net.caffeinemc.mods.sodium.client.render.SodiumWorldRenderer")
                    AvailabilityState.AVAILABLE
                } catch (e: ClassNotFoundException) {
                    Glint.LOGGER.debug("Sodium not detected, using vanilla chunk rendering detection")
                    AvailabilityState.UNAVAILABLE
                }
        }
        return availability == AvailabilityState.AVAILABLE
    }

    private fun initReflection(): Boolean {
        if (instanceNullableMethod != null) return true

        try {
            val swrClass = sodiumWorldRendererClass ?: return false

            // Get SodiumWorldRenderer.instanceNullable()
            instanceNullableMethod = swrClass.getMethod("instanceNullable")

            // Get renderSectionManager field (private field, not a method)
            renderSectionManagerField =
                swrClass.getDeclaredField("renderSectionManager").apply {
                    isAccessible = true
                }

            // Get RenderSectionManager class and methods
            renderSectionManagerClass = Class.forName("net.caffeinemc.mods.sodium.client.render.chunk.RenderSectionManager")
            needsUpdateMethod = renderSectionManagerClass!!.getMethod("needsUpdate")
            getBuilderMethod = renderSectionManagerClass!!.getMethod("getBuilder")

            // Get ChunkBuilder class and methods
            chunkBuilderClass = Class.forName("net.caffeinemc.mods.sodium.client.render.chunk.compile.executor.ChunkBuilder")
            getScheduledJobCountMethod = chunkBuilderClass!!.getMethod("getScheduledJobCount")
            getBusyThreadCountMethod = chunkBuilderClass!!.getMethod("getBusyThreadCount")
            getTotalThreadCountMethod = chunkBuilderClass!!.getMethod("getTotalThreadCount")
            isBuildQueueEmptyMethod = chunkBuilderClass!!.getMethod("isBuildQueueEmpty")

            Glint.LOGGER.debug("Sodium integration initialized successfully")
            return true
        } catch (e: Exception) {
            Glint.LOGGER.warn("Failed to initialize Sodium reflection: ${e.message}")
            availability = AvailabilityState.UNAVAILABLE
            return false
        }
    }

    fun resetStabilizationState() {
        sawRebuildActivity = false
    }

    /**
     * Check if Sodium's chunk rendering is complete.
     *
     * Returns true only when:
     * 1. We've seen rebuild activity (needsUpdate=true or jobs scheduled) since last reset
     * 2. The render graph doesn't need updating (needsUpdate() == false)
     * 3. The build queue is empty (isBuildQueueEmpty() == true)
     *
     * Returns null if Sodium is not available or reflection fails.
     */
    fun isRenderingComplete(): Boolean? {
        if (!isAvailable()) return null
        if (!initReflection()) return null

        try {
            // Get SodiumWorldRenderer instance
            val renderer = instanceNullableMethod!!.invoke(null) ?: return null

            // Get RenderSectionManager
            val sectionManager = renderSectionManagerField!!.get(renderer) ?: return null

            // Check if render graph needs updating
            val needsUpdate = needsUpdateMethod!!.invoke(sectionManager) as Boolean

            // Get ChunkBuilder and check queue state
            val builder = getBuilderMethod!!.invoke(sectionManager) ?: return null
            val scheduledJobs = getScheduledJobCountMethod!!.invoke(builder) as Int
            val queueEmpty = isBuildQueueEmptyMethod!!.invoke(builder) as Boolean

            // Track if we've seen any rebuild activity
            if (needsUpdate || scheduledJobs > 0 || !queueEmpty) {
                sawRebuildActivity = true
            }

            // Don't consider complete until we've seen rebuild activity
            // This prevents false positives when checking on the same tick as allChanged()
            if (!sawRebuildActivity) {
                return false
            }

            // Now check actual completion: graph updated and queue empty
            if (needsUpdate) {
                return false
            }

            return queueEmpty
        } catch (e: Exception) {
            Glint.LOGGER.debug("Sodium rendering check failed: ${e.message}")
            return null
        }
    }

    /**
     * Get the number of scheduled chunk build jobs.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun getScheduledJobCount(): Int? {
        if (!isAvailable()) return null
        if (!initReflection()) return null

        try {
            val renderer = instanceNullableMethod!!.invoke(null) ?: return null
            val sectionManager = renderSectionManagerField!!.get(renderer) ?: return null
            val builder = getBuilderMethod!!.invoke(sectionManager) ?: return null
            return getScheduledJobCountMethod!!.invoke(builder) as Int
        } catch (e: Exception) {
            Glint.LOGGER.debug("Failed to get Sodium scheduled job count: ${e.message}")
            return null
        }
    }

    /**
     * Check if Sodium's render graph needs updating.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun needsGraphUpdate(): Boolean? {
        if (!isAvailable()) return null
        if (!initReflection()) return null

        try {
            val renderer = instanceNullableMethod!!.invoke(null) ?: return null
            val sectionManager = renderSectionManagerField!!.get(renderer) ?: return null
            return needsUpdateMethod!!.invoke(sectionManager) as Boolean
        } catch (e: Exception) {
            Glint.LOGGER.debug("Failed to check Sodium graph update state: ${e.message}")
            return null
        }
    }

    /**
     * Get the number of busy (actively working) chunk builder threads.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun getBusyThreadCount(): Int? {
        if (!isAvailable()) return null
        if (!initReflection()) return null

        try {
            val renderer = instanceNullableMethod!!.invoke(null) ?: return null
            val sectionManager = renderSectionManagerField!!.get(renderer) ?: return null
            val builder = getBuilderMethod!!.invoke(sectionManager) ?: return null
            return getBusyThreadCountMethod!!.invoke(builder) as Int
        } catch (e: Exception) {
            Glint.LOGGER.debug("Failed to get Sodium busy thread count: ${e.message}")
            return null
        }
    }

    /**
     * Get the total number of chunk builder threads.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun getTotalThreadCount(): Int? {
        if (!isAvailable()) return null
        if (!initReflection()) return null

        try {
            val renderer = instanceNullableMethod!!.invoke(null) ?: return null
            val sectionManager = renderSectionManagerField!!.get(renderer) ?: return null
            val builder = getBuilderMethod!!.invoke(sectionManager) ?: return null
            return getTotalThreadCountMethod!!.invoke(builder) as Int
        } catch (e: Exception) {
            Glint.LOGGER.debug("Failed to get Sodium total thread count: ${e.message}")
            return null
        }
    }
}
