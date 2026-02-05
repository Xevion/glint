package com.xevion.glint.orchestration

import kotlinx.serialization.Serializable

/**
 * Unified specification for a capture run.
 * Both interactive UI and autonomous agent produce a CaptureSpec,
 * which the Orchestrator executes identically.
 */
@Serializable
data class CaptureSpec(
    /** Scene IDs to capture, in order. Worlds derived from scenes. */
    val sceneIds: List<String>,
    /** Shaders to capture for each scene. */
    val shaders: List<ShaderSpec>,
    /** Output directory (relative to game dir). Null = auto-generate under glint/captures/. */
    val outputDir: String? = null,
    /** Shutdown Minecraft after completion. */
    val shutdownOnComplete: Boolean = false,
    /** Job ID from agent, echoed in manifest. */
    val jobId: String? = null,
)

/**
 * Shader pack + optional profile to capture.
 */
@Serializable
data class ShaderSpec(
    /** Shader pack filename in shaderpacks/. Null = vanilla. */
    val filename: String? = null,
    /** Iris profile name. Null = default. */
    val profile: String? = null,
) {
    val displayName: String
        get() =
            when {
                filename == null -> "Vanilla"
                profile != null -> "$filename ($profile)"
                else -> filename
            }
}
