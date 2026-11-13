package com.moestret.lightraw.ui.views

import android.annotation.SuppressLint
import android.content.Context
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.RadialGradient
import android.graphics.Shader
import android.graphics.SweepGradient
import android.util.AttributeSet
import android.view.MotionEvent
import android.view.View
import kotlin.math.PI
import kotlin.math.atan2
import kotlin.math.cos
import kotlin.math.min
import kotlin.math.sin
import kotlin.math.sqrt

/**
 * Interactive 2D Circular Color Grading Wheel (Hue 0..360 deg + Saturation 0..100%)
 * inspired by Lightroom Mobile's 3-Way Color Grading interface.
 */
class ColorWheelView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null
) : View(context, attrs) {

    var hueDeg: Float = 0f
        set(value) {
            field = ((value % 360f) + 360f) % 360f
            invalidate()
        }

    var saturation: Float = 0f
        set(value) {
            field = value.coerceIn(0f, 100f)
            invalidate()
        }

    var onColorChanged: ((hueDeg: Float, saturation: Float, commit: Boolean) -> Unit)? = null

    private var wheelCx = 0f
    private var wheelCy = 0f
    private var wheelR = 1f
    private var lastTapMs = 0L

    private val sweepPaint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val satFadePaint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val ringPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = 0xFF444444.toInt()
        style = Paint.Style.STROKE
        strokeWidth = 3f
    }
    private val crosshairPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = 0x55FFFFFF
        style = Paint.Style.STROKE
        strokeWidth = 1.5f
    }
    private val puckFillPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.FILL
    }
    private val puckStrokePaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = 0xFFFFFFFF.toInt()
        style = Paint.Style.STROKE
        strokeWidth = 4f
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        wheelCx = w * 0.5f
        wheelCy = h * 0.5f
        wheelR = (min(w, h) * 0.5f - 14f).coerceAtLeast(16f)

        val sweepColors = intArrayOf(
            0xFFFF3B30.toInt(), // 0 deg Red
            0xFFFFCC00.toInt(), // 60 deg Yellow
            0xFF34C759.toInt(), // 120 deg Green
            0xFF00C7BE.toInt(), // 180 deg Cyan
            0xFF2563EB.toInt(), // 240 deg Blue
            0xFFAF52DE.toInt(), // 300 deg Magenta
            0xFFFF3B30.toInt()  // 360 deg Red
        )
        sweepPaint.shader = SweepGradient(wheelCx, wheelCy, sweepColors, null)
        satFadePaint.shader = RadialGradient(
            wheelCx, wheelCy, wheelR,
            intArrayOf(0xFF2C2C2C.toInt(), 0x002C2C2C),
            floatArrayOf(0f, 1f),
            Shader.TileMode.CLAMP
        )
    }

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        canvas.drawCircle(wheelCx, wheelCy, wheelR, sweepPaint)
        canvas.drawCircle(wheelCx, wheelCy, wheelR, satFadePaint)
        canvas.drawCircle(wheelCx, wheelCy, wheelR, ringPaint)
        canvas.drawCircle(wheelCx, wheelCy, wheelR * 0.5f, crosshairPaint)
        canvas.drawLine(wheelCx - 8f, wheelCy, wheelCx + 8f, wheelCy, crosshairPaint)
        canvas.drawLine(wheelCx, wheelCy - 8f, wheelCx, wheelCy + 8f, crosshairPaint)

        val rad = (hueDeg * PI / 180.0).toFloat()
        val dist = (saturation / 100f) * wheelR
        val px = wheelCx + cos(rad) * dist
        val py = wheelCy + sin(rad) * dist

        puckFillPaint.color = Color.HSVToColor(floatArrayOf(hueDeg, (saturation / 100f).coerceIn(0f, 1f), 1f))
        canvas.drawCircle(px, py, 12f, puckFillPaint)
        canvas.drawCircle(px, py, 12f, puckStrokePaint)
    }

    @SuppressLint("ClickableViewAccessibility")
    override fun onTouchEvent(event: MotionEvent): Boolean {
        parent?.requestDisallowInterceptTouchEvent(true)
        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                val now = System.currentTimeMillis()
                if (now - lastTapMs < 260L) {
                    hueDeg = 0f
                    saturation = 0f
                    onColorChanged?.invoke(0f, 0f, true)
                    lastTapMs = 0L
                    return true
                }
                lastTapMs = now
                updateFromTouch(event.x, event.y, commit = false)
                return true
            }
            MotionEvent.ACTION_MOVE -> {
                updateFromTouch(event.x, event.y, commit = false)
                return true
            }
            MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                updateFromTouch(event.x, event.y, commit = true)
                return true
            }
        }
        return super.onTouchEvent(event)
    }

    private fun updateFromTouch(x: Float, y: Float, commit: Boolean) {
        val dx = x - wheelCx
        val dy = y - wheelCy
        val r = sqrt(dx * dx + dy * dy).coerceAtMost(wheelR)
        var angle = Math.toDegrees(atan2(dy.toDouble(), dx.toDouble())).toFloat()
        if (angle < 0f) angle += 360f
        hueDeg = angle
        saturation = ((r / wheelR) * 100f).coerceIn(0f, 100f)
        onColorChanged?.invoke(hueDeg, saturation, commit)
    }
}
