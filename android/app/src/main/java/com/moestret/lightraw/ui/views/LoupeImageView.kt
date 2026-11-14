package com.moestret.lightraw.ui.views

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.DashPathEffect
import android.graphics.Paint
import android.graphics.RectF
import android.os.Handler
import android.os.Looper
import android.util.AttributeSet
import android.view.GestureDetector
import android.view.MotionEvent
import android.view.ScaleGestureDetector
import android.view.View
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.MaskGroup
import com.moestret.lightraw.model.MaskPrimitiveKind
import com.moestret.lightraw.model.RetouchArea
import kotlin.math.abs
import kotlin.math.hypot
import kotlin.math.max
import kotlin.math.min

/**
 * Interactive Darkroom Loupe Canvas supporting:
 * - Pinch-to-Zoom (1x .. 8x), Pan, Double-Tap Zoom toggle
 * - Press-and-Hold for Before (Original) / After comparison
 * - Interactive Crop frame corner/edge dragging + rule-of-thirds overlay
 * - Interactive Mask handles (Linear Gradient / Radial / Brush stroke)
 * - Spot Heal circle tap placement
 */
class LoupeImageView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null
) : View(context, attrs) {

    enum class InteractionMode {
        VIEW,
        CROP,
        MASK,
        HEAL
    }

    var mode: InteractionMode = InteractionMode.VIEW
        set(value) {
            field = value
            resetZoom()
            invalidate()
        }

    private var editedBitmap: Bitmap? = null
    private var originalBitmap: Bitmap? = null
    private var showingBefore = false

    private var params: DevelopParams = DevelopParams()
    var activeMaskId: String? = null

    var onBeforeStateChanged: ((Boolean) -> Unit)? = null
    var onCropChanged: ((left: Float, top: Float, right: Float, bottom: Float, commit: Boolean) -> Unit)? = null
    var onMaskUpdated: ((MaskGroup, Boolean) -> Unit)? = null
    var onHealTap: ((nx: Float, ny: Float) -> Unit)? = null

    private var zoomScale = 1f
    private var panX = 0f
    private var panY = 0f

    private val imgRect = RectF()
    private val maskLinePaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = 0xFFEF4444.toInt()
        style = Paint.Style.STROKE
        strokeWidth = 3.5f
    }
    private val maskFillPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = 0x26EF4444
        style = Paint.Style.FILL
    }
    private val cropDimPaint = Paint().apply {
        color = 0x99000000.toInt()
        style = Paint.Style.FILL
    }
    private val cropBorderPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = 0xFFFFFFFF.toInt()
        style = Paint.Style.STROKE
        strokeWidth = 3.5f
    }
    private val cropGridPaint = Paint().apply {
        color = 0x77FFFFFF
        strokeWidth = 1.5f
    }
    private val healPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = 0xFF60A5FA.toInt()
        style = Paint.Style.STROKE
        strokeWidth = 3f
        pathEffect = DashPathEffect(floatArrayOf(10f, 8f), 0f)
    }
    private val imageDrawPaint = Paint(Paint.FILTER_BITMAP_FLAG)

    private var dragCorner = -1 // 0=TL,1=TR,2=BR,3=BL,4=ALL
    private var lastTouchX = 0f
    private var lastTouchY = 0f
    private var isBrushDrawing = false

    private val handler = Handler(Looper.getMainLooper())
    private var pendingLongPress = false
    private val beforeHoldRunnable = Runnable {
        if (mode == InteractionMode.VIEW && originalBitmap != null) {
            showingBefore = true
            onBeforeStateChanged?.invoke(true)
            invalidate()
        }
    }

    private val scaleDetector = ScaleGestureDetector(
        context,
        object : ScaleGestureDetector.SimpleOnScaleGestureListener() {
            override fun onScale(detector: ScaleGestureDetector): Boolean {
                if (mode != InteractionMode.VIEW) return false
                handler.removeCallbacks(beforeHoldRunnable)
                val newScale = (zoomScale * detector.scaleFactor).coerceIn(1f, 8f)
                val factor = newScale / zoomScale
                zoomScale = newScale
                if (zoomScale <= 1.02f) {
                    zoomScale = 1f
                    panX = 0f
                    panY = 0f
                } else {
                    panX = (panX - (detector.focusX - width * 0.5f)) * factor + (detector.focusX - width * 0.5f)
                    panY = (panY - (detector.focusY - height * 0.5f)) * factor + (detector.focusY - height * 0.5f)
                }
                invalidate()
                return true
            }
        }
    )

    private val gestureDetector = GestureDetector(
        context,
        object : GestureDetector.SimpleOnGestureListener() {
            override fun onDoubleTap(e: MotionEvent): Boolean {
                if (mode != InteractionMode.VIEW) return false
                handler.removeCallbacks(beforeHoldRunnable)
                if (zoomScale > 1.1f) {
                    resetZoom()
                } else {
                    zoomScale = 2.5f
                    panX = -(e.x - width * 0.5f) * 1.1f
                    panY = -(e.y - height * 0.5f) * 1.1f
                }
                invalidate()
                return true
            }

            override fun onScroll(
                e1: MotionEvent?,
                e2: MotionEvent,
                distanceX: Float,
                distanceY: Float
            ): Boolean {
                if (mode != InteractionMode.VIEW || zoomScale <= 1.02f) return false
                handler.removeCallbacks(beforeHoldRunnable)
                val maxPanX = width * (zoomScale - 1f) * 0.5f
                val maxPanY = height * (zoomScale - 1f) * 0.5f
                panX = (panX - distanceX).coerceIn(-maxPanX, maxPanX)
                panY = (panY - distanceY).coerceIn(-maxPanY, maxPanY)
                invalidate()
                return true
            }
        }
    )

    fun setImages(edited: Bitmap?, original: Bitmap?, currentParams: DevelopParams) {
        editedBitmap = edited
        if (original != null) originalBitmap = original
        params = currentParams
        invalidate()
    }

    fun updateParamsOverlay(currentParams: DevelopParams) {
        params = currentParams
        invalidate()
    }

    fun setBeforePreview(show: Boolean) {
        showingBefore = show
        onBeforeStateChanged?.invoke(show)
        invalidate()
    }

    fun resetZoom() {
        zoomScale = 1f
        panX = 0f
        panY = 0f
        invalidate()
    }

    private fun computeImageBounds(bmp: Bitmap): RectF {
        val vw = width.toFloat().coerceAtLeast(1f)
        val vh = height.toFloat().coerceAtLeast(1f)
        val pad = 16f
        val availW = (vw - pad * 2f).coerceAtLeast(1f)
        val availH = (vh - pad * 2f).coerceAtLeast(1f)
        val scale = min(availW / bmp.width, availH / bmp.height) * zoomScale
        val dw = bmp.width * scale
        val dh = bmp.height * scale
        val cx = vw * 0.5f + panX
        val cy = vh * 0.5f + panY
        imgRect.set(cx - dw * 0.5f, cy - dh * 0.5f, cx + dw * 0.5f, cy + dh * 0.5f)
        return imgRect
    }

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        val bmp = if (showingBefore && originalBitmap != null) originalBitmap else editedBitmap
        if (bmp == null || bmp.isRecycled) return

        val rect = computeImageBounds(bmp)
        canvas.drawBitmap(bmp, null, rect, imageDrawPaint)

        when (mode) {
            InteractionMode.CROP -> drawCropOverlay(canvas, rect)
            InteractionMode.MASK -> drawMaskOverlay(canvas, rect)
            InteractionMode.HEAL -> drawHealOverlay(canvas, rect)
            InteractionMode.VIEW -> Unit
        }
    }

    private fun drawCropOverlay(canvas: Canvas, rect: RectF) {
        val cl = rect.left + params.cropLeft * rect.width()
        val ct = rect.top + params.cropTop * rect.height()
        val cr = rect.left + params.cropRight * rect.width()
        val cb = rect.top + params.cropBottom * rect.height()

        // Dim outside regions
        canvas.drawRect(rect.left, rect.top, rect.right, ct, cropDimPaint)
        canvas.drawRect(rect.left, cb, rect.right, rect.bottom, cropDimPaint)
        canvas.drawRect(rect.left, ct, cl, cb, cropDimPaint)
        canvas.drawRect(cr, ct, rect.right, cb, cropDimPaint)

        // Rule of thirds
        val cw = cr - cl
        val ch = cb - ct
        for (i in 1..2) {
            canvas.drawLine(cl + cw * i / 3f, ct, cl + cw * i / 3f, cb, cropGridPaint)
            canvas.drawLine(cl, ct + ch * i / 3f, cr, ct + ch * i / 3f, cropGridPaint)
        }
        canvas.drawRect(cl, ct, cr, cb, cropBorderPaint)

        // Corner grab handles
        val handlePaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { color = 0xFF2563EB.toInt() }
        val r = 13f
        canvas.drawCircle(cl, ct, r, handlePaint)
        canvas.drawCircle(cr, ct, r, handlePaint)
        canvas.drawCircle(cr, cb, r, handlePaint)
        canvas.drawCircle(cl, cb, r, handlePaint)
    }

    private fun drawMaskOverlay(canvas: Canvas, rect: RectF) {
        val active = params.masks.firstOrNull { it.id == activeMaskId } ?: params.masks.lastOrNull() ?: return
        when (active.kind) {
            MaskPrimitiveKind.RADIAL_GRADIENT -> {
                val oval = RectF(
                    rect.left + (active.x1 - active.radiusX) * rect.width(),
                    rect.top + (active.y1 - active.radiusY) * rect.height(),
                    rect.left + (active.x1 + active.radiusX) * rect.width(),
                    rect.top + (active.y1 + active.radiusY) * rect.height()
                )
                canvas.drawOval(oval, maskFillPaint)
                canvas.drawOval(oval, maskLinePaint)
                canvas.drawCircle(oval.centerX(), oval.centerY(), 11f, maskLinePaint)
            }
            MaskPrimitiveKind.LINEAR_GRADIENT -> {
                val x1 = rect.left + active.x1 * rect.width()
                val y1 = rect.top + active.y1 * rect.height()
                val x2 = rect.left + active.x2 * rect.width()
                val y2 = rect.top + active.y2 * rect.height()
                canvas.drawLine(x1, y1, x2, y2, maskLinePaint)
                canvas.drawCircle(x1, y1, 12f, maskLinePaint)
                canvas.drawCircle(x2, y2, 12f, maskLinePaint)
            }
            MaskPrimitiveKind.BRUSH -> {
                val radPx = (0.04f + active.feather / 100f * 0.04f) * min(rect.width(), rect.height())
                for (pt in active.brushPoints) {
                    canvas.drawCircle(
                        rect.left + pt.x * rect.width(),
                        rect.top + pt.y * rect.height(),
                        radPx * 0.55f,
                        maskFillPaint
                    )
                }
            }
        }
    }

    private fun drawHealOverlay(canvas: Canvas, rect: RectF) {
        for (spot in params.retouchSpots) {
            val cx = rect.left + spot.x * rect.width()
            val cy = rect.top + spot.y * rect.height()
            val rad = spot.radius * min(rect.width(), rect.height())
            canvas.drawCircle(cx, cy, rad.coerceAtLeast(14f), healPaint)
        }
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        val bmp = editedBitmap
        if (bmp != null) computeImageBounds(bmp)

        when (mode) {
            InteractionMode.VIEW -> {
                scaleDetector.onTouchEvent(event)
                gestureDetector.onTouchEvent(event)
                when (event.actionMasked) {
                    MotionEvent.ACTION_DOWN -> {
                        if (event.pointerCount == 1) {
                            lastTouchX = event.x
                            lastTouchY = event.y
                            pendingLongPress = true
                            handler.postDelayed(beforeHoldRunnable, 240L)
                        }
                    }
                    MotionEvent.ACTION_MOVE -> {
                        if (hypot(event.x - lastTouchX, event.y - lastTouchY) > 16f || event.pointerCount > 1) {
                            pendingLongPress = false
                            handler.removeCallbacks(beforeHoldRunnable)
                        }
                    }
                    MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                        handler.removeCallbacks(beforeHoldRunnable)
                        if (showingBefore) {
                            showingBefore = false
                            onBeforeStateChanged?.invoke(false)
                            invalidate()
                        }
                    }
                }
                return true
            }
            InteractionMode.CROP -> return handleCropTouch(event)
            InteractionMode.MASK -> return handleMaskTouch(event)
            InteractionMode.HEAL -> {
                if (event.actionMasked == MotionEvent.ACTION_UP && imgRect.width() > 1f) {
                    val nx = ((event.x - imgRect.left) / imgRect.width()).coerceIn(0f, 1f)
                    val ny = ((event.y - imgRect.top) / imgRect.height()).coerceIn(0f, 1f)
                    onHealTap?.invoke(nx, ny)
                }
                return true
            }
        }
    }

    private fun handleCropTouch(event: MotionEvent): Boolean {
        if (imgRect.width() < 10f || imgRect.height() < 10f) return false
        val cl = imgRect.left + params.cropLeft * imgRect.width()
        val ct = imgRect.top + params.cropTop * imgRect.height()
        val cr = imgRect.left + params.cropRight * imgRect.width()
        val cb = imgRect.top + params.cropBottom * imgRect.height()

        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                val thresh = 48f
                dragCorner = when {
                    hypot(event.x - cl, event.y - ct) < thresh -> 0
                    hypot(event.x - cr, event.y - ct) < thresh -> 1
                    hypot(event.x - cr, event.y - cb) < thresh -> 2
                    hypot(event.x - cl, event.y - cb) < thresh -> 3
                    event.x in cl..cr && event.y in ct..cb -> 4
                    else -> -1
                }
                lastTouchX = event.x
                lastTouchY = event.y
            }
            MotionEvent.ACTION_MOVE -> {
                if (dragCorner < 0) return true
                val dx = (event.x - lastTouchX) / imgRect.width()
                val dy = (event.y - lastTouchY) / imgRect.height()
                lastTouchX = event.x
                lastTouchY = event.y
                var nl = params.cropLeft; var nt = params.cropTop
                var nr = params.cropRight; var nb = params.cropBottom
                when (dragCorner) {
                    0 -> { nl = (nl + dx).coerceIn(0f, nr - 0.12f); nt = (nt + dy).coerceIn(0f, nb - 0.12f) }
                    1 -> { nr = (nr + dx).coerceIn(nl + 0.12f, 1f); nt = (nt + dy).coerceIn(0f, nb - 0.12f) }
                    2 -> { nr = (nr + dx).coerceIn(nl + 0.12f, 1f); nb = (nb + dy).coerceIn(nt + 0.12f, 1f) }
                    3 -> { nl = (nl + dx).coerceIn(0f, nr - 0.12f); nb = (nb + dy).coerceIn(nt + 0.12f, 1f) }
                    4 -> {
                        val w = nr - nl; val h = nb - nt
                        nl = (nl + dx).coerceIn(0f, 1f - w); nr = nl + w
                        nt = (nt + dy).coerceIn(0f, 1f - h); nb = nt + h
                    }
                }
                params = params.copy(cropLeft = nl, cropTop = nt, cropRight = nr, cropBottom = nb)
                invalidate()
                onCropChanged?.invoke(nl, nt, nr, nb, false)
            }
            MotionEvent.ACTION_UP -> {
                if (dragCorner >= 0) {
                    onCropChanged?.invoke(params.cropLeft, params.cropTop, params.cropRight, params.cropBottom, true)
                }
                dragCorner = -1
            }
        }
        return true
    }

    private fun handleMaskTouch(event: MotionEvent): Boolean {
        val active = params.masks.firstOrNull { it.id == activeMaskId } ?: params.masks.lastOrNull() ?: return true
        if (imgRect.width() < 10f) return true
        val nx = ((event.x - imgRect.left) / imgRect.width()).coerceIn(0f, 1f)
        val ny = ((event.y - imgRect.top) / imgRect.height()).coerceIn(0f, 1f)

        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                if (active.kind == MaskPrimitiveKind.BRUSH) {
                    isBrushDrawing = true
                    val pts = active.brushPoints + com.moestret.lightraw.model.CurvePoint(nx, ny)
                    val upd = active.copy(brushPoints = pts)
                    updateActiveMaskInPlace(upd)
                    onMaskUpdated?.invoke(upd, false)
                } else {
                    lastTouchX = nx
                    lastTouchY = ny
                }
            }
            MotionEvent.ACTION_MOVE -> {
                val upd = when (active.kind) {
                    MaskPrimitiveKind.BRUSH -> {
                        val pts = active.brushPoints + com.moestret.lightraw.model.CurvePoint(nx, ny)
                        active.copy(brushPoints = pts)
                    }
                    MaskPrimitiveKind.RADIAL_GRADIENT -> {
                        active.copy(x1 = nx, y1 = ny)
                    }
                    MaskPrimitiveKind.LINEAR_GRADIENT -> {
                        if (abs(nx - active.x1) + abs(ny - active.y1) < abs(nx - active.x2) + abs(ny - active.y2)) {
                            active.copy(x1 = nx, y1 = ny)
                        } else {
                            active.copy(x2 = nx, y2 = ny)
                        }
                    }
                }
                updateActiveMaskInPlace(upd)
                onMaskUpdated?.invoke(upd, false)
            }
            MotionEvent.ACTION_UP -> {
                isBrushDrawing = false
                val cur = params.masks.firstOrNull { it.id == active.id } ?: active
                onMaskUpdated?.invoke(cur, true)
            }
        }
        return true
    }

    private fun updateActiveMaskInPlace(updated: MaskGroup) {
        params = params.copy(
            masks = params.masks.map { if (it.id == updated.id) updated else it }
        )
        invalidate()
    }
}
