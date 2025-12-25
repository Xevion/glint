package com.xevion.glint.session

import com.xevion.glint.Glint
import com.xevion.glint.capture.CaptureSession
import com.xevion.glint.orchestration.Orchestrator

/**
 * Manages lifecycle of tickable sessions globally.
 * Provides centralized access to active capture sessions and orchestrators.
 */
object SessionRegistry {
    private val captureSessionManager = SessionManager<CaptureSession>()
    private val orchestratorManager = SessionManager<Orchestrator>()

    /**
     * Starts a capture session for a specific scene (or all scenes if null).
     * @param sceneId The scene to capture, or null to capture all scenes
     * @return true if session started successfully, false if already running
     */
    fun startCaptureSession(sceneId: String? = null): Boolean = captureSessionManager.start(
        name = "Capture session",
        factory = { CaptureSession(sceneId = sceneId) },
        starter = { it.start() },
        isRunning = { it.isRunning },
    )

    /**
     * Starts orchestration of all scenes across all worlds.
     * @return true if orchestration started successfully, false if already running
     */
    fun startOrchestration(): Boolean = orchestratorManager.start(
        name = "Orchestration",
        factory = { Orchestrator() },
        starter = { it.start() },
        isRunning = { it.isRunning },
    )

    /**
     * Ticks active sessions. Must be called every client tick.
     */
    fun tick() {
        captureSessionManager.tick({ it.tick() }, { it.isRunning })
        orchestratorManager.tick({ it.tick() }, { it.isRunning })
    }

    /**
     * Checks if a capture session is currently active.
     */
    fun isCaptureSessionActive(): Boolean = captureSessionManager.isActive { it.isRunning }

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
                Glint.LOGGER.warn("$name already in progress")
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
