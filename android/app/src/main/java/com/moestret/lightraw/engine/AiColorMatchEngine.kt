package com.moestret.lightraw.engine

import android.graphics.Bitmap
import com.moestret.lightraw.model.DevelopParams
import kotlin.math.atan2
import kotlin.math.cbrt
import kotlin.math.cos
import kotlin.math.exp
import kotlin.math.floor
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.roundToInt
import kotlin.math.sin
import kotlin.math.sqrt

/**
 * LightRaw AI Color Match & Auto Tone Engine (Mobile Twin of `src/ai/color_match.rs`).
 * Computes perceptual OKLab Luminance-CDF (with highlight rolloff protection) + 8-zone
 * median-anchored 2x2 Monge-Kantorovich chromatic covariance transport encoded as a smooth
 * 25x25x25 3D LUT with neutral stability, deep-shadow & highlight restraint, and face safety.
 */
object AiColorMatchEngine {

    const val MATCH_LUT_DIM = 25

    data class ColorStats(
        val p05Lum: Float,
        val p25Lum: Float,
        val p50Lum: Float,
        val p75Lum: Float,
        val p95Lum: Float,
        val meanSat: Float,
        val midRgRatio: Float,
        val midBgRatio: Float,
        val shadowHue: Float,
        val shadowSat: Float,
        val highlightHue: Float,
        val highlightSat: Float,
        val skinPixelCount: Int,
        val skinMedianLum: Float,
        val skinMeanRg: Float,
        val skinMeanBg: Float
    )

    enum class RegionRole { UNIFIED, SUBJECT, BACKGROUND }

    data class FaceRect(val cx: Float, val cy: Float, val rx: Float, val ry: Float)

    data class SemanticGrid(
        val width: Int,
        val height: Int,
        val subjectWeights: FloatArray,
        val skinWeights: FloatArray,
        val faces: List<FaceRect>
    )

    fun sampleBilinear(grid: FloatArray, gw: Int, gh: Int, nx: Float, ny: Float): Float {
        if (gw <= 0 || gh <= 0 || grid.size != gw * gh) return 1f
        val x = nx.coerceIn(0f, 1f) * max(1, gw - 1)
        val y = ny.coerceIn(0f, 1f) * max(1, gh - 1)
        val x0 = floor(x).toInt().coerceIn(0, gw - 1)
        val y0 = floor(y).toInt().coerceIn(0, gh - 1)
        val x1 = min(gw - 1, x0 + 1)
        val y1 = min(gh - 1, y0 + 1)
        val fx = x - x0
        val fy = y - y0
        val v00 = grid[y0 * gw + x0]
        val v10 = grid[y0 * gw + x1]
        val v01 = grid[y1 * gw + x0]
        val v11 = grid[y1 * gw + x1]
        val top = v00 + (v10 - v00) * fx
        val bot = v01 + (v11 - v01) * fx
        return top + (bot - top) * fy
    }

    fun detectFaceRects(bitmap: Bitmap): List<FaceRect> {
        return try {
            val fw = max(4, bitmap.width and -2)
            val fh = max(4, bitmap.height)
            var scaled: Bitmap? = null
            var rgb565: Bitmap? = null
            try {
                scaled = if (fw != bitmap.width || fh != bitmap.height) {
                    Bitmap.createScaledBitmap(bitmap, fw, fh, true)
                } else null
                val srcFor565 = scaled ?: bitmap
                rgb565 = if (srcFor565.config == Bitmap.Config.RGB_565) {
                    srcFor565
                } else {
                    srcFor565.copy(Bitmap.Config.RGB_565, false)
                }
                if (rgb565 == null) return emptyList()
                val detector = android.media.FaceDetector(fw, fh, 4)
                val found = arrayOfNulls<android.media.FaceDetector.Face>(4)
                val count = detector.findFaces(rgb565, found)
                val out = ArrayList<FaceRect>(count)
                val mid = android.graphics.PointF()
                for (k in 0 until count) {
                    val f = found[k] ?: continue
                    if (f.confidence() < 0.32f) continue
                    f.getMidPoint(mid)
                    val ed = f.eyesDistance()
                    if (ed > 1.5f) {
                        out.add(
                            FaceRect(
                                cx = (mid.x / fw).coerceIn(0f, 1f),
                                cy = ((mid.y + ed * 0.25f) / fh).coerceIn(0f, 1f),
                                rx = (ed * 1.15f / fw).coerceIn(0.03f, 0.30f),
                                ry = (ed * 1.55f / fh).coerceIn(0.04f, 0.38f)
                            )
                        )
                    }
                }
                out
            } finally {
                if (rgb565 != null && rgb565 !== bitmap && rgb565 !== scaled && !rgb565.isRecycled) {
                    rgb565.recycle()
                }
                if (scaled != null && scaled !== bitmap && !scaled.isRecycled) {
                    scaled.recycle()
                }
            }
        } catch (_: Throwable) {
            emptyList()
        }
    }

    fun extractSemanticGrid(
        pixels: IntArray,
        w: Int,
        h: Int,
        faces: List<FaceRect> = emptyList()
    ): SemanticGrid {
        if (w < 16 || h < 16 || pixels.size < w * h) {
            return SemanticGrid(1, 1, floatArrayOf(1f), floatArrayOf(0f), faces)
        }
        val scale = min(1f, 72f / max(w, h))
        val gw = max(16, (w * scale).roundToInt())
        val gh = max(16, (h * scale).roundToInt())
        val n = gw * gh

        val linList = ArrayList<Float>(n / 2 + 4)
        val linR = FloatArray(n)
        val linG = FloatArray(n)
        val linB = FloatArray(n)
        for (gy in 0 until gh) {
            val sy = min(h - 1, ((gy + 0.5f) / gh * h).toInt())
            for (gx in 0 until gw) {
                val sx = min(w - 1, ((gx + 0.5f) / gw * w).toInt())
                val c = pixels[sy * w + sx]
                val r = srgbToLinear(((c ushr 16) and 0xFF) / 255f)
                val g = srgbToLinear(((c ushr 8) and 0xFF) / 255f)
                val b = srgbToLinear((c and 0xFF) / 255f)
                val idx = gy * gw + gx
                linR[idx] = r; linG[idx] = g; linB[idx] = b
                if (((gx + gy) and 1) == 0) {
                    linList.add(0.2126f * r + 0.7152f * g + 0.0722f * b)
                }
            }
        }
        linList.sort()
        val medLin = linList[linList.size / 2].coerceAtLeast(0.015f)
        val expGain = (0.20f / medLin).coerceIn(0.45f, 4.5f)

        val gL = FloatArray(n)
        val gA = FloatArray(n)
        val gB = FloatArray(n)
        val skinWeights = FloatArray(n)
        val labTmp = FloatArray(3)
        val bL = ArrayList<Float>()
        val bA = ArrayList<Float>()
        val bB = ArrayList<Float>()
        val rowMargin = max(2, (gh * 0.08f).toInt())
        val colMargin = max(2, (gw * 0.08f).toInt())

        for (gy in 0 until gh) {
            val ny = gy / max(1, gh - 1).toFloat()
            val isRowBorder = gy < rowMargin || gy >= gh - rowMargin
            for (gx in 0 until gw) {
                val nx = gx / max(1, gw - 1).toFloat()
                val idx = gy * gw + gx
                val rn = linearToSrgb((linR[idx] * expGain).coerceIn(0f, 1f))
                val gn = linearToSrgb((linG[idx] * expGain).coerceIn(0f, 1f))
                val bn = linearToSrgb((linB[idx] * expGain).coerceIn(0f, 1f))
                srgbToOklab(rn, gn, bn, labTmp)
                gL[idx] = labTmp[0]; gA[idx] = labTmp[1]; gB[idx] = labTmp[2]
                if (isRowBorder || gx < colMargin || gx >= gw - colMargin) {
                    bL.add(labTmp[0]); bA.add(labTmp[1]); bB.add(labTmp[2])
                }
                var fMask = 0f
                for (f in faces) {
                    val dx = (nx - f.cx) / max(0.02f, f.rx)
                    val dy = (ny - f.cy) / max(0.02f, f.ry)
                    val d = sqrt(dx * dx + dy * dy)
                    fMask = max(fMask, 1f - smoothstep(0.75f, 1.20f, d))
                }
                val isSk = isSkinSrgb(rn, gn, bn, true)
                val centerSkZone = nx in 0.18f..0.82f && ny in 0.08f..0.72f
                skinWeights[idx] = when {
                    fMask > 0.2f && isSk -> fMask
                    fMask > 0.45f -> fMask * 0.65f
                    faces.isEmpty() && centerSkZone && isSk -> 0.80f
                    else -> 0f
                }
            }
        }

        bL.sort(); bA.sort(); bB.sort()
        val bgL = bL[bL.size / 2]
        val bgA = bA[bA.size / 2]
        val bgB = bB[bB.size / 2]

        var borderInliers = 0
        for (bi in bL.indices) {
            val dl = bL[bi] - bgL
            val da = bA[bi] - bgA
            val db = bB[bi] - bgB
            val d = sqrt(dl * dl * 0.75f + da * da * 6.8f + db * db * 6.8f)
            if (d <= 0.22f) borderInliers++
        }
        val borderInlierFrac = borderInliers.toFloat() / max(1, bL.size)
        if (faces.isEmpty() && borderInlierFrac < 0.48f) {
            return SemanticGrid(
                gw,
                gh,
                FloatArray(n) { 1f },
                skinWeights,
                faces
            )
        }

        val rawSubj = FloatArray(n)
        for (gy in 0 until gh) {
            val ny = gy / max(1, gh - 1).toFloat()
            for (gx in 0 until gw) {
                val nx = gx / max(1, gw - 1).toFloat()
                val idx = gy * gw + gx
                val dl = gL[idx] - bgL
                val da = gA[idx] - bgA
                val db = gB[idx] - bgB
                val distBg = sqrt(dl * dl * 0.75f + da * da * 6.8f + db * db * 6.8f)
                val sBg = smoothstep(0.11f, 0.28f, distBg)

                var faceCore = 0f
                var torsoPrior = 0f
                for (f in faces) {
                    val dx = (nx - f.cx) / max(0.02f, f.rx)
                    val dy = (ny - f.cy) / max(0.02f, f.ry)
                    val dHead = sqrt(dx * dx + dy * dy)
                    faceCore = max(faceCore, 1f - smoothstep(0.90f, 1.35f, dHead))
                    val downT = ((ny - f.cy) / max(0.04f, f.ry)).coerceIn(0f, 5.2f)
                    if (ny >= f.cy - f.ry * 0.5f) {
                        val halfW = f.rx * (1.25f + downT * 0.36f)
                        val tDist = kotlin.math.abs(nx - f.cx) / max(0.04f, halfW)
                        torsoPrior = max(torsoPrior, 1f - smoothstep(0.65f, 1.25f, tDist))
                    }
                }
                val cdx = (nx - 0.50f) * 1.15f
                val cdy = (ny - 0.52f) * 0.90f
                val centerPrior = 1f - smoothstep(0.28f, 0.76f, sqrt(cdx * cdx + cdy * cdy))
                val spatialPrior = if (faces.isNotEmpty()) max(torsoPrior, centerPrior * 0.85f) else centerPrior
                rawSubj[idx] = max(faceCore, sBg * (0.68f + 0.32f * spatialPrior)).coerceIn(0f, 1f)
            }
        }

        // Smooth subject weights with a separable 3-tap box blur pass so mask boundaries have no staircase jitter
        val tmpBlur = FloatArray(n)
        for (gy in 0 until gh) {
            val row = gy * gw
            for (gx in 0 until gw) {
                val l = rawSubj[row + max(0, gx - 1)]
                val r = rawSubj[row + min(gw - 1, gx + 1)]
                tmpBlur[row + gx] = (l + rawSubj[row + gx] * 2f + r) * 0.25f
            }
        }
        val subjOut = FloatArray(n)
        for (gy in 0 until gh) {
            val up = max(0, gy - 1) * gw
            val cur = gy * gw
            val dn = min(gh - 1, gy + 1) * gw
            for (gx in 0 until gw) {
                val v = (tmpBlur[up + gx] + tmpBlur[cur + gx] * 2f + tmpBlur[dn + gx]) * 0.25f
                subjOut[cur + gx] = v.coerceIn(0f, 1f)
            }
        }
        return SemanticGrid(gw, gh, subjOut, skinWeights, faces)
    }

