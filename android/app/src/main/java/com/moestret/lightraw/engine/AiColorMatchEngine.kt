package com.moestret.lightraw.engine

import android.graphics.Bitmap
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.WhiteBalanceMode
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
 * Computes perceptual OKLab Luminance-CDF + 5-zone 2x2 Monge-Kantorovich color transfer
 * encoded as a smooth 17x17x17 3D LUT with live `Tonal %` & `Warna %` sliders and skin protection.
 */
object AiColorMatchEngine {

    const val MATCH_LUT_DIM = 17

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
        val wt = (toneAmountPct / 100f).coerceIn(0f, 1f)
        val wc = (colorAmountPct / 100f).coerceIn(0f, 1f)
        if ((wt <= 1e-4f && wc <= 1e-4f) || dim < 2 || lut.size != dim * dim * dim * 3) {
            outRgb[0] = r; outRgb[1] = g; outRgb[2] = b
            return
        }
        sampleLut3d(lut, dim, r, g, b, tmpLutRgb)
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

    private fun computeMk2x2(
        taa: Float, tab: Float, tbb: Float,
        raa: Float, rab: Float, rbb: Float,
        eps: Float, maxGain: Float
    ): Mat2 {
        val et = eigenSym2x2(taa + eps, tab, tbb + eps)
        val sqrtT = reconstructSym2x2(et[0], et[1], sqrt(et[2]), sqrt(et[3]))
        val invSqrtT = reconstructSym2x2(et[0], et[1], 1f / sqrt(et[2]), 1f / sqrt(et[3]))
        val sigR = Mat2(raa + eps, rab, rab, rbb + eps)
        val inner = sqrtT.mul(sigR).mul(sqrtT)
        val ep = eigenSym2x2(inner.m00, 0.5f * (inner.m01 + inner.m10), inner.m11)
        val sqrtInner = reconstructSym2x2(ep[0], ep[1], sqrt(ep[2]), sqrt(ep[3]))
        val tMat = invSqrtT.mul(sqrtInner).mul(invSqrtT)
        val em = eigenSym2x2(tMat.m00, 0.5f * (tMat.m01 + tMat.m10), tMat.m11)
        return reconstructSym2x2(em[0], em[1], em[2].coerceIn(0.35f, maxGain), em[3].coerceIn(0.35f, maxGain))
    }

