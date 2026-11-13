package com.moestret.lightraw.engine

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Matrix
import android.graphics.Paint
import android.graphics.Rect
import android.graphics.RectF
import android.graphics.Typeface
import com.moestret.lightraw.R
import com.moestret.lightraw.data.MediaStoreImporter
import com.moestret.lightraw.model.WatermarkAnchor
import com.moestret.lightraw.model.WatermarkSettings
import com.moestret.lightraw.model.WatermarkSizeMode
import com.moestret.lightraw.model.WatermarkStyle
import kotlin.math.max
import kotlin.math.min
import kotlin.math.roundToInt

/**
 * Renders non-destructive Text and Graphic (PNG/JPG/WebP) watermarks using Android Canvas
 * across the 3x3 anchor grid with Proportional/Fit/Fill sizing, rotation, opacity, and shadow.
 */
object WatermarkRenderer {

    const val BUILTIN_LR_MARK_URI = "builtin://lightraw_mark"

    /** Pure-JVM coordinate helper computing anchored `(left, top, width, height)` box. */
    fun computeGraphicBox(
        targetW: Int,
        targetH: Int,
        graphicW: Int,
        graphicH: Int,
        settings: WatermarkSettings
    ): FloatArray {
        val tw = targetW.coerceAtLeast(32).toFloat()
        val th = targetH.coerceAtLeast(32).toFloat()
        val gw = graphicW.coerceAtLeast(1).toFloat()
        val gh = graphicH.coerceAtLeast(1).toFloat()
        val (boxW, boxH) = when (settings.sizeMode) {
            WatermarkSizeMode.PROPORTIONAL -> {
                val pct = (settings.sizePercent / 100f).coerceIn(0.02f, 1f)
                val w = (tw * pct).coerceAtLeast(16f)
                val h = (w * (gh / gw)).coerceAtLeast(16f)
                w to h
            }
            WatermarkSizeMode.FIT -> {
                val scale = min((tw * 0.95f) / gw, (th * 0.95f) / gh)
                (gw * scale).coerceAtLeast(16f) to (gh * scale).coerceAtLeast(16f)
            }
            WatermarkSizeMode.FILL -> {
                val scale = max(tw / gw, th / gh)
                (gw * scale).coerceAtLeast(16f) to (gh * scale).coerceAtLeast(16f)
            }
        }
        val (x, y) = computeAnchorOrigin(tw, th, boxW, boxH, settings.anchor, settings.insetPercent)
        return floatArrayOf(x, y, boxW, boxH)
    }

    /** Shared 3x3 anchor origin calculation with inset percentage margins. */
    fun computeAnchorOrigin(
        canvasW: Float,
        canvasH: Float,
        elemW: Float,
        elemH: Float,
        anchor: WatermarkAnchor,
        insetPercent: Float
    ): Pair<Float, Float> {
        val insetX = canvasW * (insetPercent / 100f)
        val insetY = canvasH * (insetPercent / 100f)
        val x = when (anchor) {
            WatermarkAnchor.TOP_LEFT, WatermarkAnchor.CENTER_LEFT, WatermarkAnchor.BOTTOM_LEFT -> insetX
            WatermarkAnchor.TOP_CENTER, WatermarkAnchor.CENTER, WatermarkAnchor.BOTTOM_CENTER ->
                (canvasW - elemW) * 0.5f
            WatermarkAnchor.TOP_RIGHT, WatermarkAnchor.CENTER_RIGHT, WatermarkAnchor.BOTTOM_RIGHT ->
                canvasW - elemW - insetX
        }
        val y = when (anchor) {
            WatermarkAnchor.TOP_LEFT, WatermarkAnchor.TOP_CENTER, WatermarkAnchor.TOP_RIGHT -> insetY
            WatermarkAnchor.CENTER_LEFT, WatermarkAnchor.CENTER, WatermarkAnchor.CENTER_RIGHT ->
                (canvasH - elemH) * 0.5f
            WatermarkAnchor.BOTTOM_LEFT, WatermarkAnchor.BOTTOM_CENTER, WatermarkAnchor.BOTTOM_RIGHT ->
                canvasH - elemH - insetY
        }
        return x to y
    }

    fun loadGraphicBitmap(context: Context, uriString: String?, maxEdge: Int = 1024): Bitmap? {
        if (uriString.isNullOrBlank()) return null
        if (uriString == BUILTIN_LR_MARK_URI) {
            return BitmapFactory.decodeResource(context.resources, R.drawable.ic_lightraw_mark)
        }
        return MediaStoreImporter.decodeScaledBitmap(context, uriString, maxEdge)
    }

    fun applyWatermarkWithContext(
        context: Context,
        source: Bitmap,
        settings: WatermarkSettings,
        maxGraphicEdge: Int = 2048
    ): Bitmap {
        if (!settings.enabled) return source
        if (settings.style == WatermarkStyle.GRAPHIC) {
            val loaded = loadGraphicBitmap(context, settings.graphicUriString, maxGraphicEdge)
                ?: return source
            val result = applyWatermark(source, settings, loaded)
            if (!loaded.isRecycled) loaded.recycle()
            return result
        }
        return applyWatermark(source, settings, null)
    }