    private fun srgbToLinear(c: Float): Float {
        val x = c.coerceIn(0f, 1f)
        return if (x <= 0.04045f) x / 12.92f else ((x + 0.055f) / 1.055f).pow(2.4f)
    }

    private fun linearToSrgb(c: Float): Float {
        val x = c.coerceIn(0f, 1f)
        return if (x <= 0.0031308f) x * 12.92f else 1.055f * x.pow(1f / 2.4f) - 0.055f
    }

    private fun srgbToOklab(r: Float, g: Float, b: Float, out: FloatArray) {
        val rl = srgbToLinear(r)
        val gl = srgbToLinear(g)
        val bl = srgbToLinear(b)

        val l = cbrt(max(0f, 0.41222147f * rl + 0.53633255f * gl + 0.05144599f * bl))
        val m = cbrt(max(0f, 0.21190350f * rl + 0.68069955f * gl + 0.10739696f * bl))
        val s = cbrt(max(0f, 0.08830246f * rl + 0.28171884f * gl + 0.62997870f * bl))

        out[0] = 0.21045426f * l + 0.79361779f * m - 0.00407205f * s
        out[1] = 1.97799850f * l - 2.42859220f * m + 0.45059371f * s
        out[2] = 0.02590404f * l + 0.78277177f * m - 0.80867577f * s
    }

    private fun oklabToLinear(okL: Float, okA: Float, okB: Float, out: FloatArray) {
        val l_ = okL + 0.39633778f * okA + 0.21580376f * okB
        val m_ = okL - 0.10556135f * okA - 0.06385417f * okB
        val s_ = okL - 0.08948418f * okA - 1.29148555f * okB

        val l = l_ * l_ * l_
        val m = m_ * m_ * m_
        val s = s_ * s_ * s_

        out[0] = 4.07674166f * l - 3.30771159f * m + 0.23096993f * s
        out[1] = -1.26843800f * l + 2.60975740f * m - 0.34131940f * s
        out[2] = -0.00419609f * l - 0.70341861f * m + 1.70761470f * s
    }

    private fun oklabToSrgbGamut(okL: Float, okA: Float, okB: Float, linTmp: FloatArray, out: FloatArray) {
        val lC = okL.coerceIn(0f, 1f)
        oklabToLinear(lC, okA, okB, linTmp)
        if (linTmp[0] < 0f || linTmp[1] < 0f || linTmp[2] < 0f ||
            linTmp[0] > 1f || linTmp[1] > 1f || linTmp[2] > 1f
        ) {
            var lo = 0f
            var hi = 1f
            for (iter in 0 until 6) {
                val mid = 0.5f * (lo + hi)
                oklabToLinear(lC, okA * mid, okB * mid, linTmp)
                if (linTmp[0] >= -1e-4f && linTmp[1] >= -1e-4f && linTmp[2] >= -1e-4f &&
                    linTmp[0] <= 1.0001f && linTmp[1] <= 1.0001f && linTmp[2] <= 1.0001f
                ) {
                    lo = mid
                } else {
                    hi = mid
                }
            }
            oklabToLinear(lC, okA * lo, okB * lo, linTmp)
        }
        out[0] = linearToSrgb(linTmp[0])
        out[1] = linearToSrgb(linTmp[1])
        out[2] = linearToSrgb(linTmp[2])
    }

    fun sampleLut3d(lut: FloatArray, dim: Int, r: Float, g: Float, b: Float, out: FloatArray) {
        if (dim < 2 || lut.size != dim * dim * dim * 3) {
            out[0] = r; out[1] = g; out[2] = b
            return
        }
        val scale = (dim - 1).toFloat()
        val rf = (r.coerceIn(0f, 1f) * scale).coerceIn(0f, scale)
        val gf = (g.coerceIn(0f, 1f) * scale).coerceIn(0f, scale)
        val bf = (b.coerceIn(0f, 1f) * scale).coerceIn(0f, scale)

        val r0 = min(dim - 1, floor(rf).toInt())
        val g0 = min(dim - 1, floor(gf).toInt())
        val b0 = min(dim - 1, floor(bf).toInt())
        val r1 = min(dim - 1, r0 + 1)
        val g1 = min(dim - 1, g0 + 1)
        val b1 = min(dim - 1, b0 + 1)

        val dr = rf - r0
        val dg = gf - g0
        val db = bf - b0
        val d2 = dim * dim

        fun ch(ri: Int, gi: Int, bi: Int, c: Int) = lut[(bi * d2 + gi * dim + ri) * 3 + c]

        for (c in 0 until 3) {
            val c00 = ch(r0, g0, b0, c) + (ch(r1, g0, b0, c) - ch(r0, g0, b0, c)) * dr
            val c10 = ch(r0, g1, b0, c) + (ch(r1, g1, b0, c) - ch(r0, g1, b0, c)) * dr
            val c01 = ch(r0, g0, b1, c) + (ch(r1, g0, b1, c) - ch(r0, g0, b1, c)) * dr
            val c11 = ch(r0, g1, b1, c) + (ch(r1, g1, b1, c) - ch(r0, g1, b1, c)) * dr
            val c0 = c00 + (c10 - c00) * dg
            val c1 = c01 + (c11 - c01) * dg
            out[c] = c0 + (c1 - c0) * db
        }
    }

    fun applyColorMatchLut(
        r: Float,
        g: Float,
        b: Float,
        lut: FloatArray,
        dim: Int,
        toneAmountPct: Float,
        colorAmountPct: Float,
        tmpLutRgb: FloatArray,
        tmpLabIn: FloatArray,
        tmpLabLut: FloatArray,
        tmpLin: FloatArray,
        outRgb: FloatArray
    ) {
        applyColorMatchLutSemantic(
            r, g, b, lut, null, 1f, dim, toneAmountPct, colorAmountPct,
            tmpLutRgb, tmpLabIn, tmpLabLut, tmpLin, outRgb
        )
    }

