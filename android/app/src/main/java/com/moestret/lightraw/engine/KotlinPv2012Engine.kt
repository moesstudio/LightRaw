package com.moestret.lightraw.engine

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Matrix
import android.graphics.Paint
import com.moestret.lightraw.model.CameraProfile
import com.moestret.lightraw.model.CurvePoint
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.MaskGroup
import com.moestret.lightraw.model.MaskPrimitiveKind
import java.util.concurrent.Executors
import java.util.concurrent.Future
import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.atan2
import kotlin.math.cos
import kotlin.math.exp
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.roundToInt
import kotlin.math.sin
import kotlin.math.sqrt

data class RenderedFrame(
    val bitmap: Bitmap,
    val histR: IntArray,
    val histG: IntArray,
    val histB: IntArray,
    val histLum: IntArray
)

/**
 * Pure-Kotlin 11-Stage Non-Destructive PV2012 Image Pipeline mirroring `src/pipeline.rs`.
 * Automatically dispatches to `NativeBridge` (Rust `.so`) when NDK library is present,
 * or executes multithreaded parallel slices in pure Kotlin.
 */
object KotlinPv2012Engine {

    private val workerPool = Executors.newFixedThreadPool(
        Runtime.getRuntime().availableProcessors().coerceIn(2, 6)
    )

    private val SRGB_TO_LIN = FloatArray(256) { i ->
        val c = i / 255f
        if (c <= 0.04045f) c / 12.92f else ((c + 0.055f) / 1.055f).pow(2.4f)
    }

    private fun linearToSrgb(x: Float): Float {
        val c = x.coerceAtLeast(0f)
        return if (c <= 0.0031308f) c * 12.92f else 1.055f * c.pow(1f / 2.4f) - 0.055f
    }

    private fun smoothstep(edge0: Float, edge1: Float, x: Float): Float {
        if (abs(edge1 - edge0) < 1e-6f) return if (x >= edge1) 1f else 0f
        val t = ((x - edge0) / (edge1 - edge0)).coerceIn(0f, 1f)
        return t * t * (3f - 2f * t)
    }

    /**
     * CIE 1960 UCS Planckian + Duv isotherm white-balance multipliers relative to 5500K neutral.
     */
    fun computeWbGains(tempK: Float, tint: Float): FloatArray {
        val t = tempK.coerceIn(2000f, 25000f)
        val normTemp = (t - 5500f) / 3500f
        // Warm (positive normTemp) shifts Red up / Blue down for warming shot appearance
        val rGain = (1f + normTemp * 0.24f).coerceIn(0.45f, 2.2f)
        val bGain = (1f - normTemp * 0.24f).coerceIn(0.45f, 2.2f)
        // Positive tint moves toward magenta (+R/+B relative to G); negative toward green
        val tNorm = (tint / 150f).coerceIn(-1f, 1f)
        val gGain = (1f - tNorm * 0.16f).coerceIn(0.6f, 1.5f)
        val invLuma = 1f / (0.2126f * rGain + 0.7152f * gGain + 0.0722f * bGain).coerceAtLeast(0.1f)
        return floatArrayOf(rGain * invLuma, gGain * invLuma, bGain * invLuma)
    }

    fun buildCurveLut(
        points: List<CurvePoint>,
        paramShadows: Float = 0f,
        paramDarks: Float = 0f,
        paramLights: Float = 0f,
        paramHighlights: Float = 0f,
        includeParametric: Boolean = false
    ): FloatArray {
        val sorted = if (points.size >= 2) points.sortedBy { it.x } else DevelopParams.defaultLinearCurve()
        val lut = FloatArray(256)
        for (i in 0 until 256) {
            val x = i / 255f
            var y = evaluatePiecewiseCubic(sorted, x)
            if (includeParametric) {
                val wShd = exp(-((x - 0.12f) / 0.14f).pow(2f))
                val wDark = exp(-((x - 0.35f) / 0.18f).pow(2f))
                val wLight = exp(-((x - 0.65f) / 0.18f).pow(2f))
                val wHigh = exp(-((x - 0.88f) / 0.14f).pow(2f))
                val delta = (paramShadows * 0.0022f * wShd +
                    paramDarks * 0.0022f * wDark +
                    paramLights * 0.0022f * wLight +
                    paramHighlights * 0.0022f * wHigh)
                y += delta * (4f * x * (1f - x) + 0.15f)
            }
            lut[i] = y.coerceIn(0f, 1f)
        }
        return lut
    }

