package com.moestret.lightraw.ui.views

import android.content.Context
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.Path
import android.util.AttributeSet
import android.view.View
import kotlin.math.max

/**
 * Compact real-time RGB + Luminance histogram HUD view for the Loupe editor.
 */
class HistogramView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null
) : View(context, attrs) {

    private var histR = IntArray(256)
    private var histG = IntArray(256)
    private var histB = IntArray(256)
    private var histLum = IntArray(256)

    private val bgPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = 0xCC121212.toInt()
        style = Paint.Style.FILL
    }
    private val borderPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = 0xFF333333.toInt()
        style = Paint.Style.STROKE
        strokeWidth = 2f
    }
    private val gridPaint = Paint().apply {
        color = 0x33FFFFFF
        strokeWidth = 1f
    }

    private fun channelPaint(color: Int) = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        this.color = color
        style = Paint.Style.FILL
    }

    private val pR = channelPaint(0x66EF4444)
    private val pG = channelPaint(0x6634D399)
    private val pB = channelPaint(0x663B82F6)
    private val pL = channelPaint(0x88E5E7EB.toInt())
    private val path = Path()

    fun updateHistograms(r: IntArray, g: IntArray, b: IntArray, lum: IntArray) {
        histR = r
        histG = g
        histB = b
        histLum = lum
        postInvalidate()
    }

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        val w = width.toFloat()
        val h = height.toFloat()
        if (w <= 4f || h <= 4f) return

        canvas.drawRoundRect(1f, 1f, w - 1f, h - 1f, 12f, 12f, bgPaint)
        canvas.drawRoundRect(1f, 1f, w - 1f, h - 1f, 12f, 12f, borderPaint)

        for (i in 1..3) {
            val gx = w * (i / 4f)
            canvas.drawLine(gx, 4f, gx, h - 4f, gridPaint)
        }

        // Ignore extreme 0 and 255 spikes slightly so midtone shape stays readable
        var peak = 1
        for (i in 4..251) {
            peak = max(peak, max(histLum[i], max(histR[i], max(histG[i], histB[i]))))
        }
        val invPeak = 1f / peak.toFloat()

        drawChannel(canvas, histR, invPeak, w, h, pR)
        drawChannel(canvas, histG, invPeak, w, h, pG)
        drawChannel(canvas, histB, invPeak, w, h, pB)
        drawChannel(canvas, histLum, invPeak, w, h, pL)

        // Shadow & Highlight clipping indicator triangles
        val shdClip = histLum[0] > peak * 0.45f
        val hiClip = histLum[255] > peak * 0.45f
        val clipPaint = Paint(Paint.ANTI_ALIAS_FLAG)
        clipPaint.color = if (shdClip) 0xFF3B82F6.toInt() else 0x55888888
        canvas.drawCircle(10f, 10f, 4f, clipPaint)
        clipPaint.color = if (hiClip) 0xFFF87171.toInt() else 0x55888888
        canvas.drawCircle(w - 10f, 10f, 4f, clipPaint)
    }

    private fun drawChannel(
        canvas: Canvas,
        bins: IntArray,
        invPeak: Float,
        w: Float,
        h: Float,
        paint: Paint
    ) {
        val pad = 4f
        val plotH = h - pad * 2f
        val plotW = w - pad * 2f
        path.reset()
        path.moveTo(pad, h - pad)
        for (i in 0 until 256 step 2) {
            val x = pad + (i / 255f) * plotW
            val norm = (bins[i] * invPeak).coerceIn(0f, 1f)
            val y = h - pad - norm * plotH
            path.lineTo(x, y)
        }
        path.lineTo(w - pad, h - pad)
        path.close()
        canvas.drawPath(path, paint)
    }
}