    fun applyColorMatchLutSemantic(
        r: Float,
        g: Float,
        b: Float,
        mainLut: FloatArray,
        bgLut: FloatArray?,
        subjWeight: Float,
        dim: Int,
        toneAmountPct: Float,
        colorAmountPct: Float,
        tmpLutRgb: FloatArray,
        tmpLabIn: FloatArray,
        tmpLabLut: FloatArray,
        tmpLin: FloatArray,
        outRgb: FloatArray
    ) {
        val wt = (toneAmountPct / 100f).coerceIn(0f, 1f)
        val wc = (colorAmountPct / 100f).coerceIn(0f, 1f)
        val expectedLen = dim * dim * dim * 3
        if ((wt <= 1e-4f && wc <= 1e-4f) || dim < 2 || mainLut.size != expectedLen) {
            outRgb[0] = r; outRgb[1] = g; outRgb[2] = b
            return
        }
        val hasBg = bgLut != null && bgLut.size == expectedLen
        val sw = subjWeight.coerceIn(0f, 1f)
        if (!hasBg || sw >= 0.985f) {
            sampleLut3d(mainLut, dim, r, g, b, tmpLutRgb)
        } else if (sw <= 0.015f) {
            sampleLut3d(bgLut!!, dim, r, g, b, tmpLutRgb)
        } else {
            sampleLut3d(bgLut!!, dim, r, g, b, tmpLabIn)
            sampleLut3d(mainLut, dim, r, g, b, tmpLabLut)
            tmpLutRgb[0] = tmpLabIn[0] + (tmpLabLut[0] - tmpLabIn[0]) * sw
            tmpLutRgb[1] = tmpLabIn[1] + (tmpLabLut[1] - tmpLabIn[1]) * sw
            tmpLutRgb[2] = tmpLabIn[2] + (tmpLabLut[2] - tmpLabIn[2]) * sw
        }
        if (wt >= 0.998f && wc >= 0.998f) {
            outRgb[0] = tmpLutRgb[0]
            outRgb[1] = tmpLutRgb[1]
            outRgb[2] = tmpLutRgb[2]
            return
        }
        srgbToOklab(r, g, b, tmpLabIn)
        srgbToOklab(tmpLutRgb[0], tmpLutRgb[1], tmpLutRgb[2], tmpLabLut)
        val outL = tmpLabIn[0] + (tmpLabLut[0] - tmpLabIn[0]) * wt
        val outA = tmpLabIn[1] + (tmpLabLut[1] - tmpLabIn[1]) * wc
        val outB = tmpLabIn[2] + (tmpLabLut[2] - tmpLabIn[2]) * wc
        oklabToSrgbGamut(outL, outA, outB, tmpLin, outRgb)
    }

    private fun smoothstep(edge0: Float, edge1: Float, x: Float): Float {
        val t = ((x - edge0) / max(1e-5f, edge1 - edge0)).coerceIn(0f, 1f)
        return t * t * (3f - 2f * t)
    }

    private fun rgbHueSat(r: Float, g: Float, b: Float): Pair<Float, Float> {
        val maxC = max(r, max(g, b))
        val minC = min(r, min(g, b))
        val delta = maxC - minC
        if (delta < 1e-4f || maxC < 1e-4f) return 0f to 0f
        var hue = when {
            maxC == r -> ((g - b) / delta) % 6f
            maxC == g -> ((b - r) / delta) + 2f
            else -> ((r - g) / delta) + 4f
        } * 60f
        if (hue < 0f) hue += 360f
        return hue to (delta / maxC * 100f).coerceIn(0f, 100f)
    }

    private fun isSkinSrgb(r: Float, g: Float, b: Float, lenientFlat: Boolean): Boolean {
        val lum = 0.2126f * r + 0.7152f * g + 0.0722f * b
        val (hue, sat) = rgbHueSat(r, g, b)
        return if (lenientFlat) {
            lum in 0.16f..0.90f &&
                r > g && g > b &&
                (r - g) >= 0.006f && (g - b) >= 0.004f &&
                (r - b) in 0.015f..0.65f &&
                hue in 8f..52f && sat in 4f..75f
        } else {
            lum in 0.15f..0.90f &&
                r > g && g > b &&
                (r - g) >= 0.02f && (g - b) >= 0.012f &&
                (r - b) in 0.05f..0.65f &&
                hue in 10f..48f && sat in 12f..75f
        }
    }

    fun extractStatsFromPixels(pixels: IntArray): ColorStats {
        val total = pixels.size
        if (total == 0) {
            return ColorStats(0.1f, 0.25f, 0.5f, 0.75f, 0.9f, 25f, 1f, 1f, 0f, 0f, 0f, 0f, 0, 0.5f, 1f, 1f)
        }
        val step = max(1, total / 12000)

        val lums = ArrayList<Float>(total / step + 8)
        val skinLums = ArrayList<Float>()
        var sumSat = 0.0
        var satCount = 0

        var midR = 0.0; var midG = 0.0; var midB = 0.0; var midCount = 0
        var shdR = 0.0; var shdG = 0.0; var shdB = 0.0; var shdCount = 0
        var hiR = 0.0; var hiG = 0.0; var hiB = 0.0; var hiCount = 0
        var skinRgSum = 0.0; var skinBgSum = 0.0

        var i = 0
        while (i < total) {
            val c = pixels[i]
            val r = ((c ushr 16) and 0xFF) / 255f
            val g = ((c ushr 8) and 0xFF) / 255f
            val b = (c and 0xFF) / 255f

            val lum = 0.2126f * r + 0.7152f * g + 0.0722f * b
            val (_, sat) = rgbHueSat(r, g, b)

            lums.add(lum)
            sumSat += sat
            satCount++

            if (isSkinSrgb(r, g, b, false)) {
                skinLums.add(lum)
                if (g > 1e-4f) {
                    skinRgSum += r / g
                    skinBgSum += b / g
                }
            }
            if (lum in 0.02f..0.30f) {
                shdR += r; shdG += g; shdB += b; shdCount++
            } else if (lum in 0.70f..0.98f) {
                hiR += r; hiG += g; hiB += b; hiCount++
            }
            if (lum in 0.20f..0.80f) {
                midR += r; midG += g; midB += b; midCount++
            }
            i += step
        }

        lums.sort()
        val n = lums.size
        fun pct(p: Float) = lums[(p * (n - 1)).toInt().coerceIn(0, n - 1)]

        val midRg = if (midCount > 0 && midG > 1e-5) (midR / midG).toFloat() else 1f
        val midBg = if (midCount > 0 && midG > 1e-5) (midB / midG).toFloat() else 1f

        val (shdH, shdS) = if (shdCount > 0) rgbHueSat((shdR / shdCount).toFloat(), (shdG / shdCount).toFloat(), (shdB / shdCount).toFloat()) else 0f to 0f
        val (hiH, hiS) = if (hiCount > 0) rgbHueSat((hiR / hiCount).toFloat(), (hiG / hiCount).toFloat(), (hiB / hiCount).toFloat()) else 0f to 0f

        skinLums.sort()
        val skinMed = if (skinLums.isNotEmpty()) skinLums[skinLums.size / 2] else 0.5f
        val skinRg = if (skinLums.isNotEmpty()) (skinRgSum / skinLums.size).toFloat() else 1f
        val skinBg = if (skinLums.isNotEmpty()) (skinBgSum / skinLums.size).toFloat() else 1f

        return ColorStats(
            p05Lum = pct(0.05f),
            p25Lum = pct(0.25f),
            p50Lum = pct(0.50f),
            p75Lum = pct(0.75f),
            p95Lum = pct(0.95f),
            meanSat = (sumSat / max(1, satCount)).toFloat(),
            midRgRatio = midRg,
            midBgRatio = midBg,
            shadowHue = shdH,
            shadowSat = shdS,
            highlightHue = hiH,
            highlightSat = hiS,
            skinPixelCount = skinLums.size,
            skinMedianLum = skinMed,
            skinMeanRg = skinRg,
            skinMeanBg = skinBg
        )
    }

    fun extractStats(bitmap: Bitmap): ColorStats {
        val pixels = IntArray(bitmap.width * bitmap.height)
        if (pixels.isNotEmpty()) {
            bitmap.getPixels(pixels, 0, bitmap.width, 0, 0, bitmap.width, bitmap.height)
        }
        return extractStatsFromPixels(pixels)
    }

