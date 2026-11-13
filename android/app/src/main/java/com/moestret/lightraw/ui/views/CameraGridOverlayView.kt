package com.moestret.lightraw.ui.views

import android.content.Context
import android.graphics.Canvas
import android.graphics.Paint
import android.util.AttributeSet
import android.view.View

/**
 * Rule-of-thirds viewfinder composition grid overlay for ProCameraActivity.
 */
class CameraGridOverlayView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null
) : View(context, attrs) {

    private val linePaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = 0x55FFFFFF
        strokeWidth = resources.displayMetrics.density
    }

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        val w = width.toFloat()
        val h = height.toFloat()
        if (w <= 0f || h <= 0f) return

        val thirdW = w / 3f
        val thirdH = h / 3f
        canvas.drawLine(thirdW, 0f, thirdW, h, linePaint)
        canvas.drawLine(thirdW * 2f, 0f, thirdW * 2f, h, linePaint)
        canvas.drawLine(0f, thirdH, w, thirdH, linePaint)
        canvas.drawLine(0f, thirdH * 2f, w, thirdH * 2f, linePaint)
    }
}
