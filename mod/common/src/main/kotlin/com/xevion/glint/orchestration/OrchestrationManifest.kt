package com.xevion.glint.orchestration

import com.xevion.glint.screenshot.CaptureSessionData
import kotlinx.serialization.Serializable
import java.time.Instant

/**
 * Master manifest containing all capture session details.
 * Written to the session output directory as manifest.json.
 */
@Serializable
data class OrchestrationManifest(
    val orchestration: OrchestrationInfo,
    val sessions: List<CaptureSessionData>,
) {
    companion object {
        fun create(
            sessions: List<CaptureSessionData>,
            sessionId: String,
            startedAt: Instant,
            jobId: String? = null,
        ): OrchestrationManifest {
            val completedAt = Instant.now()

            return OrchestrationManifest(
                orchestration =
                    OrchestrationInfo(
                        id = sessionId,
                        jobId = jobId,
                        startedAt = startedAt.toString(),
                        completedAt = completedAt.toString(),
                        totalSessions = sessions.size,
                        status = OrchestrationStatus.COMPLETE,
                    ),
                sessions = sessions,
            )
        }
    }
}

@Serializable
data class OrchestrationInfo(
    val id: String,
    /** Job ID from the agent's job definition (null for interactive captures) */
    val jobId: String? = null,
    val startedAt: String,
    val completedAt: String,
    val totalSessions: Int,
    val status: OrchestrationStatus,
)

@Serializable
enum class OrchestrationStatus {
    COMPLETE,
    PARTIAL,
    FAILED,
}