    fun autoTone(bitmap: Bitmap, current: DevelopParams): DevelopParams {
        val st = extractStats(bitmap)
        val anchorLum = if (st.skinPixelCount >= 30) st.skinMedianLum else st.p50Lum
        val targetMid = if (st.skinPixelCount >= 30) 0.52f else 0.48f
        val dEv = ((targetMid - anchorLum) * 1.45f).coerceIn(-0.85f, 0.85f)

        val spread = (st.p95Lum - st.p05Lum).coerceAtLeast(0.08f)
        val dCon = ((0.76f / spread - 1f) * 28f).coerceIn(-18f, 24f)
        val dHi = if (st.p95Lum > 0.85f) -((st.p95Lum - 0.82f) * 120f).coerceIn(0f, 38f) else 8f
        val dShd = if (st.p25Lum < 0.24f) ((0.24f - st.p25Lum) * 140f).coerceIn(0f, 36f) else 6f
        val dWht = ((0.90f - st.p95Lum) * 35f).coerceIn(-18f, 18f)
        val dBlk = ((0.06f - st.p05Lum) * 40f).coerceIn(-18f, 15f)
        val dVib = if (st.meanSat < 26f) ((28f - st.meanSat) * 0.7f).coerceIn(0f, 18f) else 0f

        return current.copy(
            exposure = (current.exposure + dEv).coerceIn(-3f, 3f),
            contrast = (current.contrast + dCon).coerceIn(-50f, 50f),
            highlights = dHi.coerceIn(-60f, 35f),
            shadows = dShd.coerceIn(-35f, 60f),
            whites = dWht.coerceIn(-25f, 25f),
            blacks = dBlk.coerceIn(-25f, 25f),
            vibrance = (current.vibrance + dVib).coerceIn(-20f, 28f)
        )
    }

    private fun extractPaletteFromLabs(
        samples: FloatArray,
        sampleCount: Int,
        count: Int = 10
    ): IntArray {
        if (sampleCount <= 0 || count <= 0) return IntArray(0)
        val order = (0 until sampleCount).sortedBy { samples[it * 3] }
        val tmpLin = FloatArray(3)
        val tmpRgb = FloatArray(3)
        val palette = IntArray(count)
        for (k in 0 until count) {
            val start = (k * sampleCount) / count
            val end = max(start + 1, min(sampleCount, ((k + 1) * sampleCount) / count))
            var sumW = 0.0
            var sumL = 0.0
            var sumA = 0.0
            var sumB = 0.0
            for (pos in start until end) {
                val i = order[pos] * 3
                val l = samples[i]
                val a = samples[i + 1]
                val b = samples[i + 2]
                val ch = sqrt(a * a + b * b)
                val w = (0.35 + ch * 6.0)
                sumW += w
                sumL += l * w
                sumA += a * w
                sumB += b * w
            }
            val inv = 1.0 / max(1e-6, sumW)
            oklabToSrgbGamut(
                (sumL * inv).toFloat(),
                (sumA * inv).toFloat(),
                (sumB * inv).toFloat(),
                tmpLin,
                tmpRgb
            )
            val ri = (tmpRgb[0] * 255f).roundToInt().coerceIn(0, 255)
            val gi = (tmpRgb[1] * 255f).roundToInt().coerceIn(0, 255)
            val bi = (tmpRgb[2] * 255f).roundToInt().coerceIn(0, 255)
            palette[k] = (0xFF shl 24) or (ri shl 16) or (gi shl 8) or bi
        }
        return palette
    }

    private fun quantileFromSorted(sorted: FloatArray, q: Float): Float {
        if (sorted.isEmpty()) return 0.5f
        val idxF = q.coerceIn(0f, 1f) * (sorted.size - 1)
        val i0 = floor(idxF).toInt()
        val i1 = min(sorted.size - 1, i0 + 1)
        val frac = idxF - i0
        return sorted[i0] * (1f - frac) + sorted[i1] * frac
    }

    private fun lowerBound(arr: FloatArray, x: Float): Int {
        var lo = 0
        var hi = arr.size
        while (lo < hi) {
            val mid = (lo + hi) ushr 1
            if (arr[mid] < x) lo = mid + 1 else hi = mid
        }
        return lo
    }

    private fun upperBound(arr: FloatArray, x: Float): Int {
        var lo = 0
        var hi = arr.size
        while (lo < hi) {
            val mid = (lo + hi) ushr 1
            if (arr[mid] <= x) lo = mid + 1 else hi = mid
        }
        return lo
    }

    private fun cdfRank(sorted: FloatArray, x: Float): Float {
        if (sorted.size <= 1) return x.coerceIn(0f, 1f)
        val lo = lowerBound(sorted, x)
        val hi = upperBound(sorted, x)
        if (lo < hi) {
            return (0.5f * (lo + hi - 1)) / (sorted.size - 1)
        }
        if (lo == 0) return 0f
        if (lo >= sorted.size) return 1f
        val v0 = sorted[lo - 1]
        val v1 = sorted[lo]
        val frac = if (kotlin.math.abs(v1 - v0) > 1e-6f) ((x - v0) / (v1 - v0)).coerceIn(0f, 1f) else 0.5f
        return ((lo - 1) + frac) / (sorted.size - 1)
    }

    private data class Mat2(val m00: Float, val m01: Float, val m10: Float, val m11: Float) {
        fun mul(r: Mat2) = Mat2(
            m00 * r.m00 + m01 * r.m10,
            m00 * r.m01 + m01 * r.m11,
            m10 * r.m00 + m11 * r.m10,
            m10 * r.m01 + m11 * r.m11
        )
    }

    private fun eigenSym2x2(a: Float, b: Float, c: Float): FloatArray {
        val tr = 0.5f * (a + c)
        val diff = 0.5f * (a - c)
        val rad = sqrt(diff * diff + b * b)
        val l1 = max(0f, tr + rad)
        val l2 = max(0f, tr - rad)
        val theta = if (kotlin.math.abs(b) < 1e-10f && kotlin.math.abs(diff) < 1e-10f) 0f else 0.5f * atan2(b, diff)
        return floatArrayOf(cos(theta), sin(theta), l1, l2)
    }

    private fun reconstructSym2x2(cosT: Float, sinT: Float, d1: Float, d2: Float): Mat2 {
        val c2 = cosT * cosT
        val s2 = sinT * sinT
        val cs = cosT * sinT
        return Mat2(d1 * c2 + d2 * s2, (d1 - d2) * cs, (d1 - d2) * cs, d1 * s2 + d2 * c2)
    }

    private fun weightedMedian(vals: ArrayList<Float>, weights: ArrayList<Float>): Float {
        if (vals.isEmpty()) return 0f
        val idx = IntArray(vals.size) { it }.sortedBy { vals[it] }
        var totalW = 0.0
        for (k in idx) totalW += weights[k]
        if (totalW <= 1e-6) return vals[idx[idx.size / 2]]
        val half = 0.5 * totalW
        var acc = 0.0
        for (k in idx) {
            acc += weights[k]
            if (acc >= half) return vals[k]
        }
        return vals[idx.last()]
    }

    private fun computeMk2x2(
        taa: Float, tab: Float, tbb: Float,
        raa: Float, rab: Float, rbb: Float,
        epsT: Float, epsR: Float,
        minGain: Float, maxGain: Float
    ): Mat2 {
        val et = eigenSym2x2(taa + epsT, tab, tbb + epsT)
        val sqrtT = reconstructSym2x2(et[0], et[1], sqrt(et[2]), sqrt(et[3]))
        val invSqrtT = reconstructSym2x2(et[0], et[1], 1f / sqrt(et[2]), 1f / sqrt(et[3]))
        val sigR = Mat2(raa + epsR, rab, rab, rbb + epsR)
        val inner = sqrtT.mul(sigR).mul(sqrtT)
        val ep = eigenSym2x2(inner.m00, 0.5f * (inner.m01 + inner.m10), inner.m11)
        val sqrtInner = reconstructSym2x2(ep[0], ep[1], sqrt(ep[2]), sqrt(ep[3]))
        val tMat = invSqrtT.mul(sqrtInner).mul(invSqrtT)
        val em = eigenSym2x2(tMat.m00, 0.5f * (tMat.m01 + tMat.m10), tMat.m11)
        return reconstructSym2x2(
            em[0], em[1],
            em[2].coerceIn(minGain, maxGain),
            em[3].coerceIn(minGain, maxGain)
        )
    }

    private data class RegionSamples(
        val r: FloatArray,
        val g: FloatArray,
        val b: FloatArray,
        val size: Int
    )

    private data class SemanticSampleBundle(
        val all: RegionSamples,
        val subj: RegionSamples,
        val bg: RegionSamples,
        val faceSkinMedianLab: FloatArray?
    )

    private data class MatchDualLutOutput(
        val mainLut: FloatArray,
        val bgLut: FloatArray,
        val palette: IntArray,
        val isFlat: Boolean
    )

