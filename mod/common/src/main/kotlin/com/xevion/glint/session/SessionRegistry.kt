package com.xevion.glint.session

import com.xevion.glint.Loggers
import com.xevion.glint.orchestration.CaptureSpec
import com.xevion.glint.orchestration.Orchestrator

/**
 * Manages lifecycle of the orchestrator session globally.
 */
object SessionRegistry {
    private val orchestratorManager = SessionManager<Orchestrator>()

    /**
     * Starts orchestration with the given capture spec.
     * @return true if orchestration started successfully, false if already running
     */
    fun startOrchestration(spec: CaptureSpec): Boolean =
        orchestratorManager.start(
            name = "Orchestration",
            factory = { Orchestrator() },
            starter = { it.start(spec) },
            isRunning = { it.isRunning },
        )

    /**
     * Ticks active sessions. Must be called every client tick.
     */
    fun tick() {
        orchestratorManager.tick({ it.tick() }, { it.isRunning })
    }

    /**
     * Checks if orchestration is currently active.
     */
    fun isOrchestrationActive(): Boolean = orchestratorManager.isActive { it.isRunning }

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
