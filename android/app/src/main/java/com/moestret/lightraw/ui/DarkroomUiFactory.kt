package com.moestret.lightraw.ui

import android.app.AlertDialog
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.Color
import android.graphics.Typeface
import android.net.Uri
import android.view.Gravity
import android.view.View
import android.widget.Button
import android.widget.CheckBox
import android.widget.EditText
import android.widget.GridLayout
import android.widget.ImageView
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.SeekBar
import android.widget.Switch
import android.widget.TextView
import android.widget.Toast
import com.moestret.lightraw.R
import com.moestret.lightraw.data.MediaStoreImporter
import com.moestret.lightraw.engine.WatermarkRenderer
import com.moestret.lightraw.model.CopySettingsMask
import com.moestret.lightraw.model.ExportConfig
import com.moestret.lightraw.model.ExportFormat
import com.moestret.lightraw.model.ExportResize
import com.moestret.lightraw.model.WatermarkAnchor
import com.moestret.lightraw.model.WatermarkSettings
import com.moestret.lightraw.model.WatermarkSizeMode
import com.moestret.lightraw.model.WatermarkStyle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.util.Locale
import kotlin.math.roundToInt

/**
 * Reusable Material 3 darkroom slider widgets & modal sheets (Copy Settings, Watermark + Export).
 */
object DarkroomUiFactory {

    fun dp(context: Context, value: Int): Int =
        (value * context.resources.displayMetrics.density).roundToInt()

    fun addSectionHeader(parent: LinearLayout, title: String) {
        val tv = TextView(parent.context).apply {
            text = title
            setTextColor(parent.context.getColor(R.color.lr_text_muted))
            textSize = 11f
            setTypeface(typeface, Typeface.BOLD)
            setPadding(0, dp(parent.context, 8), 0, dp(parent.context, 4))
        }
        parent.addView(tv)
    }

    fun addSliderRow(
        parent: LinearLayout,
        label: String,
        min: Float,
        max: Float,
        defaultVal: Float,
        currentVal: Float,
        decimals: Int = 0,
        unitSuffix: String = "",
        onChanged: (Float, Boolean) -> Unit
    ): SeekBar {
        val ctx = parent.context
        val row = LinearLayout(ctx).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(0, dp(ctx, 2), 0, dp(ctx, 4))
        }

