package com.xevion.glint.capture

import com.xevion.glint.Loggers
import kotlin.math.cbrt
import kotlin.math.exp
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.sqrt

/**
 * Computes image analytics (luminance, edge density, dominant colors) from raw ARGB pixel data.
 *
 * All methods are thread-safe and stateless — designed to run on a background thread
 * (e.g. `Util.ioPool()`) while the render thread continues.
 */
object CaptureAnalyzer {
    private val log = Loggers.Capture.get()

    /** D65 white point reference values for CIELAB conversion. */
    private const val XN = 0.95047
    private const val YN = 1.00000
    private const val ZN = 1.08883

    /** Sobel edge detection threshold (0–255 grayscale magnitude). */
    private const val EDGE_THRESHOLD = 30

    /** Mean-shift bandwidth in Lab space (ΔE units). */
    private const val MEAN_SHIFT_BANDWIDTH = 15.0

    /** Maximum mean-shift iterations per seed point. */
    private const val MAX_SHIFT_ITERATIONS = 50

    /** Maximum dominant colors to return. */
    private const val MAX_DOMINANT_COLORS = 8

    /** Convergence threshold for mean-shift (shift distance in ΔE). */
    private const val SHIFT_CONVERGENCE = 0.5

    /** Merge distance for converged modes (ΔE units). Independent of bandwidth. */
    private const val MODE_MERGE_DISTANCE = 8.0

    /**
     * Analyzes an ARGB pixel array and returns a [CaptureAnalysis].
     *
     * @param argbPixels ARGB-packed integer array (alpha in bits 24–31, red 16–23, green 8–15, blue 0–7)
     * @param width image width in pixels
     * @param height image height in pixels
     */
    fun analyze(
        argbPixels: IntArray,
        width: Int,
        height: Int,
    ): CaptureAnalysis {
        val start = System.nanoTime()

        // Downsample for performance (4x reduction → ~960x540 from 3840x2160)
        val (downsampled, dsWidth, dsHeight) = downsample(argbPixels, width, height, factor = 4)

        val luminance = computeLuminance(downsampled, dsWidth, dsHeight)
        val edgeDensity = computeEdgeDensity(downsampled, dsWidth, dsHeight)
        val dominantColors = computeDominantColors(downsampled, dsWidth, dsHeight)

        val elapsed = (System.nanoTime() - start) / 1_000_000
        log.debug("Capture analysis complete") {
            "elapsed_ms" to elapsed
            "downsampled" to "${dsWidth}x$dsHeight"
            "edge_density" to "%.4f".format(edgeDensity)
            "dominant_colors" to dominantColors.size
        }

        return CaptureAnalysis(
            luminance = luminance,
            edgeDensity = edgeDensity,
            dominantColors = dominantColors,
        )
    }

    // -- Downsampling --

    private data class DownsampleResult(
        val pixels: IntArray,
        val width: Int,
        val height: Int,
    )

    /**
     * Box-filter downsample: averages NxN pixel blocks.
     * Non-divisible remainders are dropped (truncated to factor-aligned size).
     */
    private fun downsample(
        pixels: IntArray,
        width: Int,
        height: Int,
        factor: Int,
    ): DownsampleResult {
        if (factor <= 1) return DownsampleResult(pixels, width, height)

        val outW = width / factor
        val outH = height / factor
        val result = IntArray(outW * outH)
        val blockSize = factor * factor

        for (oy in 0 until outH) {
            for (ox in 0 until outW) {
                var rSum = 0
                var gSum = 0
                var bSum = 0
                val baseX = ox * factor
                val baseY = oy * factor
                for (dy in 0 until factor) {
                    val srcRow = (baseY + dy) * width + baseX
                    for (dx in 0 until factor) {
                        val argb = pixels[srcRow + dx]
                        rSum += (argb shr 16) and 0xFF
                        gSum += (argb shr 8) and 0xFF
                        bSum += argb and 0xFF
                    }
                }
                val r = rSum / blockSize
                val g = gSum / blockSize
                val b = bSum / blockSize
                result[oy * outW + ox] = (0xFF shl 24) or (r shl 16) or (g shl 8) or b
            }
        }

        return DownsampleResult(result, outW, outH)
    }

    // -- Luminance --

    private fun computeLuminance(
        pixels: IntArray,
        width: Int,
        height: Int,
    ): LuminanceData {
        val totalPixels = width * height
        val bins = IntArray(8)
        var luminanceSum = 0.0

        for (i in 0 until totalPixels) {
            val argb = pixels[i]
            val r = ((argb shr 16) and 0xFF) / 255.0
            val g = ((argb shr 8) and 0xFF) / 255.0
            val b = (argb and 0xFF) / 255.0
            val lum = 0.2126 * r + 0.7152 * g + 0.0722 * b
            luminanceSum += lum

            val bin = min((lum * 8.0).toInt(), 7)
            bins[bin]++
        }

        val mean = luminanceSum / totalPixels
        val histogram = bins.map { it.toDouble() / totalPixels }

        return LuminanceData(mean = mean, histogram = histogram)
    }

