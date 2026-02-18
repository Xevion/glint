package com.xevion.glint.orchestration

import kotlinx.serialization.Serializable

/**
 * Shader pack + optional profile to capture.
 */
@Serializable
data class ShaderSpec(
    /** Shader pack filename in shaderpacks/. Null = vanilla. */
    val filename: String? = null,
    /** Iris profile name (for applying to shader config). Null = default. */
    val profile: String? = null,
    /** Backend profile ID (for API round-trips and lookup keys). Null = no profile. */
    val profileId: String? = null,
) {
    val displayName: String
        get() =
            when {
                filename == null -> "Vanilla"
                profile != null -> "$filename ($profile)"
                else -> filename
            }
}