        val topLine = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
        }

        val tvLabel = TextView(ctx).apply {
            text = label
            setTextColor(ctx.getColor(R.color.lr_text))
            textSize = 13f
            layoutParams = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f)
        }

        fun fmt(v: Float): String {
            val s = if (decimals <= 0) "${v.roundToInt()}" else "%.${decimals}f".format(Locale.US, v)
            val plus = if (min < 0f && v > 0.001f) "+" else ""
            return "$plus$s$unitSuffix"
        }

        val tvVal = TextView(ctx).apply {
            text = fmt(currentVal)
            val changed = kotlin.math.abs(currentVal - defaultVal) > 0.01f
            setTextColor(ctx.getColor(if (changed) R.color.lr_accent else R.color.lr_text_muted))
            textSize = 12f
            setPadding(dp(ctx, 10), dp(ctx, 4), dp(ctx, 4), dp(ctx, 4))
        }

        topLine.addView(tvLabel)
        topLine.addView(tvVal)
        row.addView(topLine)

        val steps = 400
        fun toProg(v: Float): Int = (((v - min) / (max - min).coerceAtLeast(1e-4f)) * steps)
            .roundToInt().coerceIn(0, steps)

        val sb = SeekBar(ctx).apply {
            this.max = steps
            progress = toProg(currentVal)
            minimumHeight = dp(ctx, 36)
        }

        // Tap value readout to reset individual slider to default
        tvVal.setOnClickListener {
            sb.progress = toProg(defaultVal)
            tvVal.text = fmt(defaultVal)
            tvVal.setTextColor(ctx.getColor(R.color.lr_text_muted))
            onChanged(defaultVal, true)
        }

        sb.setOnSeekBarChangeListener(object : SeekBar.OnSeekBarChangeListener {
            override fun onProgressChanged(seekBar: SeekBar?, progress: Int, fromUser: Boolean) {
                if (!fromUser) return
                val v = min + (progress.toFloat() / steps) * (max - min)
                tvVal.text = fmt(v)
                val isMod = kotlin.math.abs(v - defaultVal) > 0.01f
                tvVal.setTextColor(ctx.getColor(if (isMod) R.color.lr_accent else R.color.lr_text_muted))
                onChanged(v, false)
            }

            override fun onStartTrackingTouch(seekBar: SeekBar?) = Unit

            override fun onStopTrackingTouch(seekBar: SeekBar?) {
                val p = seekBar?.progress ?: return
                val v = min + (p.toFloat() / steps) * (max - min)
                onChanged(v, true)
            }
        })

        row.addView(sb)
        parent.addView(row)
        return sb
    }

    fun addChipBar(
        parent: LinearLayout,
        options: List<Pair<String, Boolean>>,
        onSelect: (Int) -> Unit
    ) {
        val ctx = parent.context
        val scroll = android.widget.HorizontalScrollView(ctx).apply {
            isHorizontalScrollBarEnabled = false
            setPadding(0, dp(ctx, 4), 0, dp(ctx, 6))
        }
        val strip = LinearLayout(ctx).apply { orientation = LinearLayout.HORIZONTAL }
        options.forEachIndexed { idx, (label, selected) ->
            val btn = Button(ctx).apply {
                text = label
                textSize = 11.5f
                isAllCaps = false
                minHeight = dp(ctx, 40)
                setPadding(dp(ctx, 12), 0, dp(ctx, 12), 0)
                setTextColor(ctx.getColor(if (selected) R.color.lr_on_accent else R.color.lr_text))
                setBackgroundResource(if (selected) R.drawable.bg_pill_accent else R.drawable.bg_pill_neutral)
                val lp = LinearLayout.LayoutParams(
                    LinearLayout.LayoutParams.WRAP_CONTENT,
                    dp(ctx, 36)
                ).apply { marginEnd = dp(ctx, 6) }
                layoutParams = lp
                setOnClickListener { onSelect(idx) }
            }
            strip.addView(btn)
        }
        scroll.addView(strip)
        parent.addView(scroll)
    }

    fun showCopySettingsDialog(
        context: Context,
        initialMask: CopySettingsMask,
        onConfirm: (CopySettingsMask) -> Unit
    ) {
        val mask = initialMask.copy()
        val scroll = ScrollView(context)
        val col = LinearLayout(context).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dp(context, 20), dp(context, 12), dp(context, 20), dp(context, 12))
        }

        val items = listOf(
            "Treatment & Camera Profile" to { v: Boolean -> mask.profileAndColorTreatment = v } to mask.profileAndColorTreatment,
            "White Balance (Temp & Tint)" to { v: Boolean -> mask.whiteBalance = v } to mask.whiteBalance,
            "Basic Tone (Exposure/Contrast/Hi/Shd)" to { v: Boolean -> mask.basicTone = v } to mask.basicTone,
            "Presence (Texture/Clarity/Dehaze/Vib/Sat)" to { v: Boolean -> mask.presence = v } to mask.presence,
            "Tone Curve (Parametric & Point RGB)" to { v: Boolean -> mask.toneCurve = v } to mask.toneCurve,
            "Color Mixer (8-Band HSL & B&W)" to { v: Boolean -> mask.colorMixer = v } to mask.colorMixer,
            "Color Grading (3-Way Wheels & Split)" to { v: Boolean -> mask.colorGrading = v } to mask.colorGrading,
            "Detail (Sharpening & Noise Reduction)" to { v: Boolean -> mask.detail = v } to mask.detail,
            "Effects (Vignette & Film Grain)" to { v: Boolean -> mask.effects = v } to mask.effects,
            "Optics & Lens Blur" to { v: Boolean -> mask.optics = v } to mask.optics,
            "Crop, Rotate & Perspective" to { v: Boolean -> mask.cropAndTransform = v } to mask.cropAndTransform,
            "Masking & Spot Heal" to { v: Boolean -> mask.masksAndHeal = v } to mask.masksAndHeal
        )

        val boxes = mutableListOf<CheckBox>()
        items.forEach { (pair, initVal) ->
            val cb = CheckBox(context).apply {
                text = pair.first
                isChecked = initVal
                setTextColor(context.getColor(R.color.lr_text))
                setOnCheckedChangeListener { _, checked -> pair.second(checked) }
            }
            boxes.add(cb)
            col.addView(cb)
        }
        scroll.addView(col)

        AlertDialog.Builder(context, android.R.style.Theme_DeviceDefault_Dialog_Alert)
            .setTitle("Copy Develop Settings")
            .setView(scroll)
            .setNeutralButton("Toggle All", null)
            .setNegativeButton("Cancel", null)
            .setPositiveButton("Copy Selected") { _, _ -> onConfirm(mask) }
            .create().apply {
                show()
                getButton(AlertDialog.BUTTON_NEUTRAL)?.setOnClickListener {
                    val targetState = boxes.any { !it.isChecked }
                    boxes.forEach { it.isChecked = targetState }
                }
            }
    }

    fun showExportAndWatermarkDialog(
        context: Context,
        previewThumb: Bitmap?,
        baseFileName: String,
        renderFullBitmap: suspend () -> Bitmap?,
        onPickWatermarkGraphic: (((String, String) -> Unit) -> Unit)? = null,
        onExportComplete: (Uri?) -> Unit
    ) {
        var fmt = ExportFormat.JPEG
        var resize = ExportResize.ORIGINAL
        var quality = 94
        var wm = SessionClipboard.lastWatermark
        var cachedGraphic: Bitmap? = if (wm.graphicUriString != null) {
            WatermarkRenderer.loadGraphicBitmap(context, wm.graphicUriString, 512)
        } else null

        val scroll = ScrollView(context)
        val root = LinearLayout(context).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dp(context, 18), dp(context, 12), dp(context, 18), dp(context, 12))
            setBackgroundColor(context.getColor(R.color.lr_surface))
        }

        // Live Watermark Thumbnail Preview (supports both Text & Graphic overlays)
        val ivPreview = ImageView(context).apply {
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                dp(context, 108)
            )
            scaleType = ImageView.ScaleType.FIT_CENTER
            setBackgroundColor(context.getColor(R.color.lr_bg))
        }
        fun refreshPreview() {
            if (previewThumb != null) {
                ivPreview.setImageBitmap(WatermarkRenderer.applyWatermark(previewThumb, wm, cachedGraphic))
            }
        }
        root.addView(ivPreview)
        refreshPreview()

        addSectionHeader(root, "EXPORT FORMAT & RESOLUTION")
        val fmtRow = LinearLayout(context).apply { orientation = LinearLayout.VERTICAL }
        fun rebuildFormatChips() {
            fmtRow.removeAllViews()
            addChipBar(fmtRow, ExportFormat.entries.map { it.label to (it == fmt) }) { idx ->
                fmt = ExportFormat.entries[idx]
                rebuildFormatChips()
            }
            addChipBar(fmtRow, ExportResize.entries.map { it.label to (it == resize) }) { idx ->
                resize = ExportResize.entries[idx]
                rebuildFormatChips()
            }
        }
        rebuildFormatChips()
        root.addView(fmtRow)

        addSliderRow(root, "Quality", 20f, 100f, 94f, quality.toFloat(), 0, "%") { v, _ ->
            quality = v.roundToInt()
        }

        addSectionHeader(root, "WATERMARK OVERLAY (TEXT / GRAPHIC)")
        val swWm = Switch(context).apply {
            text = "Include Copyright / Studio Watermark"
            isChecked = wm.enabled
            setTextColor(context.getColor(R.color.lr_text))
        }
        root.addView(swWm)

        val styleBarHost = LinearLayout(context).apply { orientation = LinearLayout.VERTICAL }
        root.addView(styleBarHost)

        val etText = EditText(context).apply {
            setText(wm.text)
            hint = "© Photographer Name"
            setTextColor(context.getColor(R.color.lr_text))
            setHintTextColor(context.getColor(R.color.lr_text_muted))
            textSize = 13f
        }
        root.addView(etText)

        // Graphic Watermark Controls (PNG/JPG/WebP logo + built-in LR mark)
        val graphicBox = LinearLayout(context).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(0, dp(context, 4), 0, dp(context, 4))
        }
        val tvGraphicName = TextView(context).apply {
            textSize = 12f
            setTextColor(context.getColor(R.color.lr_text_muted))
            setPadding(0, 0, 0, dp(context, 4))
        }
        graphicBox.addView(tvGraphicName)

        val graphicBtnRow = LinearLayout(context).apply { orientation = LinearLayout.HORIZONTAL }
        fun wmPill(label: String, textColorRes: Int = R.color.lr_text, weight: Float = 1f, marginEndDp: Int = 6, onClick: () -> Unit): Button =
            Button(context).apply {
                text = label
                textSize = 11.5f
                isAllCaps = false
                maxLines = 1
                minHeight = 0
                minimumHeight = 0
                minWidth = 0
                minimumWidth = 0
                setPadding(dp(context, 8), 0, dp(context, 8), 0)
                setTextColor(context.getColor(textColorRes))
                setBackgroundResource(R.drawable.bg_pill_neutral)
                layoutParams = LinearLayout.LayoutParams(0, dp(context, 36), weight).apply {
                    marginEnd = dp(context, marginEndDp)
                }
                setOnClickListener { onClick() }
            }

        fun updateGraphicLabel() {
            tvGraphicName.text = if (wm.graphicDisplayName != null) {
                "Graphic: ${wm.graphicDisplayName}"
            } else {
                "Choose a transparent PNG, WebP, or JPG logo:"
            }
        }
        fun applyPickedGraphic(uriStr: String, name: String) {
            if (cachedGraphic != null && cachedGraphic?.isRecycled == false) {
                cachedGraphic?.recycle()
            }
            cachedGraphic = WatermarkRenderer.loadGraphicBitmap(context, uriStr, 512)
            val targetSize = if (wm.sizePercent < 14f) 24f else wm.sizePercent
            wm = wm.copy(
                enabled = true,
                style = WatermarkStyle.GRAPHIC,
                graphicUriString = uriStr,
                graphicDisplayName = name,
                sizePercent = targetSize
            )
            swWm.isChecked = true
            updateGraphicLabel()
            refreshPreview()
        }

        if (onPickWatermarkGraphic != null) {
            graphicBtnRow.addView(wmPill("Choose PNG / JPG", R.color.lr_accent, 1.3f) {
                onPickWatermarkGraphic { uriStr, name -> applyPickedGraphic(uriStr, name) }
            })
        }
        graphicBtnRow.addView(wmPill("LR Mark", R.color.lr_text, 0.9f) {
            applyPickedGraphic(WatermarkRenderer.BUILTIN_LR_MARK_URI, "LightRaw Studio Mark.png")
        })
        graphicBtnRow.addView(wmPill("Clear", R.color.lr_danger, 0.7f, marginEndDp = 0) {
            if (cachedGraphic != null && cachedGraphic?.isRecycled == false) cachedGraphic?.recycle()
            cachedGraphic = null
            wm = wm.copy(graphicUriString = null, graphicDisplayName = null)
            updateGraphicLabel()
            refreshPreview()
        })
        graphicBox.addView(graphicBtnRow)
        root.addView(graphicBox)

        fun syncStyleVisibility() {
            val isGraphic = wm.style == WatermarkStyle.GRAPHIC
            etText.visibility = if (isGraphic) View.GONE else View.VISIBLE
            graphicBox.visibility = if (isGraphic) View.VISIBLE else View.GONE
            updateGraphicLabel()
        }
        fun rebuildStyleBar() {
            styleBarHost.removeAllViews()
            addChipBar(styleBarHost, WatermarkStyle.entries.map { it.label to (it == wm.style) }) { idx ->
                val chosen = WatermarkStyle.entries[idx]
                val adjSize = if (chosen == WatermarkStyle.GRAPHIC && wm.sizePercent < 14f) 24f else wm.sizePercent
                wm = wm.copy(style = chosen, sizePercent = adjSize)
                rebuildStyleBar()
                syncStyleVisibility()
                refreshPreview()
            }
        }
        rebuildStyleBar()
        syncStyleVisibility()

        swWm.setOnCheckedChangeListener { _, checked ->
            wm = wm.copy(enabled = checked, text = etText.text.toString())
            refreshPreview()
        }
        etText.addTextChangedListener(object : android.text.TextWatcher {
            override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) = Unit
            override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) = Unit
            override fun afterTextChanged(s: android.text.Editable?) {
                wm = wm.copy(text = s?.toString() ?: "")
                refreshPreview()
            }
        })

        // Size Mode (Proportional / Fit / Fill) + Rotation
        val sizeModeHost = LinearLayout(context).apply { orientation = LinearLayout.VERTICAL }
        fun rebuildSizeModeBar() {
            sizeModeHost.removeAllViews()
            val opts = WatermarkSizeMode.entries.map { it.label to (it == wm.sizeMode) } +
                listOf("Rotate ${wm.rotationDegrees}°" to (wm.rotationDegrees != 0))
            addChipBar(sizeModeHost, opts) { idx ->
                if (idx < WatermarkSizeMode.entries.size) {
                    wm = wm.copy(sizeMode = WatermarkSizeMode.entries[idx])
                } else {
                    wm = wm.copy(rotationDegrees = (wm.rotationDegrees + 90) % 360)
                }
                rebuildSizeModeBar()
                refreshPreview()
            }
        }
        rebuildSizeModeBar()
        root.addView(sizeModeHost)

        // 3x3 Anchor Selector Grid
        val anchorGrid = GridLayout(context).apply {
            rowCount = 3
            columnCount = 3
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.WRAP_CONTENT
            )
            setPadding(0, dp(context, 4), 0, dp(context, 4))
        }
        val anchorButtons = mutableListOf<Button>()
        WatermarkAnchor.entries.forEachIndexed { idx, anchor ->
            val btn = Button(context).apply {
                text = anchor.label
                textSize = 11f
                minimumHeight = 0
                minHeight = 0
                setPadding(0, dp(context, 6), 0, dp(context, 6))
                layoutParams = GridLayout.LayoutParams(
                    GridLayout.spec(idx / 3, 1),
                    GridLayout.spec(idx % 3, 1, 1f)
                ).apply {
                    width = 0
                    height = dp(context, 36)
                    setMargins(dp(context, 2), dp(context, 2), dp(context, 2), dp(context, 2))
                }
                setOnClickListener {
                    wm = wm.copy(anchor = anchor)
                    anchorButtons.forEachIndexed { bIdx, b ->
                        val sel = WatermarkAnchor.entries[bIdx] == wm.anchor
                        b.setTextColor(context.getColor(if (sel) R.color.lr_on_accent else R.color.lr_text))
                        b.setBackgroundColor(context.getColor(if (sel) R.color.lr_accent else R.color.lr_surface_elevated))
                    }
                    refreshPreview()
                }
            }
            val sel = anchor == wm.anchor
            btn.setTextColor(context.getColor(if (sel) R.color.lr_on_accent else R.color.lr_text))
            btn.setBackgroundColor(context.getColor(if (sel) R.color.lr_accent else R.color.lr_surface_elevated))
            anchorButtons.add(btn)
            anchorGrid.addView(btn)
        }
        root.addView(anchorGrid)

        addSliderRow(root, "Watermark Size", 3f, 60f, 8f, wm.sizePercent.coerceIn(3f, 60f), 0, "%") { v, _ ->
            wm = wm.copy(sizePercent = v)
            refreshPreview()
        }
        addSliderRow(root, "Watermark Opacity", 15f, 100f, 82f, wm.opacityPercent, 0, "%") { v, _ ->
            wm = wm.copy(opacityPercent = v)
            refreshPreview()
        }

        scroll.addView(root)

        AlertDialog.Builder(context, android.R.style.Theme_DeviceDefault_Dialog_Alert)
            .setTitle("Export & Watermark")
            .setView(scroll)
            .setNegativeButton("Cancel") { _, _ ->
                if (cachedGraphic != null && cachedGraphic?.isRecycled == false) cachedGraphic?.recycle()
            }
            .setNeutralButton("Share") { _, _ ->
                if (cachedGraphic != null && cachedGraphic?.isRecycled == false) cachedGraphic?.recycle()
                runExport(context, true, fmt, quality, resize, wm, baseFileName, renderFullBitmap, onExportComplete)
            }
            .setPositiveButton("Save") { _, _ ->
                if (cachedGraphic != null && cachedGraphic?.isRecycled == false) cachedGraphic?.recycle()
                runExport(context, false, fmt, quality, resize, wm, baseFileName, renderFullBitmap, onExportComplete)
            }
            .show()
    }

    private fun runExport(
        context: Context,
        shareAfter: Boolean,
        fmt: ExportFormat,
        quality: Int,
        resize: ExportResize,
        wm: WatermarkSettings,
        baseFileName: String,
        renderFullBitmap: suspend () -> Bitmap?,
        onDone: (Uri?) -> Unit
    ) {
        SessionClipboard.lastWatermark = wm
        Toast.makeText(context, "Rendering full-resolution export…", Toast.LENGTH_SHORT).show()
        CoroutineScope(Dispatchers.Main).launch {
            val savedUri = withContext(Dispatchers.Default) {
                var bmp = renderFullBitmap() ?: return@withContext null
                val maxPx = resize.maxEdgePx
                if (maxPx != null && maxOf(bmp.width, bmp.height) > maxPx) {
                    val sc = maxPx.toFloat() / maxOf(bmp.width, bmp.height)
                    val nw = (bmp.width * sc).roundToInt().coerceAtLeast(16)
                    val nh = (bmp.height * sc).roundToInt().coerceAtLeast(16)
                    val scaled = Bitmap.createScaledBitmap(bmp, nw, nh, true)
                    bmp.recycle()
                    bmp = scaled
                }
                val stamped = WatermarkRenderer.applyWatermarkWithContext(context, bmp, wm)
                val uri = MediaStoreImporter.exportToGallery(
                    context,
                    stamped,
                    baseFileName,
                    ExportConfig(fmt, quality, resize, wm)
                )
                if (stamped != bmp) stamped.recycle()
                bmp.recycle()
                uri
            }
            if (savedUri != null) {
                Toast.makeText(context, "Saved to Pictures/LightRaw", Toast.LENGTH_LONG).show()
                if (shareAfter) {
                    val shareIntent = Intent(Intent.ACTION_SEND).apply {
                        type = fmt.mime
                        putExtra(Intent.EXTRA_STREAM, savedUri)
                        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                    }
                    context.startActivity(Intent.createChooser(shareIntent, "Share LightRaw Export"))
                }
            } else {
                Toast.makeText(context, "Export failed", Toast.LENGTH_SHORT).show()
            }
            onDone(savedUri)
        }
    }
}
