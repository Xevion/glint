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
 * during the same tick's render pass when `update()` is called. Callers must provide their own
 * timing buffer (e.g. the [CaptureSession] `WaitingForRebuild` state) to avoid checking on
 * the same tick as `allChanged()`.
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
        } catch (_: ClassNotFoundException) {
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
            val swrClass = checkNotNull(sodiumWorldRendererClass) { "Sodium class not loaded" }

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

    /** Resolves the RenderSectionManager from the active SodiumWorldRenderer, or null. */
    private fun getSectionManager(): Any? {
        val renderer = instanceNullableMethod!!.invoke(null) ?: return null
        return renderSectionManagerField!!.get(renderer)
    }

    /** Resolves the ChunkBuilder from the active RenderSectionManager, or null. */
    private fun getBuilder(): Any? {
        val sectionManager = getSectionManager() ?: return null
        return getBuilderMethod!!.invoke(sectionManager)
    }

    /**
     * Check if Sodium's chunk rendering is complete.
     *
     * Returns true when the render graph is up-to-date and the build queue is empty.
     * Returns null if Sodium is not available or reflection fails.
     *
     * Callers must provide their own timing buffer (e.g. the [CaptureSession.State.WaitingForRebuild]
     * state) to avoid checking on the same tick as `allChanged()`.
     */
    fun isRenderingComplete(): Boolean? {
        if (!isAvailable()) return null

        try {
            val sectionManager = getSectionManager() ?: return null
            val needsUpdate = needsUpdateMethod!!.invoke(sectionManager) as Boolean
            if (needsUpdate) return false

            val builder = getBuilderMethod!!.invoke(sectionManager) ?: return null
            return isBuildQueueEmptyMethod!!.invoke(builder) as Boolean
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
        return try {
            getScheduledJobCountMethod!!.invoke(getBuilder() ?: return null) as Int
        } catch (e: Exception) {
            log.debug("Failed to get Sodium scheduled job count") { "error" to e.message }
            null
        }
    }

    /**
     * Check if Sodium's render graph needs updating.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun needsGraphUpdate(): Boolean? {
        if (!isAvailable()) return null
        return try {
            needsUpdateMethod!!.invoke(getSectionManager() ?: return null) as Boolean
        } catch (e: Exception) {
            log.debug("Failed to check Sodium graph update state") { "error" to e.message }
            null
        }
    }

    /**
     * Get the number of busy (actively working) chunk builder threads.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun getBusyThreadCount(): Int? {
        if (!isAvailable()) return null
        return try {
            getBusyThreadCountMethod!!.invoke(getBuilder() ?: return null) as Int
        } catch (e: Exception) {
            log.debug("Failed to get Sodium busy thread count") { "error" to e.message }
            null
        }
    }

    /**
     * Get the total number of chunk builder threads.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun getTotalThreadCount(): Int? {
        if (!isAvailable()) return null
        return try {
            getTotalThreadCountMethod!!.invoke(getBuilder() ?: return null) as Int
        } catch (e: Exception) {
            log.debug("Failed to get Sodium total thread count") { "error" to e.message }
            null
        }
    }

    /**
     * Get the total number of render sections from Sodium's RenderSectionManager.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun getTotalSections(): Int? {
        if (!isAvailable()) return null
        return try {
            getTotalSectionsMethod!!.invoke(getSectionManager() ?: return null) as Int
        } catch (e: Exception) {
            log.debug("Failed to get Sodium section count") { "error" to e.message }
            null
        }
    }

    /**
     * Check if Sodium's chunk build queue is empty.
     * Returns null if Sodium is not available or reflection fails.
     */
    fun isBuildQueueEmpty(): Boolean? {
        if (!isAvailable()) return null
        return try {
            isBuildQueueEmptyMethod!!.invoke(getBuilder() ?: return null) as Boolean
        } catch (e: Exception) {
            log.debug("Failed to check Sodium build queue") { "error" to e.message }
            null
        }
    }
}
