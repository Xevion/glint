package com.xevion.glint.session

import com.xevion.glint.Loggers
import com.xevion.glint.api.WorkItem
import com.xevion.glint.orchestration.CaptureSpec
import com.xevion.glint.orchestration.LinearOrchestrator
import com.xevion.glint.orchestration.Orchestrator
import java.io.File

/**
 * Manages lifecycle of orchestrator sessions globally.
 *
 * Supports two orchestration modes:
 * - Interactive ([Orchestrator]) — world-based, UI-driven captures
 * - Linear ([LinearOrchestrator]) — injection-based, autonomous captures
 *
 * Only one orchestration session may be active at a time across both modes.
 */
object SessionRegistry {
    private val orchestratorManager = SessionManager<Orchestrator>()
    private val linearOrchestratorManager = SessionManager<LinearOrchestrator>()

    /**
     * Starts orchestration with the given capture spec (interactive UI path).
     * @return true if orchestration started successfully, false if already running
     */
    fun startOrchestration(
        spec: CaptureSpec,
        configure: ((Orchestrator) -> Unit)? = null,
    ): Boolean {
        if (isOrchestrationActive()) return false
        return orchestratorManager.start(
            name = "Orchestration",
            factory = { Orchestrator().also { configure?.invoke(it) } },
            starter = { it.start(spec) },
            isRunning = { it.isRunning },
        )
    }

    /**
     * Starts linear orchestration with pre-ordered work items (autonomous capture path).
     * Uses [LinearOrchestrator] with scene package injection instead of world files.
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
        return linearOrchestratorManager.start(
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
        orchestratorManager.tick({ it.tick() }, { it.isRunning })
        linearOrchestratorManager.tick({ it.tick() }, { it.isRunning })
    }

    /**
     * Checks if any orchestration is currently active.
     */
    fun isOrchestrationActive(): Boolean =
        orchestratorManager.isActive { it.isRunning } ||
            linearOrchestratorManager.isActive { it.isRunning }

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
