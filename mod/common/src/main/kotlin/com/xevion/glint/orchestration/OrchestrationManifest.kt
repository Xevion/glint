package com.xevion.glint.orchestration

import com.xevion.glint.screenshot.CaptureSessionData
import kotlinx.serialization.Serializable
import java.time.Instant

/**
 * Master manifest containing all autonomous capture session details.
 * Written to glint/captures/auto_<timestamp>/manifest.json
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
        ): OrchestrationManifest {
            val completedAt = Instant.now()

            return OrchestrationManifest(
                orchestration =
                    OrchestrationInfo(
                        id = sessionId,
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
