package com.moestret.lightraw.ui.views

import android.content.Context
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.Path
import android.util.AttributeSet
import android.view.MotionEvent
import android.view.View
import com.moestret.lightraw.engine.KotlinPv2012Engine
import com.moestret.lightraw.model.CurvePoint
import kotlin.math.abs
import kotlin.math.hypot

/**
 * Interactive multi-node Tone Curve canvas for Luma, Red, Green, or Blue channels.
 */
class ToneCurveView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null
) : View(context, attrs) {

    var curvePoints: List<CurvePoint> = listOf(CurvePoint(0f, 0f), CurvePoint(1f, 1f))
        set(value) {
            field = value.sortedBy { it.x }
            invalidate()
        }

    var strokeColor: Int = Color.WHITE
        set(value) {
            field = value
            curvePaint.color = value
            invalidate()
        }

    var onCurveChanged: ((List<CurvePoint>, Boolean) -> Unit)? = null

    private var activeIndex = -1
    private var lastTapTime = 0L
    private var lastTapIdx = -1

    private val bgPaint = Paint().apply {
        color = 0xFF141414.toInt()
        style = Paint.Style.FILL
    }
    private val gridPaint = Paint().apply {
        color = 0xFF2D2D2D.toInt()
        strokeWidth = 2f
    }
    private val diagPaint = Paint().apply {
        color = 0xFF3A3A3A.toInt()
        strokeWidth = 2f
    }
    private val curvePaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = Color.WHITE
        style = Paint.Style.STROKE
        strokeWidth = 4.5f
    }
    private val nodePaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = Color.WHITE
        style = Paint.Style.FILL
    }
    private val nodeActivePaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = 0xFF2563EB.toInt()
        style = Paint.Style.FILL
    }
    private val path = Path()

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        val w = width.toFloat()
        val h = height.toFloat()
        val pad = 16f
        val plotW = (w - pad * 2f).coerceAtLeast(10f)
        val plotH = (h - pad * 2f).coerceAtLeast(10f)

        canvas.drawRoundRect(0f, 0f, w, h, 16f, 16f, bgPaint)

        for (i in 0..4) {
            val fx = pad + plotW * (i / 4f)
            val fy = pad + plotH * (i / 4f)
            canvas.drawLine(fx, pad, fx, pad + plotH, gridPaint)
            canvas.drawLine(pad, fy, pad + plotW, fy, gridPaint)
        }
        canvas.drawLine(pad, pad + plotH, pad + plotW, pad, diagPaint)

        val lut = KotlinPv2012Engine.buildCurveLut(curvePoints)
        path.reset()
        for (i in 0 until 256 step 2) {
            val x = pad + (i / 255f) * plotW
            val y = pad + (1f - lut[i]) * plotH
            if (i == 0) path.moveTo(x, y) else path.lineTo(x, y)
        }
        canvas.drawPath(path, curvePaint)

        for (i in curvePoints.indices) {
            val pt = curvePoints[i]
            val cx = pad + pt.x * plotW
            val cy = pad + (1f - pt.y) * plotH
            canvas.drawCircle(cx, cy, if (i == activeIndex) 13f else 10f, if (i == activeIndex) nodeActivePaint else nodePaint)
        }
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        val w = width.toFloat()
        val h = height.toFloat()
        val pad = 16f
        val plotW = (w - pad * 2f).coerceAtLeast(10f)
        val plotH = (h - pad * 2f).coerceAtLeast(10f)

        val nx = ((event.x - pad) / plotW).coerceIn(0f, 1f)
        val ny = (1f - (event.y - pad) / plotH).coerceIn(0f, 1f)

        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                parent.requestDisallowInterceptTouchEvent(true)
                val hitThresholdPx = 38f
                var hit = -1
                var bestDist = Float.MAX_VALUE
                for (i in curvePoints.indices) {
                    val px = pad + curvePoints[i].x * plotW
                    val py = pad + (1f - curvePoints[i].y) * plotH
                    val d = hypot(event.x - px, event.y - py)
                    if (d < hitThresholdPx && d < bestDist) {
                        bestDist = d
                        hit = i
                    }
                }
                val now = System.currentTimeMillis()
                if (hit > 0 && hit < curvePoints.size - 1 && hit == lastTapIdx && now - lastTapTime < 320L) {
                    // Double tap removes interior node
                    val updated = curvePoints.toMutableList().apply { removeAt(hit) }
                    curvePoints = updated
                    activeIndex = -1
                    onCurveChanged?.invoke(updated, true)
                    return true
                }
                lastTapTime = now
                lastTapIdx = hit

                if (hit >= 0) {
                    activeIndex = hit
                } else if (curvePoints.size < 12) {
                    val inserted = curvePoints.toMutableList()
                    inserted.add(CurvePoint(nx, ny))
                    val sorted = inserted.sortedBy { it.x }
                    activeIndex = sorted.indexOfFirst { abs(it.x - nx) < 1e-4f && abs(it.y - ny) < 1e-4f }
                    curvePoints = sorted
                    onCurveChanged?.invoke(sorted, false)
                }
                invalidate()
                return true
            }
            MotionEvent.ACTION_MOVE -> {
                if (activeIndex >= 0 && activeIndex < curvePoints.size) {
                    val mutable = curvePoints.toMutableList()
                    val lockedX = when (activeIndex) {
                        0 -> nx.coerceIn(0f, (mutable.getOrNull(1)?.x ?: 0.9f) - 0.02f)
                        mutable.lastIndex -> nx.coerceIn((mutable.getOrNull(mutable.lastIndex - 1)?.x ?: 0.1f) + 0.02f, 1f)
                        else -> {
                            val minX = mutable[activeIndex - 1].x + 0.02f
                            val maxX = mutable[activeIndex + 1].x - 0.02f
                            nx.coerceIn(minX, max(minX, maxX))
                        }
                    }
                    mutable[activeIndex] = CurvePoint(lockedX, ny)
                    curvePoints = mutable
                    onCurveChanged?.invoke(mutable, false)
                }
                return true
            }
            MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                parent.requestDisallowInterceptTouchEvent(false)
                if (activeIndex >= 0) {
                    onCurveChanged?.invoke(curvePoints, true)
                }
                activeIndex = -1
                invalidate()
                return true
            }
        }
        return super.onTouchEvent(event)
    }

    private fun max(a: Float, b: Float) = if (a > b) a else b
}
