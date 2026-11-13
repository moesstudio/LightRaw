package com.moestret.lightraw.engine

import android.graphics.Bitmap
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.WhiteBalanceMode
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min

/**
 * LightRaw AI Color Match & Auto Tone Engine (Mobile Twin of `src/ai/color_match.rs`).
 * Analyzes reference photo tone distribution, white balance ratios, split toning, and
 * skin-tone anchors so portraits never blow out highlights or over-warm uniform whites.
 */
object AiColorMatchEngine {

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

    fun extractStats(bitmap: Bitmap): ColorStats {
        val w = bitmap.width
        val h = bitmap.height
        val total = w * h
        if (total == 0) {
            return ColorStats(0.1f, 0.25f, 0.5f, 0.75f, 0.9f, 25f, 1f, 1f, 0f, 0f, 0f, 0f, 0, 0.5f, 1f, 1f)
        }
        val pixels = IntArray(total)
        bitmap.getPixels(pixels, 0, w, 0, 0, w, h)
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
            val maxC = max(r, max(g, b))
            val minC = min(r, min(g, b))
            val delta = maxC - minC
            val sat = if (maxC > 1e-4f) (delta / maxC) * 100f else 0f
            val hue = if (delta > 1e-4f) {
                val hDeg = when {
                    maxC == r -> ((g - b) / delta) % 6f
                    maxC == g -> ((b - r) / delta) + 2f
                    else -> ((r - g) / delta) + 4f
                } * 60f
                if (hDeg < 0f) hDeg + 360f else hDeg
            } else 0f

            lums.add(lum)
            sumSat += sat
            satCount++

            val isSkin = lum in 0.15f..0.90f &&
                r > g && g > b &&
                (r - g) >= 0.02f && (g - b) >= 0.012f &&
                (r - b) in 0.05f..0.65f &&
                hue in 10f..48f && sat in 12f..75f

            if (isSkin) {
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

    fun matchToReference(
        refBitmap: Bitmap,
        targetBitmap: Bitmap,
        refParams: DevelopParams?,
        targetParams: DevelopParams
    ): DevelopParams {
        val ref = extractStats(refBitmap)
        val tgt = extractStats(targetBitmap)
        val bothPortraits = ref.skinPixelCount >= 30 && tgt.skinPixelCount >= 30

        val dEv = if (bothPortraits) {
            ((ref.skinMedianLum.coerceAtLeast(0.12f) - tgt.skinMedianLum.coerceAtLeast(0.12f)) * 1.8f)
                .coerceIn(-0.65f, 0.65f)
        } else {
            ((ref.p50Lum.coerceAtLeast(0.05f) - tgt.p50Lum.coerceAtLeast(0.05f)) * 1.5f)
                .coerceIn(-0.75f, 0.75f)
        }

        val refSpread = (ref.p95Lum - ref.p05Lum).coerceAtLeast(0.08f)
        val tgtSpread = (tgt.p95Lum - tgt.p05Lum).coerceAtLeast(0.08f)
        val dCon = ((refSpread / tgtSpread - 1f) * 30f).coerceIn(-18f, 22f)
        val dWht = ((ref.p95Lum - tgt.p95Lum) * 25f).coerceIn(-15f, 15f)
        val dBlk = ((ref.p05Lum - tgt.p05Lum) * 30f).coerceIn(-15f, 15f)
        val dHi = ((tgt.p95Lum - ref.p95Lum) * -30f).coerceIn(-25f, 20f)
        val dShd = ((ref.p25Lum - tgt.p25Lum) * 35f).coerceIn(-20f, 25f)

        val (dTemp, dTint) = if (bothPortraits) {
            val rg = ref.skinMeanRg - tgt.skinMeanRg
            val bg = ref.skinMeanBg - tgt.skinMeanBg
            (((rg - bg) * 850f).coerceIn(-300f, 300f)) to (((rg + bg) * 4f).coerceIn(-5f, 5f))
        } else {
            val rg = ref.midRgRatio - tgt.midRgRatio
            val bg = ref.midBgRatio - tgt.midBgRatio
            (((rg - bg) * 750f).coerceIn(-400f, 400f)) to (((rg + bg - 2f) * 8f).coerceIn(-6f, 6f))
        }

        val satDiff = (ref.meanSat - tgt.meanSat) * 0.65f
        var out = targetParams.copy(
            whiteBalance = WhiteBalanceMode.CUSTOM,
            temperature = (targetParams.temperature + dTemp).coerceIn(2800f, 12000f),
            tint = (targetParams.tint + dTint).coerceIn(-50f, 50f),
            exposure = (targetParams.exposure + dEv).coerceIn(-3f, 3f),
            contrast = (targetParams.contrast + dCon).coerceIn(-50f, 50f),
            highlights = (targetParams.highlights + dHi).coerceIn(-50f, 30f),
            shadows = (targetParams.shadows + dShd).coerceIn(-40f, 50f),
            whites = (targetParams.whites + dWht).coerceIn(-25f, 25f),
            blacks = (targetParams.blacks + dBlk).coerceIn(-25f, 25f),
            vibrance = (targetParams.vibrance + satDiff).coerceIn(-20f, 30f),
            saturation = (targetParams.saturation + satDiff * 0.45f).coerceIn(-15f, 20f),
            cgShadowHue = if (ref.shadowSat > 2f) ref.shadowHue else targetParams.cgShadowHue,
            cgShadowSat = if (ref.shadowSat > 2f) (ref.shadowSat * 0.35f).coerceIn(0f, 35f) else targetParams.cgShadowSat,
            cgHighHue = if (ref.highlightSat > 2f) ref.highlightHue else targetParams.cgHighHue,
            cgHighSat = if (ref.highlightSat > 2f) (ref.highlightSat * 0.30f).coerceIn(0f, 30f) else targetParams.cgHighSat
        )

        if (refParams != null) {
            out = out.copy(
                toneCurveMaster = refParams.toneCurveMaster.toList(),
                hueAdjustment = refParams.hueAdjustment.copyOf(),
                saturationAdjustment = refParams.saturationAdjustment.copyOf(),
                luminanceAdjustment = refParams.luminanceAdjustment.copyOf(),
                cgMidHue = refParams.cgMidHue,
                cgMidSat = refParams.cgMidSat,
                clarity = (out.clarity * 0.5f + refParams.clarity * 0.5f).coerceIn(-100f, 100f),
                dehaze = (out.dehaze * 0.5f + refParams.dehaze * 0.5f).coerceIn(-100f, 100f),
                cameraProfile = refParams.cameraProfile
            )
        }
        return out
    }
}