    private fun buildMatchLutFromPixels(
        refPixels: IntArray,
        tgtPixels: IntArray,
        forceRawFlat: Boolean?
    ): Triple<FloatArray, IntArray, Boolean> {
        val stepR = max(1, refPixels.size / 12000)
        val stepT = max(1, tgtPixels.size / 12000)

        val refCountMax = refPixels.size / stepR + 1
        val tgtCountMax = tgtPixels.size / stepT + 1

        val refLabs = FloatArray(refCountMax * 3)
        val refLList = ArrayList<Float>(refCountMax)
        val refSkinL = ArrayList<Float>()
        var refSkinASum = 0.0
        var refSkinBSum = 0.0
        var refN = 0

        val labTmp = FloatArray(3)
        var i = 0
        while (i < refPixels.size) {
            val c = refPixels[i]
            val r = ((c ushr 16) and 0xFF) / 255f
            val g = ((c ushr 8) and 0xFF) / 255f
            val b = (c and 0xFF) / 255f
            srgbToOklab(r, g, b, labTmp)
            refLabs[refN * 3] = labTmp[0]
            refLabs[refN * 3 + 1] = labTmp[1]
            refLabs[refN * 3 + 2] = labTmp[2]
            refLList.add(labTmp[0])
            refN++
            if (isSkinSrgb(r, g, b, false)) {
                refSkinL.add(labTmp[0])
                refSkinASum += labTmp[1]
                refSkinBSum += labTmp[2]
            }
            i += stepR
        }
        val refLSorted = refLList.toFloatArray().apply { sort() }
        val palette = extractPaletteFromLabs(refLabs, refN, 10)

        val tgtLabs = FloatArray(tgtCountMax * 3)
        val tgtLList = ArrayList<Float>(tgtCountMax)
        var tgtN = 0
        i = 0
        while (i < tgtPixels.size) {
            val c = tgtPixels[i]
            val r = ((c ushr 16) and 0xFF) / 255f
            val g = ((c ushr 8) and 0xFF) / 255f
            val b = (c and 0xFF) / 255f
            srgbToOklab(r, g, b, labTmp)
            tgtLabs[tgtN * 3] = labTmp[0]
            tgtLabs[tgtN * 3 + 1] = labTmp[1]
            tgtLabs[tgtN * 3 + 2] = labTmp[2]
            tgtLList.add(labTmp[0])
            tgtN++
            i += stepT
        }
        val tgtLSorted = tgtLList.toFloatArray().apply { sort() }
        if (refLSorted.isEmpty() || tgtLSorted.isEmpty()) {
            return Triple(FloatArray(0), palette, false)
        }

        val tgtP05 = quantileFromSorted(tgtLSorted, 0.05f)
        val tgtP95 = quantileFromSorted(tgtLSorted, 0.95f)
        val tgtSpread = max(0.02f, tgtP95 - tgtP05)
        val isFlat = forceRawFlat ?: (tgtSpread < 0.48f || tgtP05 > 0.13f)

        val tgtSkinL = ArrayList<Float>()
        var tgtSkinASum = 0.0
        var tgtSkinBSum = 0.0
        var skIdx = 0
        i = 0
        while (i < tgtPixels.size && skIdx < tgtN) {
            val c = tgtPixels[i]
            val r = ((c ushr 16) and 0xFF) / 255f
            val g = ((c ushr 8) and 0xFF) / 255f
            val b = (c and 0xFF) / 255f
            if (isSkinSrgb(r, g, b, isFlat)) {
                tgtSkinL.add(tgtLabs[skIdx * 3])
                tgtSkinASum += tgtLabs[skIdx * 3 + 1]
                tgtSkinBSum += tgtLabs[skIdx * 3 + 2]
            }
            i += stepT
            skIdx++
        }

        // Step 1: 65-entry Smooth Luminance CDF Curve
        val lBins = 65
        val lMap = FloatArray(lBins)
        val t01 = quantileFromSorted(tgtLSorted, 0.01f)
        val t99 = quantileFromSorted(tgtLSorted, 0.99f)
        val r01 = quantileFromSorted(refLSorted, 0.01f)
        val r99 = quantileFromSorted(refLSorted, 0.99f)
        val cdfStrength = if (isFlat) 0.92f else 0.85f

        for (bin in 0 until lBins) {
            val x = bin.toFloat() / (lBins - 1)
            val q = cdfRank(tgtLSorted, x)
            val lRef = when {
                x < t01 -> {
                    val k = (x / max(0.02f, t01)).coerceIn(0f, 1f)
                    val targetLo = min(0.04f, r01 * 0.72f)
                    targetLo + (r01 - targetLo) * k
                }
                x > t99 -> {
                    val k = ((x - t99) / max(0.02f, 1f - t99)).coerceIn(0f, 1f)
                    val targetHi = max(0.90f, r99)
                    r99 + (targetHi - r99) * k
                }
                else -> quantileFromSorted(refLSorted, q)
            }
            lMap[bin] = x + (lRef - x) * cdfStrength
        }

        if (tgtSkinL.size >= 20) {
            tgtSkinL.sort()
            val tSkinMed = tgtSkinL[tgtSkinL.size / 2]
            val idx = (tSkinMed * (lBins - 1)).roundToInt().coerceIn(0, lBins - 1)
            val mappedSkin = lMap[idx]
            val floorSkin = if (refSkinL.size >= 20) {
                refSkinL.sort()
                val rSkinMed = refSkinL[refSkinL.size / 2]
                rSkinMed.coerceIn(0.44f, 0.80f) * 0.6f + tSkinMed.coerceIn(0.46f, 0.80f) * 0.4f
            } else {
                (tSkinMed * 0.82f).coerceIn(0.46f, 0.76f)
            }
            if (mappedSkin < floorSkin) {
                val lift = floorSkin - mappedSkin
                val ceilGuard = min(0.94f, max(floorSkin + 0.06f, r99))
                for (bin in 0 until lBins) {
                    val x = bin.toFloat() / (lBins - 1)
                    val dx = (x - tSkinMed) / 0.16f
                    val w = exp(-(dx * dx))
                    val lifted = min(ceilGuard, lMap[bin] + lift * w * 0.88f)
                    if (lifted > lMap[bin]) lMap[bin] = lifted
                }
            }
        }

        val minStep = 0.25f / (lBins - 1)
        val maxStep = (if (isFlat) 3.1f else 2.2f) / (lBins - 1)
        val prev = FloatArray(lBins)
        repeat(4) {
            lMap.copyInto(prev)
            for (bin in 1 until lBins - 1) {
                lMap[bin] = 0.22f * prev[bin - 1] + 0.56f * prev[bin] + 0.22f * prev[bin + 1]
            }
        }
        lMap[0] = lMap[0].coerceIn(0f, 0.12f)
        for (bin in 1 until lBins) {
            val hi = min(1f, lMap[bin - 1] + maxStep)
            val lo = min(hi, lMap[bin - 1] + minStep)
            lMap[bin] = lMap[bin].coerceIn(lo, hi)
        }
        lMap[lBins - 1] = lMap[lBins - 1].coerceIn(0.86f, 1f)

        fun evalLMap(v: Float): Float {
            val f = (v.coerceIn(0f, 1f) * (lBins - 1)).coerceIn(0f, (lBins - 1).toFloat())
            val i0 = min(lBins - 1, floor(f).toInt())
            val i1 = min(lBins - 1, i0 + 1)
            val frac = f - i0
            return lMap[i0] * (1f - frac) + lMap[i1] * frac
        }

        // Step 2: 5-Zone OKLab Mean + 2x2 Covariance Transfer
        val zones = 5
        val centers = floatArrayOf(0.10f, 0.30f, 0.50f, 0.70f, 0.90f)
        val zoneSigma = 0.21f

        val tW = DoubleArray(zones); val tA = DoubleArray(zones); val tB = DoubleArray(zones)
        val tAA = DoubleArray(zones); val tAB = DoubleArray(zones); val tBB = DoubleArray(zones)
        val rW = DoubleArray(zones); val rA = DoubleArray(zones); val rB = DoubleArray(zones)
        val rAA = DoubleArray(zones); val rAB = DoubleArray(zones); val rBB = DoubleArray(zones)

        for (p in 0 until tgtN) {
            val lEff = evalLMap(tgtLabs[p * 3])
            val a = tgtLabs[p * 3 + 1].toDouble()
            val b = tgtLabs[p * 3 + 2].toDouble()
            for (z in 0 until zones) {
                val dx = (lEff - centers[z]) / zoneSigma
                val w = exp(-(dx * dx)).toDouble()
                if (w > 0.01) {
                    tW[z] += w; tA[z] += w * a; tB[z] += w * b
                    tAA[z] += w * a * a; tAB[z] += w * a * b; tBB[z] += w * b * b
                }
            }
        }
        for (p in 0 until refN) {
            val l = refLabs[p * 3]
            val a = refLabs[p * 3 + 1].toDouble()
            val b = refLabs[p * 3 + 2].toDouble()
            for (z in 0 until zones) {
                val dx = (l - centers[z]) / zoneSigma
                val w = exp(-(dx * dx)).toDouble()
                if (w > 0.01) {
                    rW[z] += w; rA[z] += w * a; rB[z] += w * b
                    rAA[z] += w * a * a; rAB[z] += w * a * b; rBB[z] += w * b * b
                }
            }
        }

        val eps = if (isFlat) 0.00022f else 0.00042f
        val maxGain = if (isFlat) 2.65f else 1.90f
        val muTa = FloatArray(zones); val muTb = FloatArray(zones)
        val muRa = FloatArray(zones); val muRb = FloatArray(zones)
        val tMats = Array(zones) { Mat2(1f, 0f, 0f, 1f) }

        for (z in 0 until zones) {
            val (ta, tb, taa, tab, tbb) = if (tW[z] > 1e-4) {
                val ma = (tA[z] / tW[z]).toFloat(); val mb = (tB[z] / tW[z]).toFloat()
                floatArrayOf(
                    ma, mb,
                    max(1e-6f, (tAA[z] / tW[z]).toFloat() - ma * ma),
                    (tAB[z] / tW[z]).toFloat() - ma * mb,
                    max(1e-6f, (tBB[z] / tW[z]).toFloat() - mb * mb)
                )
            } else floatArrayOf(0f, 0f, 0.0005f, 0f, 0.0005f)

            val (ra, rb, raa, rab, rbb) = if (rW[z] > 1e-4) {
                val ma = (rA[z] / rW[z]).toFloat(); val mb = (rB[z] / rW[z]).toFloat()
                floatArrayOf(
                    ma, mb,
                    max(1e-6f, (rAA[z] / rW[z]).toFloat() - ma * ma),
                    (rAB[z] / rW[z]).toFloat() - ma * mb,
                    max(1e-6f, (rBB[z] / rW[z]).toFloat() - mb * mb)
                )
            } else floatArrayOf(0f, 0f, 0.0005f, 0f, 0.0005f)

            muTa[z] = ta; muTb[z] = tb
            muRa[z] = ra; muRb[z] = rb
            tMats[z] = computeMk2x2(taa, tab, tbb, raa, rab, rbb, eps, maxGain)
        }

        val hasTgtSkin = tgtSkinL.size >= 20
        val hasRefSkin = refSkinL.size >= 20
        val tgtSkinMuA = if (hasTgtSkin) (tgtSkinASum / tgtSkinL.size).toFloat() else 0.035f
        val tgtSkinMuB = if (hasTgtSkin) (tgtSkinBSum / tgtSkinL.size).toFloat() else 0.045f
        val refSkinAng = if (hasRefSkin) atan2((refSkinBSum / refSkinL.size).toFloat(), (refSkinASum / refSkinL.size).toFloat()) else null
        val refSkinC = if (hasRefSkin) {
            val a = (refSkinASum / refSkinL.size).toFloat()
            val b = (refSkinBSum / refSkinL.size).toFloat()
            sqrt(a * a + b * b)
        } else null

        // Step 3: Populate 17x17x17 3D LUT
        val dim = MATCH_LUT_DIM
        val lut = FloatArray(dim * dim * dim * 3)
        val denom = (dim - 1).toFloat()
        val d2 = dim * dim
        val linTmp = FloatArray(3)
        val srgbOutTmp = FloatArray(3)
        val wZ = FloatArray(zones)

        for (bi in 0 until dim) {
            val b = bi / denom
            for (gi in 0 until dim) {
                val g = gi / denom
                for (ri in 0 until dim) {
                    val r = ri / denom
                    srgbToOklab(r, g, b, labTmp)
                    val lIn = labTmp[0]; val aIn = labTmp[1]; val bIn = labTmp[2]
                    val lOut = evalLMap(lIn)

                    var wSum = 0f
                    for (z in 0 until zones) {
                        val dx = (lOut - centers[z]) / zoneSigma
                        val w = exp(-(dx * dx))
                        wZ[z] = w
                        wSum += w
                    }
                    val invW = if (wSum > 1e-6f) 1f / wSum else 1f / zones

                    var aMk = 0f; var bMk = 0f
                    for (z in 0 until zones) {
                        val wz = wZ[z] * invW
                        val da = aIn - muTa[z]
                        val db = bIn - muTb[z]
                        val mat = tMats[z]
                        aMk += wz * (muRa[z] + mat.m00 * da + mat.m01 * db)
                        bMk += wz * (muRb[z] + mat.m10 * da + mat.m11 * db)
                    }

                    val cIn = sqrt(aIn * aIn + bIn * bIn)
                    if (lIn > 0.85f && cIn < 0.03f) {
                        val hiGuard = smoothstep(0.85f, 0.99f, lIn) * (1f - smoothstep(0.01f, 0.03f, cIn))
                        aMk *= 1f - hiGuard * 0.55f
                        bMk *= 1f - hiGuard * 0.55f
                    }

                    var aOut = aMk; var bOut = bMk
                    if (hasTgtSkin && cIn > 0.004f && lIn > 0.18f && lIn < 0.90f) {
                        val angIn = atan2(bIn, aIn)
                        val skinAngCenter = atan2(tgtSkinMuB, tgtSkinMuA).coerceIn(0.32f, 0.96f)
                        val angDiff = kotlin.math.abs(angIn - skinAngCenter)
                        val minC = if (isFlat) 0.006f else 0.016f
                        if (angDiff < 0.42f && cIn >= minC && cIn < 0.16f) {
                            val wAng = 1f - smoothstep(0.16f, 0.42f, angDiff)
                            val wLum = smoothstep(0.18f, 0.34f, lIn) * (1f - smoothstep(0.80f, 0.92f, lIn))
                            val wCh = 1f - smoothstep(0.10f, 0.16f, cIn)
                            val skinW = (wAng * wLum * wCh).coerceIn(0f, 1f)
                            if (skinW > 0.01f) {
                                val mkAng = atan2(bOut, aOut)
                                val mkC = sqrt(aOut * aOut + bOut * bOut)
                                val (targetAng, minSkC, maxSkC) = if (refSkinAng != null && refSkinC != null) {
                                    val cRefAng = refSkinAng.coerceIn(0.40f, 0.98f)
                                    val bAng = cRefAng * 0.72f + mkAng.coerceIn(0.36f, 1.02f) * 0.28f
                                    Triple(bAng, (refSkinC * 0.65f).coerceIn(0.045f, 0.085f), 0.135f)
                                } else {
                                    val healthy = skinAngCenter.coerceIn(0.46f, 0.88f)
                                    val hinted = healthy + (mkAng - healthy).coerceIn(-0.16f, 0.16f)
                                    Triple(hinted, if (isFlat) 0.055f else 0.042f, 0.125f)
                                }
                                val targetC = mkC.coerceIn(minSkC, maxSkC)
                                val safeA = targetC * cos(targetAng)
                                val safeB = targetC * sin(targetAng)
                                val blend = skinW * 0.72f
                                aOut += (safeA - aOut) * blend
                                bOut += (safeB - bOut) * blend
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
        return Triple(lut, palette, isFlat)
    }

    /** Pure Kotlin pixel-array entry point (JVM unit-testable without Android Bitmap dependency). */
    fun matchPixelsToReference(
        refPixels: IntArray,
        targetPixels: IntArray,
        targetParams: DevelopParams,
        forceRawFlat: Boolean? = null,
        refName: String = "Reference"
    ): DevelopParams {
        val ref = extractStatsFromPixels(refPixels)
        val tgt = extractStatsFromPixels(targetPixels)
        val bothPortraits = ref.skinPixelCount >= 30 && tgt.skinPixelCount >= 30

        val dEv = if (bothPortraits) {
            ((ref.skinMedianLum.coerceAtLeast(0.12f) - tgt.skinMedianLum.coerceAtLeast(0.12f)) * 1.4f)
                .coerceIn(-0.55f, 0.55f)
        } else {
            ((ref.p50Lum.coerceAtLeast(0.05f) - tgt.p50Lum.coerceAtLeast(0.05f)) * 1.1f)
                .coerceIn(-0.60f, 0.60f)
        }
        val refSpread = (ref.p95Lum - ref.p05Lum).coerceAtLeast(0.08f)
        val tgtSpread = (tgt.p95Lum - tgt.p05Lum).coerceAtLeast(0.08f)
        val dCon = ((refSpread / tgtSpread - 1f) * 12f).coerceIn(-10f, 12f)
        val dWht = ((ref.p95Lum - tgt.p95Lum) * 15f).coerceIn(-12f, 12f)

        val (dTemp, dTint) = if (bothPortraits) {
            val rg = ref.skinMeanRg - tgt.skinMeanRg
            val bg = ref.skinMeanBg - tgt.skinMeanBg
            (((rg - bg) * 700f).coerceIn(-260f, 260f)) to (((rg + bg) * 3f).coerceIn(-5f, 5f))
        } else {
            val rg = ref.midRgRatio - tgt.midRgRatio
            val bg = ref.midBgRatio - tgt.midBgRatio
            (((rg - bg) * 650f).coerceIn(-340f, 340f)) to (((rg + bg - 2f) * 6f).coerceIn(-5f, 5f))
        }

        val (lutData, palette, isFlat) = buildMatchLutFromPixels(refPixels, targetPixels, forceRawFlat)
        val lutOk = lutData.size == MATCH_LUT_DIM * MATCH_LUT_DIM * MATCH_LUT_DIM * 3

        return targetParams.copy(
            whiteBalance = WhiteBalanceMode.CUSTOM,
            temperature = (targetParams.temperature + dTemp).coerceIn(2800f, 12000f),
            tint = (targetParams.tint + dTint).coerceIn(-50f, 50f),
            exposure = (targetParams.exposure + dEv).coerceIn(-3f, 3f),
            contrast = (targetParams.contrast + dCon).coerceIn(-50f, 50f),
            whites = (targetParams.whites + dWht).coerceIn(-20f, 20f),
            matchLutDim = if (lutOk) MATCH_LUT_DIM else 0,
            matchLutData = if (lutOk) lutData else FloatArray(0),
            matchToneAmount = if (lutOk && targetParams.hasColorMatchLut()) targetParams.matchToneAmount.coerceIn(0f, 100f) else 100f,
            matchColorAmount = if (lutOk && targetParams.hasColorMatchLut()) targetParams.matchColorAmount.coerceIn(0f, 100f) else 100f,
            matchRawFlat = isFlat,
            matchPalette = palette,
            matchRefName = refName
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
        // Compute gentle baseline adjustments cleanly, then sample the rendered pre-LUT target buffer
        // so the 3D LUT matches the live pipeline pixel distribution 1:1.
        val refPx = IntArray(renderedRef.width * renderedRef.height).also {
            renderedRef.getPixels(it, 0, renderedRef.width, 0, 0, renderedRef.width, renderedRef.height)
        }
        val rawTgtPx = IntArray(tgtSmall.width * tgtSmall.height).also {
            tgtSmall.getPixels(it, 0, tgtSmall.width, 0, 0, tgtSmall.width, tgtSmall.height)
        }
        val preBase = matchPixelsToReference(refPx, rawTgtPx, targetParams.clearColorMatch(), forceRawFlat, refName)
        val baseOnlyParams = preBase.clearColorMatch().copy(vignetteAmount = 0f, grainAmount = 0f)
        val renderedTgt = KotlinPv2012Engine.render(
            tgtSmall,
            baseOnlyParams,
            draftInteractive = true,
            applyCrop = false
        ).bitmap
        val renderedTgtPx = IntArray(renderedTgt.width * renderedTgt.height).also {
            renderedTgt.getPixels(it, 0, renderedTgt.width, 0, 0, renderedTgt.width, renderedTgt.height)
        }

        val (lutData, palette, isFlat) = buildMatchLutFromPixels(refPx, renderedTgtPx, forceRawFlat)
        if (renderedRef !== refSmall && !renderedRef.isRecycled) renderedRef.recycle()
        if (refSmall !== refBitmap && !refSmall.isRecycled) refSmall.recycle()
        if (!renderedTgt.isRecycled) renderedTgt.recycle()
        if (tgtSmall !== targetBitmap && !tgtSmall.isRecycled) tgtSmall.recycle()

        val lutOk = lutData.size == MATCH_LUT_DIM * MATCH_LUT_DIM * MATCH_LUT_DIM * 3
        return preBase.copy(
            clarity = if (refParams != null) (preBase.clarity * 0.5f + refParams.clarity * 0.5f).coerceIn(-100f, 100f) else preBase.clarity,
            dehaze = if (refParams != null) (preBase.dehaze * 0.5f + refParams.dehaze * 0.5f).coerceIn(-100f, 100f) else preBase.dehaze,
            matchLutDim = if (lutOk) MATCH_LUT_DIM else 0,
            matchLutData = if (lutOk) lutData else FloatArray(0),
            matchToneAmount = if (lutOk && targetParams.hasColorMatchLut()) targetParams.matchToneAmount.coerceIn(0f, 100f) else 100f,
            matchColorAmount = if (lutOk && targetParams.hasColorMatchLut()) targetParams.matchColorAmount.coerceIn(0f, 100f) else 100f,
            matchRawFlat = isFlat,
            matchPalette = palette,
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
