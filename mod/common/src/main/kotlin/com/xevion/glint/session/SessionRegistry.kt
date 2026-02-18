package com.xevion.glint.session

import com.xevion.glint.Loggers
import com.xevion.glint.api.WorkItem
import com.xevion.glint.orchestration.LinearOrchestrator
import java.io.File

/**
 * Manages lifecycle of the orchestrator session globally.
 *
 * Only one orchestration session may be active at a time.
 */
object SessionRegistry {
    private val manager = SessionManager<LinearOrchestrator>()

    /**
     * Starts linear orchestration with pre-ordered work items.
     * Uses [LinearOrchestrator] with scene package injection.
     *
     * @param items Pre-sorted work items from the backend
     * @param runId Capture run ID
     * @param scenePackages Map of package hash → local ZIP file
     * @param outputDir Output directory for captures (relative to game directory)
     * @param configure Optional callback to configure the orchestrator before starting
     * @return true if orchestration started successfully, false if already running
     */
    fun startLinearOrchestration(
        items: List<WorkItem>,
        runId: String,
        scenePackages: Map<String, File>,
        outputDir: String? = null,
        configure: ((LinearOrchestrator) -> Unit)? = null,
    ): Boolean {
        if (isOrchestrationActive()) return false
        return manager.start(
            name = "Linear Orchestration",
            factory = { LinearOrchestrator().also { configure?.invoke(it) } },
            starter = { it.start(items, runId, scenePackages, outputDir) },
            isRunning = { it.isRunning },
        )
    }

    /**
     * Ticks active sessions. Must be called every client tick.
     */
    fun tick() {
        manager.tick({ it.tick() }, { it.isRunning })
    }

    /**
     * Checks if any orchestration is currently active.
     */
    fun isOrchestrationActive(): Boolean = manager.isActive { it.isRunning }

    /**
     * Generic session lifecycle manager.
     */
    private class SessionManager<T : Any> {
        private var activeSession: T? = null

        fun start(
            name: String,
            factory: () -> T,
            starter: (T) -> Boolean,
            isRunning: (T) -> Boolean,
        ): Boolean {
            if (activeSession?.let { isRunning(it) } == true) {
                Loggers.Session.get().warn("Session already in progress") { "name" to name }
                return false
            }

            val session = factory()
            if (starter(session)) {
                activeSession = session
                return true
            }
            return false
        }

        fun tick(
            tickFn: (T) -> Unit,
            isRunning: (T) -> Boolean,
        ) {
            val session = activeSession ?: return
            tickFn(session)

            if (!isRunning(session)) {
                activeSession = null
            }
        }

        fun isActive(isRunning: (T) -> Boolean): Boolean = activeSession?.let { isRunning(it) } ?: false
    }
}
