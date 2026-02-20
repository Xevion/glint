package com.xevion.glint.capture

import kotlinx.serialization.Serializable

/** Luminance statistics: mean value and 8-bin histogram of relative luminance. */
@Serializable
data class LuminanceData(
    val mean: Double,
    /** 8 equal-width bins (0.0–0.125, 0.125–0.25, …, 0.875–1.0), normalized to proportions. */
    val histogram: List<Double>,
)

/** A dominant color extracted via mean-shift clustering in CIELAB space. */
@Serializable
data class DominantColor(
    val hex: String,
    /** CIELAB coordinates [L, a, b]. */
    val lab: List<Double>,
    /** Proportion of pixels belonging to this cluster. */
    val weight: Double,
)

/** Image analytics computed from capture pixel data. */
@Serializable
data class CaptureAnalysis(
    val luminance: LuminanceData,
    val edgeDensity: Double,
    val dominantColors: List<DominantColor>,
)