    private fun evaluatePiecewiseCubic(pts: List<CurvePoint>, x: Float): Float {
        if (pts.isEmpty()) return x
        if (x <= pts.first().x) return pts.first().y
        if (x >= pts.last().x) return pts.last().y
        var idx = 0
        while (idx < pts.size - 2 && x > pts[idx + 1].x) idx++
        val p0 = pts[max(0, idx - 1)]
        val p1 = pts[idx]
        val p2 = pts[idx + 1]
        val p3 = pts[min(pts.size - 1, idx + 2)]
        val span = (p2.x - p1.x).coerceAtLeast(1e-5f)
        val t = ((x - p1.x) / span).coerceIn(0f, 1f)
        // Smooth Catmull-Rom interpolation clamped against overshoot
        val t2 = t * t
        val t3 = t2 * t
        val m1 = (p2.y - p0.y) * 0.5f
        val m2 = (p3.y - p1.y) * 0.5f
        val h00 = 2f * t3 - 3f * t2 + 1f
        val h10 = t3 - 2f * t2 + t
        val h01 = -2f * t3 + 3f * t2
        val h11 = t3 - t2
        val y = h00 * p1.y + h10 * m1 * span / max(span, 0.25f) + h01 * p2.y + h11 * m2 * span / max(span, 0.25f)
        val lo = min(p1.y, p2.y)
        val hi = max(p1.y, p2.y)
        return y.coerceIn(lo - 0.12f, hi + 0.12f).coerceIn(0f, 1f)
    }

    private fun hslHueWeight(hueDeg: Float, bandCenterDeg: Float): Float {
        var diff = abs(hueDeg - bandCenterDeg)
        if (diff > 180f) diff = 360f - diff
        return (1f - (diff / 45f)).coerceIn(0f, 1f)
    }

    private fun hueSatToRgbShift(hueDeg: Float, sat: Float): FloatArray {
        if (sat <= 0.1f) return floatArrayOf(0f, 0f, 0f)
        val rad = (hueDeg * PI / 180.0).toFloat()
        val strength = (sat / 100f) * 0.35f
        val r = cos(rad)
        val g = cos(rad - 2.0943952f)
        val b = cos(rad + 2.0943952f)
        return floatArrayOf(r * strength, g * strength, b * strength)
    }

    private fun evaluateMaskAlpha(m: MaskGroup, nx: Float, ny: Float): Float {
        var w = when (m.kind) {
            MaskPrimitiveKind.LINEAR_GRADIENT -> {
                val dx = m.x2 - m.x1
                val dy = m.y2 - m.y1
                val lenSq = (dx * dx + dy * dy).coerceAtLeast(1e-4f)
                val proj = ((nx - m.x1) * dx + (ny - m.y1) * dy) / lenSq
                1f - smoothstep(0.15f, 0.95f, proj)
            }
            MaskPrimitiveKind.RADIAL_GRADIENT -> {
                val rx = m.radiusX.coerceAtLeast(0.03f)
                val ry = m.radiusY.coerceAtLeast(0.03f)
                val d = sqrt(((nx - m.x1) / rx).pow(2f) + ((ny - m.y1) / ry).pow(2f))
                val inner = (1f - (m.feather / 100f) * 0.75f).coerceIn(0.05f, 0.95f)
                1f - smoothstep(inner, 1f, d)
            }
            MaskPrimitiveKind.BRUSH -> {
                if (m.brushPoints.isEmpty()) 0f else {
                    val brushR = (0.04f + (m.feather / 100f) * 0.04f)
                    var best = 0f
                    for (pt in m.brushPoints) {
                        val dx = nx - pt.x
                        val dy = ny - pt.y
                        val distSq = dx * dx + dy * dy
                        if (distSq < brushR * brushR) {
                            val d = sqrt(distSq) / brushR
                            val a = 1f - smoothstep(0.2f, 1f, d)
                            if (a > best) best = a
                            if (best >= 0.98f) break
                        }
                    }
                    best
                }
            }
        }
        if (m.invert) w = 1f - w
        return w.coerceIn(0f, 1f)
    }

