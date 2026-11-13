package com.moestret.lightraw.ui

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Rect
import android.graphics.RectF
import android.os.Bundle
import android.widget.SeekBar
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import com.moestret.lightraw.data.CatalogDbHelper
import com.moestret.lightraw.data.MediaStoreImporter
import com.moestret.lightraw.databinding.ActivityCollageBinding
import com.moestret.lightraw.engine.KotlinPv2012Engine
import com.moestret.lightraw.model.ExportConfig
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlin.math.ceil
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sqrt

/**
 * S10 Multi-Photo Collage Layout Builder & Gallery Exporter (1:1 / 4:5 / 9:16).
 */
class CollageActivity : AppCompatActivity() {

    private lateinit var binding: ActivityCollageBinding
    private lateinit var catalog: CatalogDbHelper
    private val scope = CoroutineScope(Dispatchers.Main + Job())

    private var sourceBitmaps: List<Bitmap> = emptyList()
    private var ratioW = 4f
    private var ratioH = 5f
    private var gapPx = 20f
    private var bgDark = true
    private var currentPreview: Bitmap? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityCollageBinding.inflate(layoutInflater)
        setContentView(binding.root)
        catalog = CatalogDbHelper(this)

        val ids = intent.getLongArrayExtra("photo_ids") ?: longArrayOf()
        binding.btnCollageBack.setOnClickListener { finish() }

        binding.btnRatio11.setOnClickListener { ratioW = 1f; ratioH = 1f; renderCollage(false) }
        binding.btnRatio45.setOnClickListener { ratioW = 4f; ratioH = 5f; renderCollage(false) }
        binding.btnRatio916.setOnClickListener { ratioW = 9f; ratioH = 16f; renderCollage(false) }
        binding.btnBgToggle.setOnClickListener { bgDark = !bgDark; renderCollage(false) }

        binding.sbCollageGap.setOnSeekBarChangeListener(object : SeekBar.OnSeekBarChangeListener {
            override fun onProgressChanged(seekBar: SeekBar?, progress: Int, fromUser: Boolean) {
                gapPx = progress.toFloat()
                if (fromUser) renderCollage(false)
            }
            override fun onStartTrackingTouch(seekBar: SeekBar?) = Unit
            override fun onStopTrackingTouch(seekBar: SeekBar?) = Unit
        })

        binding.btnCollageExport.setOnClickListener { renderCollage(export = true) }

        scope.launch {
            sourceBitmaps = withContext(Dispatchers.Default) {
                ids.toList().mapNotNull { id ->
                    val asset = catalog.getPhotoById(id) ?: return@mapNotNull null
                    val raw = MediaStoreImporter.decodeScaledBitmap(this@CollageActivity, asset.uriString, 1080)
                        ?: return@mapNotNull null
                    val r = KotlinPv2012Engine.render(raw, asset.params, draftInteractive = true, applyCrop = true)
                    raw.recycle()
                    r.bitmap
                }
            }
            renderCollage(false)
        }
    }

    private fun renderCollage(export: Boolean) {
        if (sourceBitmaps.isEmpty()) return
        scope.launch {
            val result = withContext(Dispatchers.Default) {
                val outW = if (export) 1600 else 800
                val outH = (outW * (ratioH / ratioW)).toInt().coerceAtLeast(400)
                val out = Bitmap.createBitmap(outW, outH, Bitmap.Config.ARGB_8888)
                val canvas = Canvas(out)
                canvas.drawColor(if (bgDark) 0xFF121212.toInt() else 0xFFF8FAFC.toInt())

                val n = sourceBitmaps.size
                val cols = ceil(sqrt(n.toDouble())).toInt().coerceAtLeast(1)
                val rows = ceil(n.toDouble() / cols).toInt().coerceAtLeast(1)
                val gap = gapPx * (outW / 800f)

                val cellW = (outW - gap * (cols + 1)) / cols
                val cellH = (outH - gap * (rows + 1)) / rows
                val paint = Paint(Paint.ANTI_ALIAS_FLAG or Paint.FILTER_BITMAP_FLAG)

                sourceBitmaps.forEachIndexed { idx, bmp ->
                    val r = idx / cols
                    val c = idx % cols
                    val left = gap + c * (cellW + gap)
                    val top = gap + r * (cellH + gap)
                    val dst = RectF(left, top, left + cellW, top + cellH)
                    val srcRect = centerCropRect(bmp.width, bmp.height, cellW / max(1f, cellH))
                    canvas.drawBitmap(bmp, srcRect, dst, paint)
                }
                if (export) {
                    MediaStoreImporter.exportToGallery(
                        this@CollageActivity,
                        out,
                        "Collage_${n}P",
                        ExportConfig()
                    )
                }
                out
            }
            if (export) {
                Toast.makeText(this@CollageActivity, "Collage saved to Pictures/LightRaw", Toast.LENGTH_LONG).show()
                result.recycle()
            } else {
                val prev = currentPreview
                currentPreview = result
                binding.ivCollagePreview.setImageBitmap(result)
                if (prev != null && prev != result && !prev.isRecycled) {
                    prev.recycle()
                }
            }
        }
    }

    private fun centerCropRect(bw: Int, bh: Int, targetRatio: Float): Rect {
        val br = bw.toFloat() / max(1, bh)
        return if (br > targetRatio) {
            val cropW = (bh * targetRatio).toInt()
            val x = (bw - cropW) / 2
            Rect(x, 0, x + cropW, bh)
        } else {
            val cropH = (bw / targetRatio).toInt()
            val y = (bh - cropH) / 2
            Rect(0, y, bw, y + cropH)
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        sourceBitmaps.forEach { if (!it.isRecycled) it.recycle() }
        currentPreview?.recycle()
    }
}
