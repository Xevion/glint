package com.xevion.glint.capture

import com.xevion.glint.Loggers

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
    private val log = Loggers.Capture.get()

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
    private var getTotalSectionsMethod: java.lang.reflect.Method? = null

    @Suppress("unused")
    private var sawRebuildActivity: Boolean = false

    fun isAvailable(): Boolean {
        if (availability == AvailabilityState.UNKNOWN) {
            initializeSodium()
        }
        return availability == AvailabilityState.AVAILABLE
    }

    /**
     * Initialize Sodium integration on first access.
     * Fails loudly if Sodium is detected but reflection setup fails.
     */
    private fun initializeSodium() {
        try {
            sodiumWorldRendererClass = Class.forName("net.caffeinemc.mods.sodium.client.render.SodiumWorldRenderer")
            log.info("Sodium detected, initializing integration")

            // Sodium is present, now try to set up reflection
            initReflection()
        } catch (e: ClassNotFoundException) {
            log.debug("Sodium not detected, using vanilla chunk rendering detection")
            availability = AvailabilityState.UNAVAILABLE
        }
    }

    /**
     * Sets up reflection for Sodium's rendering APIs.
     * Throws exceptions if Sodium is present but API has changed.
     */
    private fun initReflection() {
        if (instanceNullableMethod != null) return

        try {
            val swrClass = sodiumWorldRendererClass ?: throw IllegalStateException("Sodium class not loaded")

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
            getTotalSectionsMethod = renderSectionManagerClass!!.getMethod("getTotalSections")

            log.info("Sodium integration initialized successfully")
            availability = AvailabilityState.AVAILABLE
        } catch (e: NoSuchMethodException) {
            availability = AvailabilityState.UNAVAILABLE
            throw IllegalStateException(
                "Sodium is installed but its API has changed. " +
                    "Method not found: ${e.message}. " +
                    "This version of Glint may not be compatible with your Sodium version.",
                e,
            )
        } catch (e: NoSuchFieldException) {
            availability = AvailabilityState.UNAVAILABLE
            throw IllegalStateException(
                "Sodium is installed but its API has changed. " +
                    "Field not found: ${e.message}. " +
                    "This version of Glint may not be compatible with your Sodium version.",
                e,
            )
        } catch (e: ClassNotFoundException) {
            availability = AvailabilityState.UNAVAILABLE
            throw IllegalStateException(
                "Sodium is installed but its internal classes have changed. " +
                    "Class not found: ${e.message}. " +
                    "This version of Glint may not be compatible with your Sodium version.",
                e,
            )
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
            log.debug("Sodium rendering check failed") { "error" to e.message }
            return null
        }
    }

    /**
     * Get the number of scheduled chunk build jobs.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun getScheduledJobCount(): Int? {
        if (!isAvailable()) return null

        try {
            val renderer = instanceNullableMethod!!.invoke(null) ?: return null
            val sectionManager = renderSectionManagerField!!.get(renderer) ?: return null
            val builder = getBuilderMethod!!.invoke(sectionManager) ?: return null
            return getScheduledJobCountMethod!!.invoke(builder) as Int
        } catch (e: Exception) {
            log.debug("Failed to get Sodium scheduled job count") { "error" to e.message }
            return null
        }
    }

    /**
     * Check if Sodium's render graph needs updating.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun needsGraphUpdate(): Boolean? {
        if (!isAvailable()) return null

        try {
            val renderer = instanceNullableMethod!!.invoke(null) ?: return null
            val sectionManager = renderSectionManagerField!!.get(renderer) ?: return null
            return needsUpdateMethod!!.invoke(sectionManager) as Boolean
        } catch (e: Exception) {
            log.debug("Failed to check Sodium graph update state") { "error" to e.message }
            return null
        }
    }

    /**
     * Get the number of busy (actively working) chunk builder threads.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun getBusyThreadCount(): Int? {
        if (!isAvailable()) return null

        try {
            val renderer = instanceNullableMethod!!.invoke(null) ?: return null
            val sectionManager = renderSectionManagerField!!.get(renderer) ?: return null
            val builder = getBuilderMethod!!.invoke(sectionManager) ?: return null
            return getBusyThreadCountMethod!!.invoke(builder) as Int
        } catch (e: Exception) {
            log.debug("Failed to get Sodium busy thread count") { "error" to e.message }
            return null
        }
    }

    /**
     * Get the total number of chunk builder threads.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun getTotalThreadCount(): Int? {
        if (!isAvailable()) return null

        try {
            val renderer = instanceNullableMethod!!.invoke(null) ?: return null
            val sectionManager = renderSectionManagerField!!.get(renderer) ?: return null
            val builder = getBuilderMethod!!.invoke(sectionManager) ?: return null
            return getTotalThreadCountMethod!!.invoke(builder) as Int
        } catch (e: Exception) {
            log.debug("Failed to get Sodium total thread count") { "error" to e.message }
            return null
        }
    }

    /**
     * Get the total number of render sections from Sodium's RenderSectionManager.
     * This represents the total sections Sodium is managing for rendering.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun getTotalSections(): Int? {
        if (!isAvailable()) return null

        try {
            val renderer = instanceNullableMethod!!.invoke(null) ?: return null
            val sectionManager = renderSectionManagerField!!.get(renderer) ?: return null
            return getTotalSectionsMethod!!.invoke(sectionManager) as Int
        } catch (e: Exception) {
            log.debug("Failed to get Sodium section count") { "error" to e.message }
            return null
        }
    }

    /**
     * Check if Sodium's chunk build queue is empty.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun isBuildQueueEmpty(): Boolean? {
        if (!isAvailable()) return null

        try {
            val renderer = instanceNullableMethod!!.invoke(null) ?: return null
            val sectionManager = renderSectionManagerField!!.get(renderer) ?: return null
            val builder = getBuilderMethod!!.invoke(sectionManager) ?: return null
            return isBuildQueueEmptyMethod!!.invoke(builder) as Boolean
        } catch (e: Exception) {
            log.debug("Failed to check Sodium build queue") { "error" to e.message }
            return null
        }
    }
}