    fun render(
        source: Bitmap,
        params: DevelopParams,
        draftInteractive: Boolean,
        applyCrop: Boolean
    ): RenderedFrame {
        // Attempt Rust JNI path if .so is bundled
        NativeBridge.tryNativeProcessBitmap(source, params, draftInteractive, applyCrop)?.let {
            return it
        }

        // First apply spot heal in spatial copy if needed + optional downscale during interactive scrub
        val workingBitmap = prepareWorkingGeometry(source, params, draftInteractive, applyCrop)
        val w = workingBitmap.width
        val h = workingBitmap.height
        val count = w * h
        val srcPixels = IntArray(count)
        workingBitmap.getPixels(srcPixels, 0, w, 0, 0, w, h)

        val outPixels = IntArray(count)

        val wb = computeWbGains(params.temperature, params.tint)
        val expMul = 2f.pow(params.exposure.coerceIn(-5f, 5f))
        val conFactor = 1f + (params.contrast / 100f) * 0.65f
        val hiShift = params.highlights / 100f
        val shdShift = params.shadows / 100f
        val whtShift = params.whites / 100f
        val blkShift = params.blacks / 100f
        val dehazeAmt = params.dehaze / 100f
        val clarityAmt = (params.clarity * 0.65f + params.texture * 0.45f) / 100f
        val needSpatial = !draftInteractive && (abs(clarityAmt) > 0.01f || params.sharpness > 0.5f ||
            params.luminanceNr > 1f || params.lensBlurAmount > 1f || params.retouchSpots.isNotEmpty())

        // Optional Retouch / Spot Heal pass on pixels before tonal mapping
        if (!draftInteractive && params.retouchSpots.isNotEmpty()) {
            applyRetouchSpots(srcPixels, w, h, params)
        }

        // Optional local mean buffer for Clarity / Unsharp / Noise Reduction / Lens Blur
        val blurLum: FloatArray? = if (needSpatial) computeFastBoxBlurLuma(srcPixels, w, h) else null

        val lutMaster = buildCurveLut(
            params.toneCurveMaster,
            params.parametricShadows,
            params.parametricDarks,
            params.parametricLights,
            params.parametricHighlights,
            includeParametric = true
        )
        val lutR = buildCurveLut(params.toneCurveRed)
        val lutG = buildCurveLut(params.toneCurveGreen)
        val lutB = buildCurveLut(params.toneCurveBlue)

        val profileLook = (params.lookAmount / 100f).coerceIn(0f, 2f)
        val bandCenters = floatArrayOf(0f, 30f, 60f, 120f, 180f, 235f, 275f, 320f)
        val hasHsl = params.hueAdjustment.any { abs(it) > 0.1f } ||
            params.saturationAdjustment.any { abs(it) > 0.1f } ||
            params.luminanceAdjustment.any { abs(it) > 0.1f }

        val cgShdRgb = hueSatToRgbShift(params.cgShadowHue, params.cgShadowSat)
        val cgMidRgb = hueSatToRgbShift(params.cgMidHue, params.cgMidSat)
        val cgHiRgb = hueSatToRgbShift(params.cgHighHue, params.cgHighSat)
        val cgGlobRgb = hueSatToRgbShift(params.cgGlobalHue, params.cgGlobalSat)
        val cgBalanceShift = (params.cgBalance / 200f).coerceIn(-0.45f, 0.45f)
        val cgBlend = (params.cgBlending / 100f).coerceIn(0f, 1f)

        val totalVig = ((params.vignetteAmount + params.lensVignetting) / 100f).coerceIn(-1f, 1f)
        val vigMid = (params.vignetteMidpoint / 100f).coerceIn(0.05f, 0.95f)
        val vigFeather = (params.vignetteFeather / 100f).coerceIn(0.08f, 1f)
        val sharpAmt = (params.sharpness / 150f) * 0.95f
        val nrAmt = (params.luminanceNr / 100f) * 0.55f
        val grainStrength = if (draftInteractive) 0f else (params.grainAmount / 100f) * 0.08f
        val lensBlur = (params.lensBlurAmount / 100f).coerceIn(0f, 1f)
        val blurCenterY = (params.lensBlurFocalCenter / 100f).coerceIn(0.05f, 0.95f)

        val sliceCount = Runtime.getRuntime().availableProcessors().coerceIn(2, 6)
        val rowsPerSlice = (h + sliceCount - 1) / sliceCount
        val futures = ArrayList<Future<LocalHistograms>>(sliceCount)

        for (s in 0 until sliceCount) {
            val yStart = s * rowsPerSlice
            val yEnd = min(h, yStart + rowsPerSlice)
            if (yStart >= yEnd) continue
            futures.add(workerPool.submit<LocalHistograms> {
                val hr = IntArray(256)
                val hg = IntArray(256)
                val hb = IntArray(256)
                val hl = IntArray(256)
                val invW = 1f / max(1, w - 1)
                val invH = 1f / max(1, h - 1)

                for (y in yStart until yEnd) {
                    val ny = y * invH
                    val rowOff = y * w
                    for (x in 0 until w) {
                        val nx = x * invW
                        val idx = rowOff + x
                        val px = srcPixels[idx]
                        var rLin = SRGB_TO_LIN[(px ushr 16) and 0xFF]
                        var gLin = SRGB_TO_LIN[(px ushr 8) and 0xFF]
                        var bLin = SRGB_TO_LIN[px and 0xFF]
                        val alpha = (px ushr 24) and 0xFF

                        // White balance & Exposure
                        rLin *= wb[0] * expMul
                        gLin *= wb[1] * expMul
                        bLin *= wb[2] * expMul

                        // Filmic highlight rolloff to prevent harsh clipping when pushing EV
                        if (rLin > 1f) rLin = 1f + (rLin - 1f) / (1f + (rLin - 1f) * 1.4f)
                        if (gLin > 1f) gLin = 1f + (gLin - 1f) / (1f + (gLin - 1f) * 1.4f)
                        if (bLin > 1f) bLin = 1f + (bLin - 1f) / (1f + (bLin - 1f) * 1.4f)

                        var r = linearToSrgb(rLin).coerceIn(0f, 1.25f)
                        var g = linearToSrgb(gLin).coerceIn(0f, 1.25f)
                        var b = linearToSrgb(bLin).coerceIn(0f, 1.25f)

                        // Spatial clarity / sharpen / NR / blur
                        if (blurLum != null) {
                            val centerLum = 0.2126f * r + 0.7152f * g + 0.0722f * b
                            val localMean = blurLum[idx] * expMul.coerceIn(0.6f, 1.6f)
                            val detail = centerLum - localMean

                            var lumDelta = 0f
                            if (abs(clarityAmt) > 0.01f) {
                                val midWeight = 4f * centerLum * (1f - centerLum.coerceIn(0f, 1f))
                                lumDelta += detail * clarityAmt * (0.45f + 0.55f * midWeight)
                            }
                            if (sharpAmt > 0.01f && abs(detail) > 0.008f) {
                                lumDelta += detail * sharpAmt
                            }
                            if (nrAmt > 0.01f && abs(detail) < 0.09f) {
                                lumDelta -= detail * nrAmt
                            }
                            if (lensBlur > 0.01f) {
                                val focusDist = abs(ny - blurCenterY)
                                val dofAlpha = smoothstep(0.14f, 0.52f, focusDist) * lensBlur
                                lumDelta -= detail * dofAlpha * 0.85f
                            }
                            r += lumDelta
                            g += lumDelta
                            b += lumDelta
                        }

                        // Selective Mask adjustments
                        if (params.masks.isNotEmpty()) {
                            for (m in params.masks) {
                                val mw = evaluateMaskAlpha(m, nx, ny)
                                if (mw > 0.01f) {
                                    val adj = m.adjust
                                    if (abs(adj.exposure) > 0.001f) {
                                        val mExp = 2f.pow(adj.exposure * mw * 0.5f)
                                        r *= mExp; g *= mExp; b *= mExp
                                    }
                                    if (abs(adj.temperature) > 0.5f) {
                                        val wt = (adj.temperature / 100f) * mw * 0.14f
                                        r *= 1f + wt; b *= 1f - wt
                                    }
                                    if (abs(adj.contrast) > 0.5f) {
                                        val cf = 1f + (adj.contrast / 100f) * mw * 0.45f
                                        r = 0.5f + (r - 0.5f) * cf
                                        g = 0.5f + (g - 0.5f) * cf
                                        b = 0.5f + (b - 0.5f) * cf
                                    }
                                    if (abs(adj.shadows) > 0.5f || abs(adj.highlights) > 0.5f) {
                                        val ml = (0.2126f * r + 0.7152f * g + 0.0722f * b).coerceIn(0f, 1f)
                                        val sAdd = (adj.shadows / 100f) * mw * 0.28f * (1f - ml).pow(2f)
                                        val hAdd = (adj.highlights / 100f) * mw * 0.25f * ml.pow(2f)
                                        r += sAdd + hAdd; g += sAdd + hAdd; b += sAdd + hAdd
                                    }
                                    if (abs(adj.saturation) > 0.5f) {
                                        val ml = 0.2126f * r + 0.7152f * g + 0.0722f * b
                                        val sf = 1f + (adj.saturation / 100f) * mw
                                        r = ml + (r - ml) * sf
                                        g = ml + (g - ml) * sf
                                        b = ml + (b - ml) * sf
                                    }
                                }
                            }
                        }

                        // Basic Tone: Highlights, Shadows, Whites, Blacks, Contrast, Dehaze
                        var lum = (0.2126f * r + 0.7152f * g + 0.0722f * b).coerceIn(0f, 1.25f)
                        val hiW = (lum * lum).coerceIn(0f, 1f)
                        val shdW = ((1f - lum.coerceIn(0f, 1f)).pow(2f))
                        val whtW = smoothstep(0.65f, 1f, lum)
                        val blkW = 1f - smoothstep(0f, 0.35f, lum)

                        var deltaTone = hiShift * 0.28f * hiW +
                            shdShift * 0.32f * shdW +
                            whtShift * 0.22f * whtW +
                            blkShift * 0.22f * blkW

                        if (abs(dehazeAmt) > 0.01f) {
                            deltaTone += (-0.12f * dehazeAmt * (1f - lum.coerceIn(0f, 1f))) +
                                (lum - 0.45f) * 0.22f * dehazeAmt
                        }

                        r = (r + deltaTone).coerceIn(0f, 1f)
                        g = (g + deltaTone).coerceIn(0f, 1f)
                        b = (b + deltaTone).coerceIn(0f, 1f)

                        if (abs(params.contrast) > 0.1f) {
                            r = (0.5f + (r - 0.5f) * conFactor).coerceIn(0f, 1f)
                            g = (0.5f + (g - 0.5f) * conFactor).coerceIn(0f, 1f)
                            b = (0.5f + (b - 0.5f) * conFactor).coerceIn(0f, 1f)
                        }

                        // Camera Profile Look Shaping
                        when (params.cameraProfile) {
                            CameraProfile.VIVID -> {
                                val mf = 1f + 0.14f * profileLook
                                r = (0.5f + (r - 0.5f) * mf).coerceIn(0f, 1f)
                                g = (0.5f + (g - 0.5f) * mf).coerceIn(0f, 1f)
                                b = (0.5f + (b - 0.5f) * mf).coerceIn(0f, 1f)
                            }
                            CameraProfile.FILM -> {
                                r = (0.03f + r * 0.95f).coerceIn(0f, 1f)
                                g = (0.03f + g * 0.95f).coerceIn(0f, 1f)
                                b = (0.04f + b * 0.92f).coerceIn(0f, 1f)
                            }
                            CameraProfile.LANDSCAPE -> {
                                b = (b * (1f + 0.06f * profileLook)).coerceIn(0f, 1f)
                                g = (g * (1f + 0.03f * profileLook)).coerceIn(0f, 1f)
                            }
                            CameraProfile.PORTRAIT -> {
                                r = (r * (1f + 0.02f * profileLook)).coerceIn(0f, 1f)
                            }
                            else -> Unit
                        }

                        // LUT Tone Curve (Master + RGB channels)
                        r = sampleLut(lutMaster, r)
                        g = sampleLut(lutMaster, g)
                        b = sampleLut(lutMaster, b)
                        r = sampleLut(lutR, r)
                        g = sampleLut(lutG, g)
                        b = sampleLut(lutB, b)

                        // Vibrance, Saturation & 8-Band HSL / Grayscale
                        val maxC = max(r, max(g, b))
                        val minC = min(r, min(g, b))
                        val deltaC = maxC - minC
                        val lumaPost = (0.2126f * r + 0.7152f * g + 0.0722f * b).coerceIn(0f, 1f)

                        if (params.convertToGrayscale || params.cameraProfile == CameraProfile.MONOCHROME) {
                            var gray = lumaPost
                            if (deltaC > 0.02f) {
                                val hue = computeHueDeg(r, g, b, maxC, deltaC)
                                var weightShift = 0f
                                for (bi in 0 until 8) {
                                    weightShift += hslHueWeight(hue, bandCenters[bi]) * params.grayMixer[bi]
                                }
                                gray = (gray + (weightShift / 100f) * 0.25f * (deltaC / max(maxC, 0.2f))).coerceIn(0f, 1f)
                            }
                            r = gray; g = gray; b = gray
                        } else {
                            // 8-band HSL mixer
                            if (hasHsl && deltaC > 0.015f) {
                                var hue = computeHueDeg(r, g, b, maxC, deltaC)
                                var satN = if (maxC > 1e-4f) deltaC / maxC else 0f
                                var lumN = (maxC + minC) * 0.5f

                                var dh = 0f
                                var ds = 0f
                                var dl = 0f
                                for (bi in 0 until 8) {
                                    val wBand = hslHueWeight(hue, bandCenters[bi])
                                    if (wBand > 0f) {
                                        dh += wBand * params.hueAdjustment[bi]
                                        ds += wBand * params.saturationAdjustment[bi]
                                        dl += wBand * params.luminanceAdjustment[bi]
                                    }
                                }
                                hue = ((hue + dh * 0.32f) % 360f + 360f) % 360f
                                satN = (satN * (1f + ds / 100f)).coerceIn(0f, 1f)
                                lumN = (lumN + (dl / 100f) * 0.22f * satN).coerceIn(0f, 1f)
                                val outHsl = hslToRgb(hue, satN, lumN)
                                r = outHsl[0]; g = outHsl[1]; b = outHsl[2]
                            }

                            // Vibrance (protects skin hues 15..50 deg and already-saturated pixels)
                            if (abs(params.vibrance) > 0.1f) {
                                val curMax = max(r, max(g, b))
                                val curMin = min(r, min(g, b))
                                val curSat = if (curMax > 1e-4f) (curMax - curMin) / curMax else 0f
                                val hue = if (curMax - curMin > 1e-4f) computeHueDeg(r, g, b, curMax, curMax - curMin) else 0f
                                val skinProtect = if (hue in 12f..48f) 0.45f else 1f
                                val vAmt = (params.vibrance / 100f) * (1f - curSat * 0.7f) * skinProtect
                                val lv = 0.2126f * r + 0.7152f * g + 0.0722f * b
                                val scale = (1f + vAmt).coerceIn(0f, 2f)
                                r = (lv + (r - lv) * scale).coerceIn(0f, 1f)
                                g = (lv + (g - lv) * scale).coerceIn(0f, 1f)
                                b = (lv + (b - lv) * scale).coerceIn(0f, 1f)
                            }

                            // Global Saturation
                            if (abs(params.saturation) > 0.1f) {
                                val ls = 0.2126f * r + 0.7152f * g + 0.0722f * b
                                val sf = (1f + params.saturation / 100f).coerceIn(0f, 2f)
                                r = (ls + (r - ls) * sf).coerceIn(0f, 1f)
                                g = (ls + (g - ls) * sf).coerceIn(0f, 1f)
                                b = (ls + (b - ls) * sf).coerceIn(0f, 1f)
                            }
                        }

                        // Color Grading (Shadows / Midtones / Highlights / Global)
                        val splitPivot = (0.5f + cgBalanceShift).coerceIn(0.1f, 0.9f)
                        val shdMask = (1f - smoothstep(0f, splitPivot + cgBlend * 0.35f, lumaPost))
                        val hiMask = smoothstep(splitPivot - cgBlend * 0.35f, 1f, lumaPost)
                        val midMask = (1f - shdMask - hiMask).coerceIn(0f, 1f)

                        if (params.cgShadowSat > 0.5f || params.cgHighSat > 0.5f ||
                            params.cgMidSat > 0.5f || params.cgGlobalSat > 0.5f
                        ) {
                            r += cgShdRgb[0] * shdMask + cgMidRgb[0] * midMask + cgHiRgb[0] * hiMask + cgGlobRgb[0]
                            g += cgShdRgb[1] * shdMask + cgMidRgb[1] * midMask + cgHiRgb[1] * hiMask + cgGlobRgb[1]
                            b += cgShdRgb[2] * shdMask + cgMidRgb[2] * midMask + cgHiRgb[2] * hiMask + cgGlobRgb[2]
                        }

                        val gradeLumAdd = (params.cgShadowLum * shdMask +
                            params.cgMidLum * midMask +
                            params.cgHighLum * hiMask +
                            params.cgGlobalLum) * 0.0018f
                        r += gradeLumAdd; g += gradeLumAdd; b += gradeLumAdd

                        // Post-Crop + Lens Vignette
                        if (abs(totalVig) > 0.01f) {
                            val dx = (nx - 0.5f) * 2f
                            val dy = (ny - 0.5f) * 2f
                            val radDist = sqrt(dx * dx + dy * dy) / 1.4142135f
                            val vw = smoothstep(vigMid * 0.5f, (vigMid * 0.5f + vigFeather).coerceAtLeast(0.1f), radDist)
                            val mul = (1f + totalVig * vw * 0.85f).coerceIn(0.05f, 1.85f)
                            r *= mul; g *= mul; b *= mul
                        }

                        // Deterministic Film Grain
                        if (grainStrength > 0.001f) {
                            val cellSize = max(1, (1 + (params.grainSize / 25f).roundToInt()))
                            val gx = x / cellSize
                            val gy = y / cellSize
                            var hash = (gx * 374761393 + gy * 668265263) xor 0x5bd1e995
                            hash = (hash xor (hash ushr 13)) * 1274126177
                            val rnd = ((hash and 0xFFFF) / 32767.5f) - 1f
                            val rough = 0.65f + (params.grainRoughness / 100f) * 0.7f
                            val gAdd = rnd * grainStrength * rough * (0.4f + 0.6f * 4f * lumaPost * (1f - lumaPost))
                            r += gAdd; g += gAdd; b += gAdd
                        }

                        val ri = (r.coerceIn(0f, 1f) * 255f).roundToInt().coerceIn(0, 255)
                        val gi = (g.coerceIn(0f, 1f) * 255f).roundToInt().coerceIn(0, 255)
                        val bi = (b.coerceIn(0f, 1f) * 255f).roundToInt().coerceIn(0, 255)
                        val li = ((0.2126f * ri + 0.7152f * gi + 0.0722f * bi).roundToInt()).coerceIn(0, 255)

                        hr[ri]++
                        hg[gi]++
                        hb[bi]++
                        hl[li]++
                        outPixels[idx] = (alpha shl 24) or (ri shl 16) or (gi shl 8) or bi
                    }
                }
                LocalHistograms(hr, hg, hb, hl)
            })
        }

        val histR = IntArray(256)
        val histG = IntArray(256)
        val histB = IntArray(256)
        val histLum = IntArray(256)
        for (f in futures) {
            val hPart = f.get()
            for (i in 0 until 256) {
                histR[i] += hPart.r[i]
                histG[i] += hPart.g[i]
                histB[i] += hPart.b[i]
                histLum[i] += hPart.l[i]
            }
        }

        val outBitmap = Bitmap.createBitmap(outPixels, w, h, Bitmap.Config.ARGB_8888)
        if (workingBitmap != source && !workingBitmap.isRecycled) {
            workingBitmap.recycle()
        }
        return RenderedFrame(outBitmap, histR, histG, histB, histLum)
    }