    private fun extractSemanticSamples(
        pixels: IntArray,
        w: Int,
        h: Int,
        faces: List<FaceRect>
    ): SemanticSampleBundle {
        val total = min(pixels.size, w * h)
        val grid = extractSemanticGrid(pixels, w, h, faces)
        val step = max(1, total / 12000)
        val maxN = total / step + 4

        val allR = FloatArray(maxN); val allG = FloatArray(maxN); val allB = FloatArray(maxN); var allN = 0
        val subjR = FloatArray(maxN); val subjG = FloatArray(maxN); val subjB = FloatArray(maxN); var subjN = 0
        val bgR = FloatArray(maxN); val bgG = FloatArray(maxN); val bgB = FloatArray(maxN); var bgN = 0
        val skinL = ArrayList<Float>(); val skinA = ArrayList<Float>(); val skinB = ArrayList<Float>()
        val labTmp = FloatArray(3)
        val hasRealGrid = grid.width >= 16 && grid.height >= 16 && grid.subjectWeights.size == grid.width * grid.height

        var i = 0
        while (i < total) {
            val c = pixels[i]
            val r = ((c ushr 16) and 0xFF) / 255f
            val g = ((c ushr 8) and 0xFF) / 255f
            val b = (c and 0xFF) / 255f
            allR[allN] = r; allG[allN] = g; allB[allN] = b; allN++

            if (hasRealGrid && w > 0 && h > 0) {
                val py = i / w
                val px = i - py * w
                val nx = px.toFloat() / max(1, w - 1)
                val ny = py.toFloat() / max(1, h - 1)
                val sw = sampleBilinear(grid.subjectWeights, grid.width, grid.height, nx, ny)
                val sk = sampleBilinear(grid.skinWeights, grid.width, grid.height, nx, ny)
                if (sw >= 0.55f) {
                    subjR[subjN] = r; subjG[subjN] = g; subjB[subjN] = b; subjN++
                } else if (sw <= 0.35f) {
                    bgR[bgN] = r; bgG[bgN] = g; bgB[bgN] = b; bgN++
                }
                if (sk >= 0.45f) {
                    srgbToOklab(r, g, b, labTmp)
                    skinL.add(labTmp[0]); skinA.add(labTmp[1]); skinB.add(labTmp[2])
                }
            }
            i += step
        }

        val skinMedian = if (skinL.size >= 12) {
            skinL.sort(); skinA.sort(); skinB.sort()
            val m = skinL.size / 2
            floatArrayOf(skinL[m], skinA[m], skinB[m])
        } else null

        return SemanticSampleBundle(
            all = RegionSamples(allR, allG, allB, allN),
            subj = RegionSamples(subjR, subjG, subjB, subjN),
            bg = RegionSamples(bgR, bgG, bgB, bgN),
            faceSkinMedianLab = skinMedian
        )
    }

    private fun finalizeZoneStats(
        wSum: Double,
        aSum: Double,
        bSum: Double,
        aaSum: Double,
        abSum: Double,
        bbSum: Double,
        aSamples: ArrayList<Float>,
        bSamples: ArrayList<Float>,
        wSamples: ArrayList<Float>,
        useMedian: Boolean
    ): FloatArray {
        if (wSum <= 1e-4) return floatArrayOf(0f, 0f, 0.0005f, 0f, 0.0005f)
        val rawMa = (aSum / wSum).toFloat()
        val rawMb = (bSum / wSum).toFloat()
        val ma = if (useMedian && aSamples.isNotEmpty()) weightedMedian(aSamples, wSamples) * 0.90f + rawMa * 0.10f else rawMa
        val mb = if (useMedian && bSamples.isNotEmpty()) weightedMedian(bSamples, wSamples) * 0.90f + rawMb * 0.10f else rawMb
        return floatArrayOf(
            ma,
            mb,
            max(1e-7f, (aaSum / wSum).toFloat() - rawMa * rawMa),
            (abSum / wSum).toFloat() - rawMa * rawMb,
            max(1e-7f, (bbSum / wSum).toFloat() - rawMb * rawMb)
        )
    }

