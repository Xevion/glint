package com.xevion.glint.session

import com.xevion.glint.api.WorkItem
import com.xevion.glint.orchestration.LinearOrchestrator
import java.io.File

/**
 * Manages lifecycle of the orchestrator session globally.
 *
 * Only one orchestration session may be active at a time.
 */
object SessionRegistry {
    private var activeSession: LinearOrchestrator? = null

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

        val orchestrator = LinearOrchestrator().also { configure?.invoke(it) }
        if (orchestrator.start(items, runId, scenePackages, outputDir)) {
            activeSession = orchestrator
            return true
        }
        return false
    }

    /**
     * Ticks active sessions. Must be called every client tick.
     */
    fun tick() {
        val session = activeSession ?: return
        session.tick()

        if (!session.isRunning) {
            activeSession = null
        }
    }

    /**
     * Checks if any orchestration is currently active.
     */
    fun isOrchestrationActive(): Boolean = activeSession?.isRunning == true
}