    // -- Edge density (Sobel) --

    private fun computeEdgeDensity(
        pixels: IntArray,
        width: Int,
        height: Int,
    ): Double {
        // Convert to grayscale byte array
        val totalPixels = width * height
        val gray = IntArray(totalPixels)
        for (i in 0 until totalPixels) {
            val argb = pixels[i]
            val r = (argb shr 16) and 0xFF
            val g = (argb shr 8) and 0xFF
            val b = argb and 0xFF
            gray[i] = (0.299 * r + 0.587 * g + 0.114 * b).toInt()
        }

        var edgeCount = 0
        // Iterate interior pixels (skip 1-pixel border)
        for (y in 1 until height - 1) {
            for (x in 1 until width - 1) {
                val idx = y * width + x
                // Sobel Gx = [[-1,0,1],[-2,0,2],[-1,0,1]]
                val gx =
                    -gray[idx - width - 1] + gray[idx - width + 1] +
                        -2 * gray[idx - 1] + 2 * gray[idx + 1] +
                        -gray[idx + width - 1] + gray[idx + width + 1]
                // Sobel Gy = [[-1,-2,-1],[0,0,0],[1,2,1]]
                val gy =
                    -gray[idx - width - 1] - 2 * gray[idx - width] - gray[idx - width + 1] +
                        gray[idx + width - 1] + 2 * gray[idx + width] + gray[idx + width + 1]

                val magnitude = sqrt((gx * gx + gy * gy).toDouble())
                if (magnitude > EDGE_THRESHOLD) edgeCount++
            }
        }

        // Interior pixel count (excluding 1-pixel border)
        val interiorPixels = (width - 2) * (height - 2)
        return if (interiorPixels > 0) edgeCount.toDouble() / interiorPixels else 0.0
    }

    // -- Dominant colors (mean-shift in CIELAB) --

    private fun computeDominantColors(
        pixels: IntArray,
        width: Int,
        height: Int,
    ): List<DominantColor> {
        // Subsample every 4th pixel for performance
        val totalPixels = width * height
        val labPoints = ArrayList<DoubleArray>(totalPixels / 4 + 1)
        for (i in 0 until totalPixels step 4) {
            val argb = pixels[i]
            val r = ((argb shr 16) and 0xFF) / 255.0
            val g = ((argb shr 8) and 0xFF) / 255.0
            val b = (argb and 0xFF) / 255.0
            labPoints.add(srgbToLab(r, g, b))
        }

        if (labPoints.isEmpty()) return emptyList()

        // Use a subset of points as seed candidates (further subsample for speed)
        val seedStride = max(1, labPoints.size / 500)
        val seeds = (0 until labPoints.size step seedStride).map { labPoints[it].copyOf() }

        val bandwidthSq = MEAN_SHIFT_BANDWIDTH * MEAN_SHIFT_BANDWIDTH
        // Gaussian kernel: σ = bandwidth/2, precompute -1/(2σ²)
        val sigma = MEAN_SHIFT_BANDWIDTH / 2.0
        val gaussCoeff = -1.0 / (2.0 * sigma * sigma)
        val mergeDistSq = MODE_MERGE_DISTANCE * MODE_MERGE_DISTANCE

        // Run mean-shift on each seed
        val modes = ArrayList<DoubleArray>()
        val modeCounts = ArrayList<Int>()

        for (seed in seeds) {
            val current = seed.copyOf()
            @Suppress("UnusedPrivateProperty")
            for (_iter in 0 until MAX_SHIFT_ITERATIONS) {
                var sumL = 0.0
                var sumA = 0.0
                var sumB = 0.0
                var weightSum = 0.0

                for (point in labPoints) {
                    val dl = current[0] - point[0]
                    val da = current[1] - point[1]
                    val db = current[2] - point[2]
                    val distSq = dl * dl + da * da + db * db
                    if (distSq <= bandwidthSq) {
                        val w = exp(gaussCoeff * distSq)
                        sumL += point[0] * w
                        sumA += point[1] * w
                        sumB += point[2] * w
                        weightSum += w
                    }
                }

                if (weightSum == 0.0) break

                val newL = sumL / weightSum
                val newA = sumA / weightSum
                val newB = sumB / weightSum

                val shiftL = newL - current[0]
                val shiftA = newA - current[1]
                val shiftB = newB - current[2]
                val shiftDist = sqrt(shiftL * shiftL + shiftA * shiftA + shiftB * shiftB)

                current[0] = newL
                current[1] = newA
                current[2] = newB

                if (shiftDist < SHIFT_CONVERGENCE) break
            }

            // Merge with existing mode via weighted average
            var merged = false
            for (j in modes.indices) {
                val dl = current[0] - modes[j][0]
                val da = current[1] - modes[j][1]
                val db = current[2] - modes[j][2]
                if (dl * dl + da * da + db * db <= mergeDistSq) {
                    val w = modeCounts[j].toDouble()
                    modes[j][0] = (modes[j][0] * w + current[0]) / (w + 1.0)
                    modes[j][1] = (modes[j][1] * w + current[1]) / (w + 1.0)
                    modes[j][2] = (modes[j][2] * w + current[2]) / (w + 1.0)
                    modeCounts[j]++
                    merged = true
                    break
                }
            }
            if (!merged) {
                modes.add(current.copyOf())
                modeCounts.add(1)
            }
        }

        // Count actual pixels belonging to each mode
        val clusterCounts = IntArray(modes.size)
        for (point in labPoints) {
            var bestIdx = 0
            var bestDistSq = Double.MAX_VALUE
            for (j in modes.indices) {
                val dl = point[0] - modes[j][0]
                val da = point[1] - modes[j][1]
                val db = point[2] - modes[j][2]
                val distSq = dl * dl + da * da + db * db
                if (distSq < bestDistSq) {
                    bestDistSq = distSq
                    bestIdx = j
                }
            }
            clusterCounts[bestIdx]++
        }

        // Sort by cluster size descending, take top N
        val sortedIndices = clusterCounts.indices.sortedByDescending { clusterCounts[it] }
        val topCount = min(MAX_DOMINANT_COLORS, sortedIndices.size)
        val totalLabPoints = labPoints.size.toDouble()

        return sortedIndices
            .take(topCount)
            .filter { clusterCounts[it] > 0 }
            .map { idx ->
                val lab = modes[idx]
                val (sr, sg, sb) = labToSrgb(lab[0], lab[1], lab[2])
                val hex =
                    "#%02x%02x%02x".format(
                        clampByte(sr),
                        clampByte(sg),
                        clampByte(sb),
                    )
                DominantColor(
                    hex = hex,
                    lab =
                        listOf(
                            roundTo4(lab[0]),
                            roundTo4(lab[1]),
                            roundTo4(lab[2]),
                        ),
                    weight = roundTo4(clusterCounts[idx] / totalLabPoints),
                )
            }
    }