    private fun buildMatchLutForRole(
        refSamples: RegionSamples,
        tgtSamples: RegionSamples,
        refSkinAnchor: FloatArray?,
        tgtSkinAnchor: FloatArray?,
        forceRawFlat: Boolean?,
        role: RegionRole
    ): Pair<FloatArray, Boolean> {
        val refN = refSamples.size
        val tgtN = tgtSamples.size
        if (refN == 0 || tgtN == 0) return FloatArray(0) to false

        val refLabs = FloatArray(refN * 3)
        val refLList = FloatArray(refN)
        val refDarkChromas = ArrayList<Float>()
        val refSkinL = ArrayList<Float>()
        var refSkinASum = 0.0
        var refSkinBSum = 0.0
        val labTmp = FloatArray(3)

        for (p in 0 until refN) {
            val r = refSamples.r[p]; val g = refSamples.g[p]; val b = refSamples.b[p]
            srgbToOklab(r, g, b, labTmp)
            val l = labTmp[0]; val a = labTmp[1]; val okb = labTmp[2]
            refLabs[p * 3] = l; refLabs[p * 3 + 1] = a; refLabs[p * 3 + 2] = okb
            refLList[p] = l
            if (l < 0.30f) refDarkChromas.add(sqrt(a * a + okb * okb))
            if (isSkinSrgb(r, g, b, false)) {
                refSkinL.add(l)
                refSkinASum += a; refSkinBSum += okb
            }
        }
        val refLSorted = refLList.apply { sort() }
        refDarkChromas.sort()
        val refDarkChromaMed = if (refDarkChromas.isNotEmpty()) refDarkChromas[refDarkChromas.size / 2] else 0.02f

        val tgtLList = FloatArray(tgtN)
        var tgtChromaSqSum = 0.0
        val tgtLabs = FloatArray(tgtN * 3)
        for (p in 0 until tgtN) {
            val r = tgtSamples.r[p]; val g = tgtSamples.g[p]; val b = tgtSamples.b[p]
            srgbToOklab(r, g, b, labTmp)
            val l = labTmp[0]; val a = labTmp[1]; val okb = labTmp[2]
            tgtLabs[p * 3] = l; tgtLabs[p * 3 + 1] = a; tgtLabs[p * 3 + 2] = okb
            tgtLList[p] = l
            tgtChromaSqSum += a * a + okb * okb
        }
        val tgtLSorted = tgtLList.apply { sort() }
        val tgtChromaRms = sqrt(tgtChromaSqSum / max(1, tgtN)).toFloat()

        val tgtP05 = quantileFromSorted(tgtLSorted, 0.05f)
        val tgtP95 = quantileFromSorted(tgtLSorted, 0.95f)
        val tgtP99 = quantileFromSorted(tgtLSorted, 0.99f)
        val tgtSpread = max(0.02f, tgtP95 - tgtP05)

        val autoIsLogFlat = tgtChromaRms < 0.025f ||
            (tgtSpread < 0.46f && tgtP99 < 0.84f) ||
            (tgtP05 > 0.28f && tgtSpread < 0.55f)
        val isFlat = forceRawFlat ?: autoIsLogFlat
        val logChromaNeed = if (isFlat) 1f - smoothstep(0.018f, 0.048f, tgtChromaRms) else 0f

        val tgtSkinL = ArrayList<Float>()
        for (p in 0 until tgtN) {
            if (isSkinSrgb(tgtSamples.r[p], tgtSamples.g[p], tgtSamples.b[p], isFlat)) {
                tgtSkinL.add(tgtLabs[p * 3])
            }
        }

        // Step 1: 256-entry Role-Aware Luminance Curve
        val lBins = 256
        val lRaw = FloatArray(lBins)
        val t005 = quantileFromSorted(tgtLSorted, 0.005f)
        val t01 = quantileFromSorted(tgtLSorted, 0.01f)
        val t50 = quantileFromSorted(tgtLSorted, 0.50f)
        val t95 = tgtP95
        val t99 = tgtP99
        val r01 = quantileFromSorted(refLSorted, 0.01f)
        val r05 = quantileFromSorted(refLSorted, 0.05f)
        val r50 = quantileFromSorted(refLSorted, 0.50f)
        val r92 = quantileFromSorted(refLSorted, 0.92f)
        val r95 = quantileFromSorted(refLSorted, 0.95f)
        val r99 = quantileFromSorted(refLSorted, 0.99f)
        val refSpread = max(0.02f, r95 - r05)
        val spanT = max(0.08f, quantileFromSorted(tgtLSorted, 0.995f) - t005)

        val preserveSubjectWhites = role == RegionRole.SUBJECT && !autoIsLogFlat && t95 > r95 + 0.08f && t95 > 0.78f
        val uniformBgBackdrop = role == RegionRole.BACKGROUND && refSpread < 0.20f

        for (bin in 0 until lBins) {
            val x = bin.toFloat() / (lBins - 1)
            val q = cdfRank(tgtLSorted, x)
            val lNorm = ((x - t005) / spanT).coerceIn(0f, 1f)
            val lCdf = when {
                x < t01 -> {
                    val k = (x / max(0.015f, t01)).coerceIn(0f, 1f)
                    val kSmooth = k * k * (3f - 2f * k)
                    val targetLo = min(0.04f, r01 * 0.65f)
                    targetLo + (r01 - targetLo) * kSmooth
                }
                x > t99 -> {
                    val k = ((x - t99) / max(0.015f, 1f - t99)).coerceIn(0f, 1f)
                    val kSmooth = k * k * (3f - 2f * k)
                    val targetHi = max(0.92f, max(r99, t99))
                    r99 + (targetHi - r99) * kSmooth
                }
                else -> quantileFromSorted(refLSorted, q)
            }

            var lGoal = when {
                uniformBgBackdrop -> {
                    val bell = smoothstep(0.04f, 0.22f, x) * (1f - smoothstep(0.58f, 0.90f, x))
                    val delta = (r50 - t50) * 0.92f * bell
                    (x + delta).coerceIn(0.02f, 0.98f)
                }
                preserveSubjectWhites && x > 0.56f -> {
                    val wHi = smoothstep(0.56f, 0.90f, x)
                    val keptHi = max(x, lCdf)
                    lCdf + (keptHi - lCdf) * (0.85f * wHi)
                }
                !isFlat -> {
                    val hiW = 0.42f * q * q * q
                    val airy = max(lCdf, lNorm * 0.95f + 0.04f)
                    lCdf + (airy - lCdf) * hiW
                }
                r92 < 0.62f -> {
                    val hiW = 0.36f * q * q
                    val airy = max(lCdf, lNorm * 0.78f + 0.05f)
                    lCdf + (airy - lCdf) * hiW
                }
                lCdf > 0.70f -> {
                    val over = lCdf - 0.70f
                    0.70f + over / (1f + over * 1.95f)
                }
                else -> lCdf
            }

            if (role != RegionRole.BACKGROUND && !autoIsLogFlat && !isFlat && lGoal > 0.78f) {
                lGoal = max(lGoal, x * 0.94f)
            }

            val wTone = when {
                uniformBgBackdrop -> 0.90f
                x < t01 && autoIsLogFlat -> 1f
                isFlat -> if (autoIsLogFlat) 0.99f else 0.88f
                else -> 0.92f
            }
            lRaw[bin] = x + (lGoal - x) * wTone
        }

        if (role == RegionRole.SUBJECT && refSkinAnchor != null && tgtSkinAnchor != null && !autoIsLogFlat) {
            val tgtSkL = tgtSkinAnchor[0].coerceIn(0.25f, 0.85f)
            val refSkGoal = refSkinAnchor[0].coerceIn(0.45f, 0.78f)
            val idx = (tgtSkL * (lBins - 1)).roundToInt().coerceIn(0, lBins - 1)
            val lift = refSkGoal - lRaw[idx]
            if (kotlin.math.abs(lift) > 0.01f) {
                for (bin in 0 until lBins) {
                    val x = bin.toFloat() / (lBins - 1)
                    val dx = (x - tgtSkL) / 0.14f
                    val w = exp(-(dx * dx))
                    lRaw[bin] = (lRaw[bin] + lift * w * 0.58f).coerceIn(0.02f, 0.96f)
                }
            }
        } else if (tgtSkinL.size >= 20) {
            tgtSkinL.sort()
            val tSkinMed = tgtSkinL[tgtSkinL.size / 2]
            val idx = (tSkinMed * (lBins - 1)).roundToInt().coerceIn(0, lBins - 1)
            if (lRaw[idx] < 0.34f) {
                val lift = 0.34f - lRaw[idx]
                for (bin in 0 until lBins) {
                    val x = bin.toFloat() / (lBins - 1)
                    val dx = (x - tSkinMed) / 0.14f
                    val w = exp(-(dx * dx))
                    lRaw[bin] = min(0.86f, lRaw[bin] + lift * w * 0.68f)
                }
            }
        }

        val lMap = FloatArray(lBins)
        val gSigma = 5.5f
        val gRad = 14
        for (bin in 0 until lBins) {
            var wSum = 0f; var vSum = 0f
            for (d in -gRad..gRad) {
                val j = (bin + d).coerceIn(0, lBins - 1)
                val x = d / gSigma
                val w = exp(-(x * x) * 0.5f)
                wSum += w; vSum += w * lRaw[j]
            }
            lMap[bin] = vSum / wSum
        }

        val minStep = 0.04f / (lBins - 1)
        val maxStep = (if (isFlat) 4.8f else 3.2f) / (lBins - 1)
        val startLo = (r05 * 0.45f).coerceIn(0f, 0.09f)
        lMap[0] = lMap[0].coerceIn(0f, max(0.03f, startLo))
        for (bin in 1 until lBins) {
            val hi = min(1f, lMap[bin - 1] + maxStep)
            val lo = min(hi, lMap[bin - 1] + minStep)
            lMap[bin] = lMap[bin].coerceIn(lo, hi)
        }
        lMap[lBins - 1] = if (t005 > 0.15f) lMap[lBins - 1].coerceIn(0.78f, 1f) else lMap[lBins - 1].coerceIn(0.86f, 1f)

        fun evalLMap(v: Float): Float {
            val f = (v.coerceIn(0f, 1f) * (lBins - 1)).coerceIn(0f, (lBins - 1).toFloat())
            val i0 = min(lBins - 1, floor(f).toInt())
            val i1 = min(lBins - 1, i0 + 1)
            val frac = f - i0
            return lMap[i0] * (1f - frac) + lMap[i1] * frac
        }

        // Step 2: 8-Zone Weighted-Median & Monge-Kantorovich 2x2 Covariance Transfer
        val zones = 8
        val centers = floatArrayOf(0.06f, 0.18f, 0.30f, 0.42f, 0.54f, 0.66f, 0.78f, 0.90f)
        val zoneSigma = 0.13f

        val tW = DoubleArray(zones); val tA = DoubleArray(zones); val tB = DoubleArray(zones)
        val tAA = DoubleArray(zones); val tAB = DoubleArray(zones); val tBB = DoubleArray(zones)
        val tASamples = Array(zones) { ArrayList<Float>() }
        val tBSamples = Array(zones) { ArrayList<Float>() }
        val tWSamples = Array(zones) { ArrayList<Float>() }

        val rW = DoubleArray(zones); val rA = DoubleArray(zones); val rB = DoubleArray(zones)
        val rAA = DoubleArray(zones); val rAB = DoubleArray(zones); val rBB = DoubleArray(zones)
        val rASamples = Array(zones) { ArrayList<Float>() }
        val rBSamples = Array(zones) { ArrayList<Float>() }
        val rWSamples = Array(zones) { ArrayList<Float>() }

        for (p in 0 until tgtN) {
            val lEff = evalLMap(tgtLabs[p * 3])
            val af = tgtLabs[p * 3 + 1]; val bf = tgtLabs[p * 3 + 2]
            val a = af.toDouble(); val b = bf.toDouble()
            for (z in 0 until zones) {
                val dx = (lEff - centers[z]) / zoneSigma
                val wf = exp(-(dx * dx))
                val w = wf.toDouble()
                if (w > 0.008) {
                    tW[z] += w; tA[z] += w * a; tB[z] += w * b
                    tAA[z] += w * a * a; tAB[z] += w * a * b; tBB[z] += w * b * b
                    tASamples[z].add(af); tBSamples[z].add(bf); tWSamples[z].add(wf)
                }
            }
        }
        for (p in 0 until refN) {
            val l = refLabs[p * 3]
            val af = refLabs[p * 3 + 1]; val bf = refLabs[p * 3 + 2]
            val a = af.toDouble(); val b = bf.toDouble()
            for (z in 0 until zones) {
                val dx = (l - centers[z]) / zoneSigma
                val wf = exp(-(dx * dx))
                val w = wf.toDouble()
                if (w > 0.008) {
                    rW[z] += w; rA[z] += w * a; rB[z] += w * b
                    rAA[z] += w * a * a; rAB[z] += w * a * b; rBB[z] += w * b * b
                    rASamples[z].add(af); rBSamples[z].add(bf); rWSamples[z].add(wf)
                }
            }
        }

        val maxGain = when {
            logChromaNeed > 0.3f -> 1.55f + 0.30f * logChromaNeed
            role == RegionRole.BACKGROUND -> 1.45f
            else -> 1.40f
        }
        val minGain = 0.38f
        val muTa = FloatArray(zones); val muTb = FloatArray(zones)
        val muRa = FloatArray(zones); val muRb = FloatArray(zones)
        val tMats = Array(zones) { Mat2(1f, 0f, 0f, 1f) }

        for (z in 0 until zones) {
            val tz = finalizeZoneStats(
                tW[z], tA[z], tB[z], tAA[z], tAB[z], tBB[z],
                tASamples[z], tBSamples[z], tWSamples[z],
                useMedian = (role == RegionRole.BACKGROUND)
            )
            val rz = finalizeZoneStats(
                rW[z], rA[z], rB[z], rAA[z], rAB[z], rBB[z],
                rASamples[z], rBSamples[z], rWSamples[z],
                useMedian = true
            )
            muTa[z] = tz[0]; muTb[z] = tz[1]
            muRa[z] = rz[0]; muRb[z] = rz[1]
            val epsT = if (logChromaNeed > 0.3f) max(1.2e-4f, (tz[2] + tz[4]) * 0.30f) else max(1.6e-4f, (tz[2] + tz[4]) * 0.42f)
            val epsR = max(6.0e-5f, (rz[2] + rz[4]) * 0.14f)
            tMats[z] = computeMk2x2(tz[2], tz[3], tz[4], rz[2], rz[3], rz[4], epsT, epsR, minGain, maxGain)
        }

        val isCleanNeutralSubject = role == RegionRole.SUBJECT && refDarkChromaMed < 0.024f && logChromaNeed < 0.3f
        val hasTgtSkin = tgtSkinAnchor != null || tgtSkinL.size >= 20
        val refSkinMean: Pair<Float, Float>? = when {
            refSkinAnchor != null -> refSkinAnchor[1] to refSkinAnchor[2]
            refSkinL.size >= 20 -> (refSkinASum / refSkinL.size).toFloat() to (refSkinBSum / refSkinL.size).toFloat()
            else -> null
        }

        // Step 3: Populate 25x25x25 3D LUT
        val dim = MATCH_LUT_DIM
        val lut = FloatArray(dim * dim * dim * 3)
        val denom = (dim - 1).toFloat()
        val d2 = dim * dim
        val linTmp = FloatArray(3)
        val srgbOutTmp = FloatArray(3)
        val wZ = FloatArray(zones)
        val bloom = 1f + 0.55f * logChromaNeed + if (!isFlat && logChromaNeed > 0.2f) 0.20f else 0f
        val passthroughK = if (logChromaNeed > 0.3f) 0.28f else 0.88f

        for (bi in 0 until dim) {
            val b = bi / denom
            for (gi in 0 until dim) {
                val g = gi / denom
                for (ri in 0 until dim) {
                    val r = ri / denom
                    srgbToOklab(r, g, b, labTmp)
                    val lIn = labTmp[0]; val aIn = labTmp[1]; val bIn = labTmp[2]
                    val lOut = evalLMap(lIn)
                    val cIn = sqrt(aIn * aIn + bIn * bIn)

                    var wSum = 0f
                    for (z in 0 until zones) {
                        val dx = (lOut - centers[z]) / zoneSigma
                        val w = exp(-(dx * dx))
                        wZ[z] = w; wSum += w
                    }
                    val invW = if (wSum > 1e-6f) 1f / wSum else 1f / zones

                    val shadowClean = 0.50f + 0.50f * smoothstep(0.04f, 0.24f, lOut)
                    val hiClean = 1f - 0.50f * smoothstep(0.62f, 0.92f, lOut)
                    val zoneCastScale = if (role == RegionRole.BACKGROUND) 1f else shadowClean * hiClean
                    val sigC = smoothstep(0.0085f, 0.017f, cIn)

                    var aMk = 0f; var bMk = 0f
                    for (z in 0 until zones) {
                        val wz = wZ[z] * invW
                        var maR = muRa[z]; var mbR = muRb[z]
                        val taCenter = muTa[z]; val tbCenter = muTb[z]

                        if (isCleanNeutralSubject) {
                            val neutralProtect = 1f - smoothstep(0.015f, 0.055f, cIn)
                            val whiteGuard = smoothstep(0.68f, 0.90f, lOut)
                            val darkGuard = 1f - smoothstep(0.08f, 0.34f, lOut)
                            val keepGrade = 1f - max(0.82f * neutralProtect, max(0.86f * whiteGuard, 0.88f * darkGuard))
                            maR *= keepGrade
                            mbR *= keepGrade
                        }

                        val da = aIn - taCenter
                        val db = bIn - tbCenter
                        val dc = sqrt(da * da + db * db)
                        val mat = tMats[z]
                        val mkA = mat.m00 * da + mat.m01 * db
                        val mkB = mat.m10 * da + mat.m11 * db
                        val signalT = sigC * smoothstep(0.006f, 0.015f, dc)
                        val ta = (da * passthroughK) * (1f - signalT) + (mkA * bloom) * signalT
                        val tb = (db * passthroughK) * (1f - signalT) + (mkB * bloom) * signalT

                        var zoneA = maR * zoneCastScale + ta
                        var zoneB = mbR * zoneCastScale + tb

                        if (role == RegionRole.BACKGROUND) {
                            val muC = max(
                                sqrt(maR * maR + mbR * mbR),
                                sqrt(taCenter * taCenter + tbCenter * tbCenter)
                            )
                            if (muC > 0.07f) {
                                val bgChromaT = smoothstep(0.025f, 0.085f, cIn) * (1f - smoothstep(0.72f, 0.92f, lOut))
                                zoneA = aIn + (zoneA - aIn) * bgChromaT
                                zoneB = bIn + (zoneB - bIn) * bgChromaT
                            }
                        }

                        aMk += wz * zoneA
                        bMk += wz * zoneB
                    }

                    if (lOut > 0.76f && cIn < 0.035f) {
                        val hiGuard = smoothstep(0.76f, 0.95f, lOut) * (1f - smoothstep(0.01f, 0.035f, cIn))
                        aMk *= 1f - hiGuard * 0.60f
                        bMk *= 1f - hiGuard * 0.60f
                    }

                    var aOut = aMk; var bOut = bMk
                    if (hasTgtSkin && aIn > 0.0068f && bIn > -0.001f && lIn > 0.32f && lIn < 0.96f) {
                        val angIn = atan2(bIn, aIn)
                        if (angIn > 0.12f && angIn < 0.96f) {
                            val wSk = smoothstep(0.0068f, 0.014f, aIn) *
                                smoothstep(0.32f, 0.50f, lIn) *
                                (1f - smoothstep(0.88f, 0.97f, lIn)) *
                                (1f - smoothstep(0.72f, 0.96f, angIn))
                            if (wSk > 0.02f) {
                                if (role == RegionRole.SUBJECT && refSkinAnchor != null && tgtSkinAnchor != null) {
                                    val ra = refSkinAnchor[1]; val rb = refSkinAnchor[2]
                                    val taSk = tgtSkinAnchor[1]; val tbSk = tgtSkinAnchor[2]
                                    val distSk = sqrt((aIn - taSk) * (aIn - taSk) + (bIn - tbSk) * (bIn - tbSk))
                                    val wDirect = wSk * (1f - smoothstep(0.025f, 0.085f, distSk))
                                    val goalA = (aIn + (ra - taSk) * 0.85f).coerceIn(0.018f, 0.075f)
                                    val goalB = (bIn + (rb - tbSk) * 0.85f).coerceIn(0.008f, 0.095f)
                                    aOut += (goalA - aOut) * wDirect * 0.82f
                                    bOut += (goalB - bOut) * wDirect * 0.82f
                                }
                                val minA = refSkinMean?.first?.coerceIn(0.026f, 0.044f) ?: 0.030f
                                if (aOut < minA) aOut += (minA - aOut) * wSk * 0.85f
                                if (bOut < 0.008f) bOut += (0.011f - bOut) * wSk * 0.75f
                                val cSk = sqrt(aOut * aOut + bOut * bOut)
                                if (cSk > 0.100f) {
                                    val sc = 0.100f / cSk
                                    aOut = aOut * (1f - wSk) + aOut * sc * wSk
                                    bOut = bOut * (1f - wSk) + bOut * sc * wSk
                                }
                            }
                        }
                    }

                    oklabToSrgbGamut(lOut, aOut, bOut, linTmp, srgbOutTmp)
                    val outIdx = (bi * d2 + gi * dim + ri) * 3
                    lut[outIdx] = srgbOutTmp[0]
                    lut[outIdx + 1] = srgbOutTmp[1]
                    lut[outIdx + 2] = srgbOutTmp[2]
                }
            }
        }
        return lut to isFlat
    }