    fun applyWatermark(
        source: Bitmap,
        settings: WatermarkSettings,
        graphicBitmap: Bitmap? = null
    ): Bitmap {
        if (!settings.enabled) return source
        return when (settings.style) {
            WatermarkStyle.GRAPHIC -> applyGraphicWatermark(source, settings, graphicBitmap)
            WatermarkStyle.TEXT -> applyTextWatermark(source, settings)
        }
    }

    private fun applyGraphicWatermark(
        source: Bitmap,
        settings: WatermarkSettings,
        graphicBitmap: Bitmap?
    ): Bitmap {
        val logo = graphicBitmap ?: return source
        if (logo.isRecycled || logo.width <= 0 || logo.height <= 0) return source

        val rotDeg = ((settings.rotationDegrees % 360) + 360) % 360
        val oriented = if (rotDeg != 0) {
            val m = Matrix().apply { postRotate(rotDeg.toFloat()) }
            Bitmap.createBitmap(logo, 0, 0, logo.width, logo.height, m, true)
        } else {
            logo
        }

        val out = source.copy(Bitmap.Config.ARGB_8888, true)
        val canvas = Canvas(out)
        val box = computeGraphicBox(out.width, out.height, oriented.width, oriented.height, settings)
        val alphaInt = ((settings.opacityPercent / 100f) * 255f).roundToInt().coerceIn(0, 255)
        val paint = Paint(Paint.ANTI_ALIAS_FLAG or Paint.FILTER_BITMAP_FLAG).apply {
            alpha = alphaInt
        }
        val srcRect = Rect(0, 0, oriented.width, oriented.height)
        val dstRect = RectF(box[0], box[1], box[0] + box[2], box[1] + box[3])
        canvas.drawBitmap(oriented, srcRect, dstRect, paint)
        if (oriented != logo && !oriented.isRecycled) oriented.recycle()
        return out
    }

    private fun applyTextWatermark(source: Bitmap, settings: WatermarkSettings): Bitmap {
        if (settings.text.isBlank()) return source
        val out = source.copy(Bitmap.Config.ARGB_8888, true)
        val canvas = Canvas(out)
        val text = settings.text.trim()

        val refDim = minOf(out.width, out.height).toFloat().coerceAtLeast(100f)
        var textSizePx = (refDim * (settings.sizePercent / 100f) * 0.72f).coerceIn(14f, 420f)
        val alphaInt = ((settings.opacityPercent / 100f) * 255f).roundToInt().coerceIn(0, 255)

        val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
            color = Color.argb(alphaInt, 255, 255, 255)
            textSize = textSizePx
            typeface = Typeface.create(Typeface.SANS_SERIF, Typeface.BOLD)
            letterSpacing = 0.04f
        }

        val measuredW = paint.measureText(text).coerceAtLeast(1f)
        if (settings.sizeMode == WatermarkSizeMode.FIT) {
            textSizePx = (textSizePx * ((out.width * 0.88f) / measuredW)).coerceIn(14f, 640f)
            paint.textSize = textSizePx
        } else if (settings.sizeMode == WatermarkSizeMode.FILL) {
            textSizePx = (textSizePx * ((out.width * 0.98f) / measuredW)).coerceIn(14f, 800f)
            paint.textSize = textSizePx
        }

        if (settings.shadowEnabled) {
            paint.setShadowLayer(
                (textSizePx * 0.14f).coerceAtLeast(2f),
                2f,
                2f,
                Color.argb((alphaInt * 0.78f).toInt().coerceIn(0, 255), 0, 0, 0)
            )
        }

        val textWidth = paint.measureText(text)
        val fm = paint.fontMetrics
        val textHeight = fm.descent - fm.ascent
        val rotDeg = ((settings.rotationDegrees % 360) + 360) % 360
        val effW = if (rotDeg == 90 || rotDeg == 270) textHeight else textWidth
        val effH = if (rotDeg == 90 || rotDeg == 270) textWidth else textHeight
        val (x, yTop) = computeAnchorOrigin(
            out.width.toFloat(),
            out.height.toFloat(),
            effW,
            effH,
            settings.anchor,
            settings.insetPercent
        )

        canvas.save()
        if (rotDeg != 0) {
            canvas.rotate(rotDeg.toFloat(), x + effW * 0.5f, yTop + effH * 0.5f)
            val dx = x + (effW - textWidth) * 0.5f
            val dy = yTop + (effH - textHeight) * 0.5f
            canvas.drawText(text, dx, dy - fm.ascent, paint)
        } else {
            canvas.drawText(text, x, yTop - fm.ascent, paint)
        }
        canvas.restore()
        return out
    }
}