    private class LocalHistograms(
        val r: IntArray,
        val g: IntArray,
        val b: IntArray,
        val l: IntArray
    )

    private fun prepareWorkingGeometry(
        source: Bitmap,
        params: DevelopParams,
        draftInteractive: Boolean,
        applyCrop: Boolean
    ): Bitmap {
        val hasOpticalWarp = abs(params.lensDistortion) > 0.5f ||
            abs(params.perspectiveVertical) > 0.5f ||
            abs(params.perspectiveHorizontal) > 0.5f ||
            abs(params.perspectiveAspect) > 0.5f ||
            abs(params.geometryScale - 100f) > 0.5f ||
            abs(params.geometryOffsetX) > 0.5f ||
            abs(params.geometryOffsetY) > 0.5f

        val hasRotation = abs(params.rotationDegrees) > 0.1f ||
            params.flipHorizontal || params.flipVertical ||
            (applyCrop && abs(params.cropAngle) > 0.1f) ||
            hasOpticalWarp

        val cL = if (applyCrop) params.cropLeft.coerceIn(0f, 0.92f) else 0f
        val cT = if (applyCrop) params.cropTop.coerceIn(0f, 0.92f) else 0f
        val cR = if (applyCrop) params.cropRight.coerceIn(cL + 0.08f, 1f) else 1f
        val cB = if (applyCrop) params.cropBottom.coerceIn(cT + 0.08f, 1f) else 1f
        val isCropped = applyCrop && (cL > 0.002f || cT > 0.002f || cR < 0.998f || cB < 0.998f)

        val maxPreviewDim = if (draftInteractive) 540 else Int.MAX_VALUE
        val srcW = source.width
        val srcH = source.height
        val scaleDown = if (max(srcW, srcH) > maxPreviewDim) {
            maxPreviewDim.toFloat() / max(srcW, srcH).toFloat()
        } else 1f

        if (!hasRotation && !isCropped && scaleDown >= 0.999f) {
            return source.copy(Bitmap.Config.ARGB_8888, false)
        }

        // First apply orthogonal rotation / flip
        val m = Matrix()
        m.postScale(
            if (params.flipHorizontal) -scaleDown else scaleDown,
            if (params.flipVertical) -scaleDown else scaleDown,
            srcW * 0.5f,
            srcH * 0.5f
        )
        m.postRotate(params.rotationDegrees)
        var intermediate = Bitmap.createBitmap(source, 0, 0, srcW, srcH, m, true)

        if (hasOpticalWarp) {
            val warped = applyPerspectiveAndDistortion(intermediate, params)
            if (intermediate != source && !intermediate.isRecycled) intermediate.recycle()
            intermediate = warped
        }

        val outW = ((intermediate.width * (cR - cL)).roundToInt()).coerceAtLeast(16)
        val outH = ((intermediate.height * (cB - cT)).roundToInt()).coerceAtLeast(16)

        if (!isCropped && (!applyCrop || abs(params.cropAngle) < 0.1f)) {
            return intermediate
        }

        val out = Bitmap.createBitmap(outW, outH, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(out)
        val paint = Paint(Paint.FILTER_BITMAP_FLAG or Paint.ANTI_ALIAS_FLAG)
        val drawMat = Matrix()
        val cropCx = intermediate.width * ((cL + cR) * 0.5f)
        val cropCy = intermediate.height * ((cT + cB) * 0.5f)
        drawMat.postTranslate(-cropCx, -cropCy)
        if (applyCrop && abs(params.cropAngle) > 0.1f) {
            drawMat.postRotate(-params.cropAngle)
            val rad = abs(params.cropAngle * PI / 180.0).toFloat()
            val autoZoom = cos(rad) + sin(rad) * max(outW.toFloat() / outH, outH.toFloat() / outW) * 0.45f
            drawMat.postScale(autoZoom.coerceAtLeast(1f), autoZoom.coerceAtLeast(1f))
        }
        drawMat.postTranslate(outW * 0.5f, outH * 0.5f)
        canvas.drawBitmap(intermediate, drawMat, paint)
        if (intermediate != source && !intermediate.isRecycled) intermediate.recycle()
        return out
    }

    private fun applyPerspectiveAndDistortion(src: Bitmap, params: DevelopParams): Bitmap {
        val w = src.width
        val h = src.height
        val srcPx = IntArray(w * h)
        src.getPixels(srcPx, 0, w, 0, 0, w, h)
        val dstPx = IntArray(w * h)

        val kv = params.perspectiveVertical * 0.0042f
        val kh = params.perspectiveHorizontal * 0.0042f
        val asp = (1f + params.perspectiveAspect * 0.004f).coerceIn(0.55f, 1.8f)
        val sc = (params.geometryScale / 100f).coerceIn(0.4f, 2.2f)
        val offX = params.geometryOffsetX * 0.008f
        val offY = params.geometryOffsetY * 0.008f
        val distK = params.lensDistortion * 0.0028f

        val bg = 0xFF080808.toInt()
        val halfW = (w - 1) * 0.5f
        val halfH = (h - 1) * 0.5f

        for (y in 0 until h) {
            val ny = (y / max(1, h - 1).toFloat()) * 2f - 1f
            val rowOff = y * w
            for (x in 0 until w) {
                val nx = (x / max(1, w - 1).toFloat()) * 2f - 1f
                var u = (nx - offX) / sc
                var v = (ny - offY) / sc
                u /= asp
                v *= asp

                val denom = (1f + kv * v + kh * u).coerceAtLeast(0.28f)
                var px = u / denom
                var py = v / denom

                if (abs(distK) > 0.0005f) {
                    val r2 = px * px + py * py
                    val radMul = 1f + distK * r2
                    px *= radMul
                    py *= radMul
                }

                val sx = (px + 1f) * halfW
                val sy = (py + 1f) * halfH
                if (sx < 0f || sx >= w - 1 || sy < 0f || sy >= h - 1) {
                    dstPx[rowOff + x] = bg
                } else {
                    val x0 = sx.toInt()
                    val y0 = sy.toInt()
                    val fx = sx - x0
                    val fy = sy - y0
                    val idx00 = y0 * w + x0
                    val c00 = srcPx[idx00]
                    val c10 = srcPx[idx00 + 1]
                    val c01 = srcPx[idx00 + w]
                    val c11 = srcPx[idx00 + w + 1]

                    val w00 = (1f - fx) * (1f - fy)
                    val w10 = fx * (1f - fy)
                    val w01 = (1f - fx) * fy
                    val w11 = fx * fy

                    val rr = (((c00 ushr 16) and 0xFF) * w00 + ((c10 ushr 16) and 0xFF) * w10 +
                        ((c01 ushr 16) and 0xFF) * w01 + ((c11 ushr 16) and 0xFF) * w11).roundToInt().coerceIn(0, 255)
                    val gg = (((c00 ushr 8) and 0xFF) * w00 + ((c10 ushr 8) and 0xFF) * w10 +
                        ((c01 ushr 8) and 0xFF) * w01 + ((c11 ushr 8) and 0xFF) * w11).roundToInt().coerceIn(0, 255)
                    val bb = ((c00 and 0xFF) * w00 + (c10 and 0xFF) * w10 +
                        (c01 and 0xFF) * w01 + (c11 and 0xFF) * w11).roundToInt().coerceIn(0, 255)
                    dstPx[rowOff + x] = (0xFF shl 24) or (rr shl 16) or (gg shl 8) or bb
                }
            }
        }
        return Bitmap.createBitmap(dstPx, w, h, Bitmap.Config.ARGB_8888)
    }

    private fun applyRetouchSpots(pixels: IntArray, w: Int, h: Int, params: DevelopParams) {
        for (spot in params.retouchSpots) {
            val cx = (spot.x * w).roundToInt().coerceIn(0, w - 1)
            val cy = (spot.y * h).roundToInt().coerceIn(0, h - 1)
            val rPx = (spot.radius * min(w, h)).roundToInt().coerceIn(3, 60)
            // Sample ring around the spot to fill texture-neutral patch
            val offsetX = if (cx > rPx * 2) -rPx * 2 else rPx * 2
            val alphaMul = (spot.opacity / 100f).coerceIn(0.1f, 1f)
            for (dy in -rPx..rPx) {
                for (dx in -rPx..rPx) {
                    val d = sqrt((dx * dx + dy * dy).toFloat()) / rPx
                    if (d <= 1f) {
                        val px = cx + dx
                        val py = cy + dy
                        val sx = (px + offsetX).coerceIn(0, w - 1)
                        val sy = py.coerceIn(0, h - 1)
                        if (px in 0 until w && py in 0 until h) {
                            val blend = (1f - smoothstep(0.35f, 1f, d)) * alphaMul
                            val dstC = pixels[py * w + px]
                            val srcC = pixels[sy * w + sx]
                            val rr = ((dstC ushr 16 and 0xFF) * (1f - blend) + (srcC ushr 16 and 0xFF) * blend).roundToInt()
                            val gg = ((dstC ushr 8 and 0xFF) * (1f - blend) + (srcC ushr 8 and 0xFF) * blend).roundToInt()
                            val bb = ((dstC and 0xFF) * (1f - blend) + (srcC and 0xFF) * blend).roundToInt()
                            pixels[py * w + px] = (0xFF shl 24) or (rr shl 16) or (gg shl 8) or bb
                        }
                    }
                }
            }
        }
    }

    private fun computeFastBoxBlurLuma(pixels: IntArray, w: Int, h: Int): FloatArray {
        val lum = FloatArray(w * h)
        for (i in pixels.indices) {
            val c = pixels[i]
            lum[i] = (0.2126f * ((c ushr 16) and 0xFF) +
                0.7152f * ((c ushr 8) and 0xFF) +
                0.0722f * (c and 0xFF)) / 255f
        }
        val tmp = FloatArray(w * h)
        val out = FloatArray(w * h)
        // Separable 3-tap box blur (2 passes) for fast edge-preserving local contrast
        for (y in 0 until h) {
            val row = y * w
            for (x in 0 until w) {
                val l = if (x > 0) lum[row + x - 1] else lum[row + x]
                val r = if (x + 1 < w) lum[row + x + 1] else lum[row + x]
                tmp[row + x] = (l + lum[row + x] * 2f + r) * 0.25f
            }
        }
        for (y in 0 until h) {
            val up = max(0, y - 1) * w
            val cur = y * w
            val dn = min(h - 1, y + 1) * w
            for (x in 0 until w) {
                out[cur + x] = (tmp[up + x] + tmp[cur + x] * 2f + tmp[dn + x]) * 0.25f
            }
        }
        return out
    }

    private fun sampleLut(lut: FloatArray, x: Float): Float {
        val f = (x.coerceIn(0f, 1f) * 255f)
        val i0 = f.toInt().coerceIn(0, 254)
        val frac = f - i0
        return lut[i0] + (lut[i0 + 1] - lut[i0]) * frac
    }

    private fun computeHueDeg(r: Float, g: Float, b: Float, maxC: Float, delta: Float): Float {
        val h = when {
            maxC == r -> ((g - b) / delta) % 6f
            maxC == g -> ((b - r) / delta) + 2f
            else -> ((r - g) / delta) + 4f
        } * 60f
        return if (h < 0f) h + 360f else h
    }

    private fun hslToRgb(hDeg: Float, s: Float, l: Float): FloatArray {
        val c = (1f - abs(2f * l - 1f)) * s
        val hPrime = hDeg / 60f
        val x = c * (1f - abs((hPrime % 2f) - 1f))
        val m = l - c * 0.5f
        val (r1, g1, b1) = when {
            hPrime < 1f -> Triple(c, x, 0f)
            hPrime < 2f -> Triple(x, c, 0f)
            hPrime < 3f -> Triple(0f, c, x)
            hPrime < 4f -> Triple(0f, x, c)
            hPrime < 5f -> Triple(x, 0f, c)
            else -> Triple(c, 0f, x)
        }
        return floatArrayOf(
            (r1 + m).coerceIn(0f, 1f),
            (g1 + m).coerceIn(0f, 1f),
            (b1 + m).coerceIn(0f, 1f)
        )
    }
}