    private fun buildDualMatchLutsFromPixels(
        refPixels: IntArray,
        refW: Int,
        refH: Int,
        refFaces: List<FaceRect>,
        tgtPixels: IntArray,
        tgtW: Int,
        tgtH: Int,
        tgtFaces: List<FaceRect>,
        forceRawFlat: Boolean?
    ): MatchDualLutOutput {
        val refSem = extractSemanticSamples(refPixels, refW, refH, refFaces)
        val tgtSem = extractSemanticSamples(tgtPixels, tgtW, tgtH, tgtFaces)

        val refLabs = FloatArray(refSem.all.size * 3)
        val labTmp = FloatArray(3)
        for (p in 0 until refSem.all.size) {
            srgbToOklab(refSem.all.r[p], refSem.all.g[p], refSem.all.b[p], labTmp)
            refLabs[p * 3] = labTmp[0]; refLabs[p * 3 + 1] = labTmp[1]; refLabs[p * 3 + 2] = labTmp[2]
        }
        val palette = extractPaletteFromLabs(refLabs, refSem.all.size, 10)

        val hasSemanticSplit = refSem.bg.size >= 160 &&
            refSem.subj.size >= 120 &&
            tgtSem.bg.size >= 160 &&
            tgtSem.subj.size >= 120

        var mainLut: FloatArray
        var bgLut: FloatArray
        val isFlat: Boolean

        if (hasSemanticSplit) {
            val (sLut, sFlat) = buildMatchLutForRole(
                refSem.subj, tgtSem.subj,
                refSem.faceSkinMedianLab, tgtSem.faceSkinMedianLab,
                forceRawFlat, RegionRole.SUBJECT
            )
            val (bLut, _) = buildMatchLutForRole(
                refSem.bg, tgtSem.bg,
                null, null,
                SomeFlatFallback(forceRawFlat, sFlat), RegionRole.BACKGROUND
            )
            mainLut = sLut
            bgLut = bLut
            isFlat = sFlat
        } else {
            val (uLut, uFlat) = buildMatchLutForRole(
                refSem.all, tgtSem.all,
                refSem.faceSkinMedianLab, tgtSem.faceSkinMedianLab,
                forceRawFlat, RegionRole.UNIFIED
            )
            mainLut = uLut
            bgLut = FloatArray(0)
            isFlat = uFlat
        }

        val expectedLen = MATCH_LUT_DIM * MATCH_LUT_DIM * MATCH_LUT_DIM * 3
        if (hasSemanticSplit && mainLut.size == expectedLen && bgLut.size == expectedLen) {
            var refBgCSum = 0.0
            for (p in 0 until refSem.bg.size) {
                srgbToOklab(refSem.bg.r[p], refSem.bg.g[p], refSem.bg.b[p], labTmp)
                refBgCSum += sqrt(labTmp[1] * labTmp[1] + labTmp[2] * labTmp[2])
            }
            var tgtBgASum = 0.0; var tgtBgBSum = 0.0; var tgtBgCSum = 0.0
            for (p in 0 until tgtSem.bg.size) {
                srgbToOklab(tgtSem.bg.r[p], tgtSem.bg.g[p], tgtSem.bg.b[p], labTmp)
                tgtBgASum += labTmp[1]; tgtBgBSum += labTmp[2]
                tgtBgCSum += sqrt(labTmp[1] * labTmp[1] + labTmp[2] * labTmp[2])
            }
            val meanCBgR = (refBgCSum / max(1, refSem.bg.size)).toFloat()
            val meanCBgT = (tgtBgCSum / max(1, tgtSem.bg.size)).toFloat()
            if (meanCBgR > 0.10f && meanCBgT > 0.10f) {
                val dirA = (tgtBgASum / max(1, tgtSem.bg.size)).toFloat()
                val dirB = (tgtBgBSum / max(1, tgtSem.bg.size)).toFloat()
                val dirLen = max(1e-5f, sqrt(dirA * dirA + dirB * dirB))
                val ua = dirA / dirLen
                val ub = dirB / dirLen
                val d = MATCH_LUT_DIM
                val denom = (d - 1).toFloat()
                for (bi in 0 until d) {
                    val bf = bi / denom
                    for (gi in 0 until d) {
                        val gf = gi / denom
                        for (ri in 0 until d) {
                            val rf = ri / denom
                            srgbToOklab(rf, gf, bf, labTmp)
                            val proj = labTmp[1] * ua + labTmp[2] * ub
                            if (proj > 0.055f) {
                                val wWall = smoothstep(0.055f, 0.13f, proj)
                                val idx = (bi * d * d + gi * d + ri) * 3
                                for (c in 0 until 3) {
                                    mainLut[idx + c] += (bgLut[idx + c] - mainLut[idx + c]) * wWall
                                }
                            }
                        }
                    }
                }
            }
        }

        return MatchDualLutOutput(mainLut, bgLut, palette, isFlat)
    }