    // -- Color space conversion: sRGB ↔ Linear RGB ↔ XYZ (D65) ↔ CIELAB --

    private fun srgbToLinear(c: Double): Double = if (c <= 0.04045) c / 12.92 else ((c + 0.055) / 1.055).pow(2.4)

    private fun linearToSrgb(c: Double): Double = if (c <= 0.0031308) 12.92 * c else 1.055 * c.pow(1.0 / 2.4) - 0.055

    private fun srgbToLab(
        r: Double,
        g: Double,
        b: Double,
    ): DoubleArray {
        val lr = srgbToLinear(r)
        val lg = srgbToLinear(g)
        val lb = srgbToLinear(b)

        // Linear RGB to XYZ (sRGB D65 matrix)
        val x = 0.4124564 * lr + 0.3575761 * lg + 0.1804375 * lb
        val y = 0.2126729 * lr + 0.7151522 * lg + 0.0721750 * lb
        val z = 0.0193339 * lr + 0.1191920 * lg + 0.9503041 * lb

        // XYZ to Lab
        val fx = labF(x / XN)
        val fy = labF(y / YN)
        val fz = labF(z / ZN)

        val lStar = 116.0 * fy - 16.0
        val aStar = 500.0 * (fx - fy)
        val bStar = 200.0 * (fy - fz)

        return doubleArrayOf(lStar, aStar, bStar)
    }

    private fun labToSrgb(
        lStar: Double,
        aStar: Double,
        bStar: Double,
    ): Triple<Double, Double, Double> {
        // Lab to XYZ
        val fy = (lStar + 16.0) / 116.0
        val fx = aStar / 500.0 + fy
        val fz = fy - bStar / 200.0

        val x = XN * labFInv(fx)
        val y = YN * labFInv(fy)
        val z = ZN * labFInv(fz)

        // XYZ to linear RGB (inverse sRGB D65 matrix)
        val lr = 3.2404542 * x - 1.5371385 * y - 0.4985314 * z
        val lg = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z
        val lb = 0.0556434 * x - 0.2040259 * y + 1.0572252 * z

        return Triple(linearToSrgb(lr), linearToSrgb(lg), linearToSrgb(lb))
    }

    /** CIE Lab f(t) function. */
    private fun labF(t: Double): Double {
        val delta = 6.0 / 29.0
        return if (t > delta * delta * delta) {
            cbrt(t)
        } else {
            t / (3.0 * delta * delta) + 4.0 / 29.0
        }
    }

    /** CIE Lab f^{-1}(t) function. */
    private fun labFInv(t: Double): Double {
        val delta = 6.0 / 29.0
        return if (t > delta) {
            t * t * t
        } else {
            3.0 * delta * delta * (t - 4.0 / 29.0)
        }
    }

    private fun clampByte(v: Double): Int = max(0, min(255, (v * 255.0 + 0.5).toInt()))

    private fun roundTo4(v: Double): Double = Math.round(v * 10000.0) / 10000.0
}