    private fun SomeFlatFallback(forceRawFlat: Boolean?, subjectFlat: Boolean): Boolean =
        forceRawFlat ?: subjectFlat

    private fun inferGridDims(size: Int): Pair<Int, Int> {
        if (size <= 0) return 0 to 0
        val root = floor(sqrt(size.toDouble())).toInt().coerceAtLeast(1)
        for (h in root downTo 1) {
            if (size % h == 0) return (size / h) to h
        }
        return size to 1
    }

    /** Pure Kotlin pixel-array entry point with explicit dimensions (JVM unit-testable). */
    fun matchPixelsToReference(
        refPixels: IntArray,
        refW: Int,
        refH: Int,
        targetPixels: IntArray,
        tgtW: Int,
        tgtH: Int,
        targetParams: DevelopParams,
        forceRawFlat: Boolean? = null,
        refName: String = "Reference"
    ): DevelopParams {
        val out = buildDualMatchLutsFromPixels(
            refPixels, refW, refH, emptyList(),
            targetPixels, tgtW, tgtH, emptyList(),
            forceRawFlat
        )
        val expectedLen = MATCH_LUT_DIM * MATCH_LUT_DIM * MATCH_LUT_DIM * 3
        val lutOk = out.mainLut.size == expectedLen
        val bgOk = lutOk && out.bgLut.size == expectedLen

        return targetParams.copy(
            matchLutDim = if (lutOk) MATCH_LUT_DIM else 0,
            matchLutData = if (lutOk) out.mainLut else FloatArray(0),
            matchBgLutData = if (bgOk) out.bgLut else FloatArray(0),
            matchToneAmount = if (lutOk && targetParams.hasColorMatchLut()) targetParams.matchToneAmount.coerceIn(0f, 100f) else 100f,
            matchColorAmount = if (lutOk && targetParams.hasColorMatchLut()) targetParams.matchColorAmount.coerceIn(0f, 100f) else 100f,
            matchRawFlat = out.isFlat,
            matchPalette = out.palette,
            matchRefName = refName
        )
    }

    /** Pure Kotlin pixel-array entry point (JVM unit-testable without Android Bitmap dependency). */
    fun matchPixelsToReference(
        refPixels: IntArray,
        targetPixels: IntArray,
        targetParams: DevelopParams,
        forceRawFlat: Boolean? = null,
        refName: String = "Reference"
    ): DevelopParams {
        val (rw, rh) = inferGridDims(refPixels.size)
        val (tw, th) = inferGridDims(targetPixels.size)
        return matchPixelsToReference(
            refPixels, rw, rh,
            targetPixels, tw, th,
            targetParams, forceRawFlat, refName
        )
    }

    fun matchToReference(
        refBitmap: Bitmap,
        targetBitmap: Bitmap,
        refParams: DevelopParams?,
        targetParams: DevelopParams,
        forceRawFlat: Boolean? = null,
        refName: String = "Reference"
    ): DevelopParams {
        val refSmall = scaleDownForMatch(refBitmap, 160)
        val tgtSmall = scaleDownForMatch(targetBitmap, 160)

        val renderedRef = if (refParams != null) {
            KotlinPv2012Engine.render(
                refSmall,
                refParams.clearColorMatch(),
                draftInteractive = true,
                applyCrop = false
            ).bitmap
        } else {
            refSmall
        }
        val refPx = IntArray(renderedRef.width * renderedRef.height).also {
            renderedRef.getPixels(it, 0, renderedRef.width, 0, 0, renderedRef.width, renderedRef.height)
        }
        val refFaces = detectFaceRects(renderedRef)

        val baseOnlyParams = targetParams.clearColorMatch().copy(vignetteAmount = 0f, grainAmount = 0f)
        val renderedTgt = KotlinPv2012Engine.render(
            tgtSmall,
            baseOnlyParams,
            draftInteractive = true,
            applyCrop = false
        ).bitmap
        val renderedTgtPx = IntArray(renderedTgt.width * renderedTgt.height).also {
            renderedTgt.getPixels(it, 0, renderedTgt.width, 0, 0, renderedTgt.width, renderedTgt.height)
        }
        val tgtFaces = detectFaceRects(renderedTgt)

        val out = buildDualMatchLutsFromPixels(
            refPx, renderedRef.width, renderedRef.height, refFaces,
            renderedTgtPx, renderedTgt.width, renderedTgt.height, tgtFaces,
            forceRawFlat
        )
        if (renderedRef !== refSmall && !renderedRef.isRecycled) renderedRef.recycle()
        if (refSmall !== refBitmap && !refSmall.isRecycled) refSmall.recycle()
        if (!renderedTgt.isRecycled) renderedTgt.recycle()
        if (tgtSmall !== targetBitmap && !tgtSmall.isRecycled) tgtSmall.recycle()

        val expectedLen = MATCH_LUT_DIM * MATCH_LUT_DIM * MATCH_LUT_DIM * 3
        val lutOk = out.mainLut.size == expectedLen
        val bgOk = lutOk && out.bgLut.size == expectedLen
        return targetParams.copy(
            clarity = if (refParams != null) (targetParams.clarity * 0.5f + refParams.clarity * 0.5f).coerceIn(-100f, 100f) else targetParams.clarity,
            dehaze = if (refParams != null) (targetParams.dehaze * 0.5f + refParams.dehaze * 0.5f).coerceIn(-100f, 100f) else targetParams.dehaze,
            matchLutDim = if (lutOk) MATCH_LUT_DIM else 0,
            matchLutData = if (lutOk) out.mainLut else FloatArray(0),
            matchBgLutData = if (bgOk) out.bgLut else FloatArray(0),
            matchToneAmount = if (lutOk && targetParams.hasColorMatchLut()) targetParams.matchToneAmount.coerceIn(0f, 100f) else 100f,
            matchColorAmount = if (lutOk && targetParams.hasColorMatchLut()) targetParams.matchColorAmount.coerceIn(0f, 100f) else 100f,
            matchRawFlat = out.isFlat,
            matchPalette = out.palette,
            matchRefName = refName
        )
    }

    private fun scaleDownForMatch(bmp: Bitmap, maxDim: Int): Bitmap {
        val cur = max(bmp.width, bmp.height)
        if (cur <= maxDim || cur == 0) return bmp
        val s = maxDim.toFloat() / cur.toFloat()
        val w = max(16, (bmp.width * s).roundToInt())
        val h = max(16, (bmp.height * s).roundToInt())
        return Bitmap.createScaledBitmap(bmp, w, h, true)
    }
}
