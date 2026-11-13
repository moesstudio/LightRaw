package com.moestret.lightraw.ui

import android.app.AlertDialog
import android.content.Context
import android.graphics.Bitmap
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.text.TextUtils
import android.view.Gravity
import android.view.View
import android.widget.Button
import android.widget.EditText
import android.widget.ImageView
import android.widget.LinearLayout
import android.widget.Switch
import android.widget.TextView
import com.moestret.lightraw.R
import com.moestret.lightraw.engine.KotlinPv2012Engine
import com.moestret.lightraw.model.AspectRatioPreset
import com.moestret.lightraw.model.CameraProfile
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.DevelopPreset
import com.moestret.lightraw.model.MaskGroup
import com.moestret.lightraw.model.MaskPrimitiveKind
import com.moestret.lightraw.model.WhiteBalanceMode
import com.moestret.lightraw.ui.views.ColorWheelView
import com.moestret.lightraw.ui.views.ToneCurveView
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.util.UUID

/**
 * Modular builder for all 13 interactive Develop tool panels inside `LoupeEditorActivity`.
 */
object LoupePanels {

    enum class ToolTab(val label: String) {
        AI_MATCH("AI Match"),
        PRESETS("Presets"),
        LIGHT("Light"),
        COLOR("Color"),
        MIXER("Mixer"),
        GRADING("Grading"),
        CURVE("Curve"),
        EFFECTS("Effects"),
        DETAIL("Detail"),
        OPTICS("Optics"),
        CROP("Crop"),
        MASK("Mask"),
        HEAL("Heal")
    }

    data class PanelHost(
        val context: Context,
        val scope: CoroutineScope,
        val container: LinearLayout,
        val thumbBitmap: Bitmap?,
        val getParams: () -> DevelopParams,
        val updateParams: (DevelopParams, Boolean, String?) -> Unit,
        val triggerAutoTone: () -> Unit,
        val triggerMatchRef: () -> Unit,
        val triggerImportXmp: () -> Unit,
        val triggerExportXmp: (DevelopPreset) -> Unit,
        val getPresets: () -> List<DevelopPreset>,
        val saveNewPreset: (String, String, DevelopParams) -> Unit,
        val renamePreset: (String, String) -> Unit,
        val toggleFavoritePreset: (String, Boolean) -> Unit,
        val updateUserPreset: (DevelopPreset) -> Unit,
        val deletePreset: (String) -> Unit,
        val setActiveMaskId: (String?) -> Unit,
        val getActiveMaskId: () -> String?,
        val rebuildPanel: () -> Unit
    )

    private var mixerBand = 0
    private var gradingZone = 0 // 0=Shadows,1=Mid,2=High,3=Global
    private var curveChannel = 0 // 0=Luma,1=R,2=G,3=B
    private var detailSubTab = 0 // 0=Sharpening,1=Noise Reduction
    private var cropSubTab = 0 // 0=Crop,1=Geometry
    private var presetCategory = "All"
    private var activePresetBase: DevelopParams? = null
    private var activePresetTarget: DevelopParams? = null

    fun renderTab(tab: ToolTab, host: PanelHost) {
        host.container.removeAllViews()
        when (tab) {
            ToolTab.AI_MATCH -> buildAiPanel(host)
            ToolTab.PRESETS -> buildPresetsPanel(host)
            ToolTab.LIGHT -> buildLightPanel(host)
            ToolTab.COLOR -> buildColorPanel(host)
            ToolTab.MIXER -> buildMixerPanel(host)
            ToolTab.GRADING -> buildGradingPanel(host)
            ToolTab.CURVE -> buildCurvePanel(host)
            ToolTab.EFFECTS -> buildEffectsPanel(host)
            ToolTab.DETAIL -> buildDetailPanel(host)
            ToolTab.OPTICS -> buildOpticsPanel(host)
            ToolTab.CROP -> buildCropPanel(host)
            ToolTab.MASK -> buildMaskPanel(host)
            ToolTab.HEAL -> buildHealPanel(host)
        }
    }

    /** Creates a compact single-line darkroom pill button with explicit token colors (no system fallback tinting). */
    private fun pillBtn(
        ctx: Context,
        title: String,
        accent: Boolean = false,
        textColorRes: Int = if (accent) R.color.lr_on_accent else R.color.lr_text,
        heightDp: Int = 40,
        textSp: Float = 12f,
        weight: Float? = null,
        marginStartDp: Int = 0,
        marginEndDp: Int = 0,
        topMarginDp: Int = 0,
        enabled: Boolean = true,
        onClick: () -> Unit
    ): Button = Button(ctx).apply {
        text = title
        textSize = textSp
        isAllCaps = false
        maxLines = 1
        ellipsize = TextUtils.TruncateAt.END
        isEnabled = enabled
        alpha = if (enabled) 1f else 0.45f
        gravity = Gravity.CENTER
        minWidth = 0
        minimumWidth = 0
        minHeight = 0
        minimumHeight = 0
        setPadding(DarkroomUiFactory.dp(ctx, 10), 0, DarkroomUiFactory.dp(ctx, 10), 0)
        setBackgroundResource(if (accent) R.drawable.bg_pill_accent else R.drawable.bg_pill_neutral)
        setTextColor(ctx.getColor(textColorRes))
        val w = if (weight != null) 0 else LinearLayout.LayoutParams.WRAP_CONTENT
        layoutParams = LinearLayout.LayoutParams(w, DarkroomUiFactory.dp(ctx, heightDp), weight ?: 0f).apply {
            marginStart = DarkroomUiFactory.dp(ctx, marginStartDp)
            marginEnd = DarkroomUiFactory.dp(ctx, marginEndDp)
            topMargin = DarkroomUiFactory.dp(ctx, topMarginDp)
        }
        setOnClickListener { onClick() }
    }

    private fun buildAiPanel(host: PanelHost) {
        val ctx = host.context
        DarkroomUiFactory.addSectionHeader(host.container, "INTELLIGENT AUTO & COLOR TRANSFER")
        val row = LinearLayout(ctx).apply { orientation = LinearLayout.HORIZONTAL }
        val btnAuto = pillBtn(
            ctx = ctx,
            title = "⚡ AI Auto Tone",
            accent = true,
            heightDp = 44,
            textSp = 12.5f,
            weight = 1f
        ) { host.triggerAutoTone() }
        val btnMatch = pillBtn(
            ctx = ctx,
            title = "🎨 Match to Ref",
            textColorRes = R.color.lr_success,
            heightDp = 44,
            textSp = 12.5f,
            weight = 1f,
            marginStartDp = 8
        ) { host.triggerMatchRef() }
        row.addView(btnAuto)
        row.addView(btnMatch)
        host.container.addView(row)

        DarkroomUiFactory.addSectionHeader(host.container, "CAMERA PROFILE & TREATMENT")
        val cur = host.getParams()
        DarkroomUiFactory.addChipBar(
            host.container,
            CameraProfile.entries.map { it.label to (it == cur.cameraProfile) }
        ) { idx ->
            val p = host.getParams()
            val prof = CameraProfile.entries[idx]
            host.updateParams(
                p.copy(cameraProfile = prof, convertToGrayscale = prof == CameraProfile.MONOCHROME),
                true,
                "Profile: ${prof.label}"
            )
            host.rebuildPanel()
        }
        DarkroomUiFactory.addSliderRow(host.container, "Profile Look Amount", 0f, 200f, 100f, cur.lookAmount, 0, "%") { v, c ->
            host.updateParams(host.getParams().copy(lookAmount = v), c, if (c) "Profile Amount" else null)
        }
    }

    private fun buildPresetsPanel(host: PanelHost) {
        val ctx = host.context
        val presets = host.getPresets()
        val categories = listOf("All", "Favorites ★") + presets.map { it.category }.distinct()

        // Action Buttons row
        val topBtns = LinearLayout(ctx).apply { orientation = LinearLayout.HORIZONTAL }
        fun smallAction(title: String, marginEndDp: Int = 6, onClick: () -> Unit) =
            pillBtn(ctx, title, heightDp = 38, textSp = 11.5f, weight = 1f, marginEndDp = marginEndDp, onClick = onClick)
        topBtns.addView(smallAction("+ Save Preset") {
            val et = EditText(ctx).apply { hint = "Preset Name" }
            AlertDialog.Builder(ctx)
                .setTitle("Create User Preset")
                .setView(et)
                .setNegativeButton("Cancel", null)
                .setPositiveButton("Save") { _, _ ->
                    val name = et.text.toString().trim().ifEmpty { "My Look" }
                    host.saveNewPreset(name, "User Presets", host.getParams().deepCopy())
                    host.rebuildPanel()
                }
                .show()
        })
        topBtns.addView(smallAction("Import .xmp") { host.triggerImportXmp() })
        topBtns.addView(smallAction("Export .xmp", marginEndDp = 0) {
            val p = host.getParams()
            val tmp = DevelopPreset(
                id = UUID.randomUUID().toString(),
                category = "User Presets",
                name = p.appliedPresetName.ifBlank { "LightRaw_Look" },
                isUser = true,
                isFavorite = false,
                params = p
            )
            host.triggerExportXmp(tmp)
        })
        host.container.addView(topBtns)

        DarkroomUiFactory.addChipBar(host.container, categories.map { it to (it == presetCategory) }) { idx ->
            presetCategory = categories[idx]
            host.rebuildPanel()
        }

        val hint = TextView(ctx).apply {
            text = "Tip: Long-press any preset for ★ Favorite, Rename, Update, Export .xmp, or Delete"
            setTextColor(ctx.getColor(R.color.lr_text_muted))
            textSize = 10.5f
            setPadding(0, 0, 0, DarkroomUiFactory.dp(ctx, 4))
        }
        host.container.addView(hint)

        // Filmstrip of presets with live rendered thumbnails
        val filtered = when (presetCategory) {
            "All" -> presets
            "Favorites ★" -> presets.filter { it.isFavorite }
            else -> presets.filter { it.category == presetCategory }
        }
        val stripScroll = android.widget.HorizontalScrollView(ctx).apply { isHorizontalScrollBarEnabled = false }
        val strip = LinearLayout(ctx).apply { orientation = LinearLayout.HORIZONTAL }
        val curName = host.getParams().appliedPresetName

        for (preset in filtered) {
            val card = LinearLayout(ctx).apply {
                orientation = LinearLayout.VERTICAL
                gravity = Gravity.CENTER_HORIZONTAL
                setPadding(DarkroomUiFactory.dp(ctx, 4), DarkroomUiFactory.dp(ctx, 4), DarkroomUiFactory.dp(ctx, 4), DarkroomUiFactory.dp(ctx, 4))
                setBackgroundColor(ctx.getColor(if (preset.name == curName) R.color.lr_accent else R.color.lr_surface_elevated))
                layoutParams = LinearLayout.LayoutParams(DarkroomUiFactory.dp(ctx, 88), LinearLayout.LayoutParams.WRAP_CONTENT).apply {
                    marginEnd = DarkroomUiFactory.dp(ctx, 8)
                }
            }
            val iv = ImageView(ctx).apply {
                layoutParams = LinearLayout.LayoutParams(DarkroomUiFactory.dp(ctx, 78), DarkroomUiFactory.dp(ctx, 58))
                scaleType = ImageView.ScaleType.CENTER_CROP
            }
            host.thumbBitmap?.let { tb ->
                host.scope.launch {
                    val preview = withContext(Dispatchers.Default) {
                        KotlinPv2012Engine.render(tb, preset.params, draftInteractive = true, applyCrop = false).bitmap
                    }
                    iv.setImageBitmap(preview)
                }
            }
            val tv = TextView(ctx).apply {
                text = if (preset.isFavorite) "★ ${preset.name}" else preset.name
                setTextColor(ctx.getColor(R.color.lr_text))
                textSize = 10.5f
                maxLines = 1
                setPadding(0, DarkroomUiFactory.dp(ctx, 4), 0, 0)
            }
            card.addView(iv)
            card.addView(tv)
            card.setOnClickListener {
                activePresetBase = host.getParams().deepCopy()
                activePresetTarget = preset.params.copy(appliedPresetName = preset.name, presetAmount = 100f)
                host.updateParams(activePresetTarget!!, true, "Preset: ${preset.name}")
                host.rebuildPanel()
            }
            card.setOnLongClickListener {
                val actions = mutableListOf(
                    if (preset.isFavorite) "Remove from Favorites" else "Add to Favorites ★",
                    "Export as .xmp"
                )
                if (preset.isUser) {
                    actions.add("Rename Preset…")
                    actions.add("Update Preset with Current Edits")
                    actions.add("Delete User Preset")
                }
                AlertDialog.Builder(ctx)
                    .setTitle(preset.name)
                    .setItems(actions.toTypedArray()) { _, which ->
                        when (which) {
                            0 -> {
                                host.toggleFavoritePreset(preset.id, !preset.isFavorite)
                                host.rebuildPanel()
                            }
                            1 -> host.triggerExportXmp(preset)
                            2 -> if (preset.isUser) {
                                val et = EditText(ctx).apply { setText(preset.name) }
                                AlertDialog.Builder(ctx)
                                    .setTitle("Rename Preset")
                                    .setView(et)
                                    .setNegativeButton("Cancel", null)
                                    .setPositiveButton("Rename") { _, _ ->
                                        val nextName = et.text.toString().trim()
                                        if (nextName.isNotEmpty()) {
                                            host.renamePreset(preset.id, nextName)
                                            host.rebuildPanel()
                                        }
                                    }
                                    .show()
                            }
                            3 -> if (preset.isUser) {
                                host.updateUserPreset(preset.copy(params = host.getParams().deepCopy()))
                                host.rebuildPanel()
                            }
                            4 -> if (preset.isUser) {
                                AlertDialog.Builder(ctx)
                                    .setTitle("Delete '${preset.name}'?")
                                    .setNegativeButton("Cancel", null)
                                    .setPositiveButton("Delete") { _, _ ->
                                        host.deletePreset(preset.id)
                                        host.rebuildPanel()
                                    }
                                    .show()
                            }
                        }
                    }
                    .show()
                true
            }
            strip.addView(card)
        }
        stripScroll.addView(strip)
        host.container.addView(stripScroll)

        // Preset Intensity Amount Slider (0..200%)
        val base = activePresetBase
        val tgt = activePresetTarget
        if (base != null && tgt != null) {
            DarkroomUiFactory.addSliderRow(
                host.container,
                "Preset Intensity (${tgt.appliedPresetName})",
                0f,
                200f,
                100f,
                host.getParams().presetAmount,
                0,
                "%"
            ) { v, c ->
                val blended = DevelopParams.interpolate(base, tgt, v)
                host.updateParams(blended, c, if (c) "Preset Intensity" else null)
            }
        }
    }

    private fun buildLightPanel(host: PanelHost) {
        val p = host.getParams()
        DarkroomUiFactory.addSliderRow(host.container, "Exposure", -5f, 5f, 0f, p.exposure, 2, " EV") { v, c ->
            host.updateParams(host.getParams().copy(exposure = v), c, if (c) "Exposure" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Contrast", -100f, 100f, 0f, p.contrast) { v, c ->
            host.updateParams(host.getParams().copy(contrast = v), c, if (c) "Contrast" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Highlights", -100f, 100f, 0f, p.highlights) { v, c ->
            host.updateParams(host.getParams().copy(highlights = v), c, if (c) "Highlights" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Shadows", -100f, 100f, 0f, p.shadows) { v, c ->
            host.updateParams(host.getParams().copy(shadows = v), c, if (c) "Shadows" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Whites", -100f, 100f, 0f, p.whites) { v, c ->
            host.updateParams(host.getParams().copy(whites = v), c, if (c) "Whites" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Blacks", -100f, 100f, 0f, p.blacks) { v, c ->
            host.updateParams(host.getParams().copy(blacks = v), c, if (c) "Blacks" else null)
        }
    }

    private fun buildColorPanel(host: PanelHost) {
        val ctx = host.context
        val p = host.getParams()

        val swBw = Switch(ctx).apply {
            text = "Black & White Treatment"
            isChecked = p.convertToGrayscale
            setTextColor(ctx.getColor(R.color.lr_text))
            setOnCheckedChangeListener { _, checked ->
                host.updateParams(host.getParams().copy(convertToGrayscale = checked), true, "B&W Toggle")
            }
        }
        host.container.addView(swBw)

        DarkroomUiFactory.addChipBar(
            host.container,
            WhiteBalanceMode.entries.map { it.label to (it == p.whiteBalance) }
        ) { idx ->
            val mode = WhiteBalanceMode.entries[idx]
            val cur = host.getParams()
            val tt = mode.defaultTempTint()
            val updated = if (tt != null) {
                cur.copy(whiteBalance = mode, temperature = tt.first, tint = tt.second)
            } else {
                cur.copy(whiteBalance = mode)
            }
            host.updateParams(updated, true, "WB: ${mode.label}")
            host.rebuildPanel()
        }

        DarkroomUiFactory.addSliderRow(host.container, "Temperature", 2000f, 15000f, 5500f, p.temperature, 0, " K") { v, c ->
            host.updateParams(host.getParams().copy(whiteBalance = WhiteBalanceMode.CUSTOM, temperature = v), c, if (c) "Temp" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Tint", -150f, 150f, 0f, p.tint) { v, c ->
            host.updateParams(host.getParams().copy(whiteBalance = WhiteBalanceMode.CUSTOM, tint = v), c, if (c) "Tint" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Vibrance", -100f, 100f, 0f, p.vibrance) { v, c ->
            host.updateParams(host.getParams().copy(vibrance = v), c, if (c) "Vibrance" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Saturation", -100f, 100f, 0f, p.saturation) { v, c ->
            host.updateParams(host.getParams().copy(saturation = v), c, if (c) "Saturation" else null)
        }
    }

    private fun buildMixerPanel(host: PanelHost) {
        val ctx = host.context
        val p = host.getParams()
        val bands = listOf("Red", "Orange", "Yellow", "Green", "Aqua", "Blue", "Purple", "Magenta")
        val swatchColors = intArrayOf(
            0xFFEF4444.toInt(), 0xFFF97316.toInt(), 0xFFEAB308.toInt(), 0xFF22C55E.toInt(),
            0xFF06B6D4.toInt(), 0xFF3B82F6.toInt(), 0xFFA855F7.toInt(), 0xFFEC4899.toInt()
        )

        val topRow = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(0, DarkroomUiFactory.dp(ctx, 2), 0, DarkroomUiFactory.dp(ctx, 8))
        }
        for (i in bands.indices) {
            val selected = i == mixerBand
            val dotSize = DarkroomUiFactory.dp(ctx, if (selected) 30 else 24)
            val dot = View(ctx).apply {
                layoutParams = LinearLayout.LayoutParams(dotSize, dotSize).apply {
                    marginEnd = DarkroomUiFactory.dp(ctx, 7)
                }
                background = GradientDrawable().apply {
                    shape = GradientDrawable.OVAL
                    setColor(swatchColors[i])
                    if (selected) setStroke(5, 0xFFFFFFFF.toInt())
                }
                contentDescription = bands[i]
                setOnClickListener {
                    mixerBand = i
                    host.rebuildPanel()
                }
            }
            topRow.addView(dot)
        }
        val spacer = View(ctx).apply { layoutParams = LinearLayout.LayoutParams(0, 1, 1f) }
        topRow.addView(spacer)
        val btnBw = pillBtn(
            ctx = ctx,
            title = "B&W",
            accent = p.convertToGrayscale,
            heightDp = 36,
            textSp = 11.5f
        ) {
            val cur = host.getParams()
            host.updateParams(cur.copy(convertToGrayscale = !cur.convertToGrayscale), true, "Toggle B&W")
            host.rebuildPanel()
        }
        topRow.addView(btnBw)
        host.container.addView(topRow)

        val b = mixerBand
        if (p.convertToGrayscale) {
            DarkroomUiFactory.addSliderRow(host.container, "${bands[b]} B&W Mix", -100f, 100f, 0f, p.grayMixer[b]) { v, c ->
                val cur = host.getParams()
                val arr = cur.grayMixer.copyOf().apply { this[b] = v }
                host.updateParams(cur.copy(grayMixer = arr), c, if (c) "B&W ${bands[b]}" else null)
            }
        } else {
            DarkroomUiFactory.addSliderRow(host.container, "${bands[b]} Hue", -100f, 100f, 0f, p.hueAdjustment[b]) { v, c ->
                val cur = host.getParams()
                val arr = cur.hueAdjustment.copyOf().apply { this[b] = v }
                host.updateParams(cur.copy(hueAdjustment = arr), c, if (c) "${bands[b]} Hue" else null)
            }
            DarkroomUiFactory.addSliderRow(host.container, "${bands[b]} Saturation", -100f, 100f, 0f, p.saturationAdjustment[b]) { v, c ->
                val cur = host.getParams()
                val arr = cur.saturationAdjustment.copyOf().apply { this[b] = v }
                host.updateParams(cur.copy(saturationAdjustment = arr), c, if (c) "${bands[b]} Sat" else null)
            }
            DarkroomUiFactory.addSliderRow(host.container, "${bands[b]} Luminance", -100f, 100f, 0f, p.luminanceAdjustment[b]) { v, c ->
                val cur = host.getParams()
                val arr = cur.luminanceAdjustment.copyOf().apply { this[b] = v }
                host.updateParams(cur.copy(luminanceAdjustment = arr), c, if (c) "${bands[b]} Lum" else null)
            }
        }
    }

    private fun buildGradingPanel(host: PanelHost) {
        val ctx = host.context
        val zones = listOf("Shadows", "Midtones", "Highlights", "Global")
        DarkroomUiFactory.addChipBar(host.container, zones.mapIndexed { i, z -> z to (i == gradingZone) }) { idx ->
            gradingZone = idx
            host.rebuildPanel()
        }
        val p = host.getParams()
        val (h, s, l) = when (gradingZone) {
            0 -> Triple(p.cgShadowHue, p.cgShadowSat, p.cgShadowLum)
            1 -> Triple(p.cgMidHue, p.cgMidSat, p.cgMidLum)
            2 -> Triple(p.cgHighHue, p.cgHighSat, p.cgHighLum)
            else -> Triple(p.cgGlobalHue, p.cgGlobalSat, p.cgGlobalLum)
        }

        val wheelRow = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(0, DarkroomUiFactory.dp(ctx, 4), 0, DarkroomUiFactory.dp(ctx, 6))
        }
        val readout = TextView(ctx).apply {
            text = "${zones[gradingZone]}\nH ${h.toInt()}° · S ${s.toInt()}%\nDouble-tap wheel to reset"
            setTextColor(ctx.getColor(R.color.lr_text_secondary))
            textSize = 11.5f
            layoutParams = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f)
        }
        val wheel = ColorWheelView(ctx).apply {
            hueDeg = h
            saturation = s
            layoutParams = LinearLayout.LayoutParams(
                DarkroomUiFactory.dp(ctx, 138),
                DarkroomUiFactory.dp(ctx, 138)
            ).apply { marginEnd = DarkroomUiFactory.dp(ctx, 12) }
            onColorChanged = { newH, newS, commit ->
                readout.text = "${zones[gradingZone]}\nH ${newH.toInt()}° · S ${newS.toInt()}%\nDouble-tap wheel to reset"
                val cur = host.getParams()
                val upd = when (gradingZone) {
                    0 -> cur.copy(cgShadowHue = newH, cgShadowSat = newS)
                    1 -> cur.copy(cgMidHue = newH, cgMidSat = newS)
                    2 -> cur.copy(cgHighHue = newH, cgHighSat = newS)
                    else -> cur.copy(cgGlobalHue = newH, cgGlobalSat = newS)
                }
                host.updateParams(upd, commit, if (commit) "${zones[gradingZone]} Wheel" else null)
            }
        }
        wheelRow.addView(wheel)
        wheelRow.addView(readout)
        host.container.addView(wheelRow)

        DarkroomUiFactory.addSliderRow(host.container, "${zones[gradingZone]} Luminance", -100f, 100f, 0f, l) { v, c ->
            val cur = host.getParams()
            val upd = when (gradingZone) {
                0 -> cur.copy(cgShadowLum = v)
                1 -> cur.copy(cgMidLum = v)
                2 -> cur.copy(cgHighLum = v)
                else -> cur.copy(cgGlobalLum = v)
            }
            host.updateParams(upd, c, if (c) "Color Grade Lum" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Blending", 0f, 100f, 50f, p.cgBlending) { v, c ->
            host.updateParams(host.getParams().copy(cgBlending = v), c, if (c) "Blending" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Balance", -100f, 100f, 0f, p.cgBalance) { v, c ->
            host.updateParams(host.getParams().copy(cgBalance = v), c, if (c) "Balance" else null)
        }
    }

    private fun buildCurvePanel(host: PanelHost) {
        val ctx = host.context
        val channels = listOf("RGB Luma", "Red", "Green", "Blue")
        DarkroomUiFactory.addChipBar(host.container, channels.mapIndexed { i, n -> n to (i == curveChannel) }) { idx ->
            curveChannel = idx
            host.rebuildPanel()
        }

        val p = host.getParams()
        val cv = ToneCurveView(ctx).apply {
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                DarkroomUiFactory.dp(ctx, 168)
            )
            strokeColor = when (curveChannel) {
                1 -> 0xFFEF4444.toInt()
                2 -> 0xFF34D399.toInt()
                3 -> 0xFF3B82F6.toInt()
                else -> 0xFFFFFFFF.toInt()
            }
            curvePoints = when (curveChannel) {
                1 -> p.toneCurveRed
                2 -> p.toneCurveGreen
                3 -> p.toneCurveBlue
                else -> p.toneCurveMaster
            }
            onCurveChanged = { pts, commit ->
                val cur = host.getParams()
                val upd = when (curveChannel) {
                    1 -> cur.copy(toneCurveRed = pts)
                    2 -> cur.copy(toneCurveGreen = pts)
                    3 -> cur.copy(toneCurveBlue = pts)
                    else -> cur.copy(toneCurveMaster = pts)
                }
                host.updateParams(upd, commit, if (commit) "${channels[curveChannel]} Curve" else null)
            }
        }
        host.container.addView(cv)

        val btnReset = pillBtn(
            ctx = ctx,
            title = "Reset ${channels[curveChannel]} Curve",
            heightDp = 36,
            textSp = 11.5f,
            topMarginDp = 6
        ) {
            val cur = host.getParams()
            val def = DevelopParams.defaultLinearCurve()
            val upd = when (curveChannel) {
                1 -> cur.copy(toneCurveRed = def)
                2 -> cur.copy(toneCurveGreen = def)
                3 -> cur.copy(toneCurveBlue = def)
                else -> cur.copy(toneCurveMaster = def, parametricHighlights = 0f, parametricLights = 0f, parametricDarks = 0f, parametricShadows = 0f)
            }
            host.updateParams(upd, true, "Reset Curve")
            host.rebuildPanel()
        }
        host.container.addView(btnReset)

        DarkroomUiFactory.addSliderRow(host.container, "Parametric Highlights", -100f, 100f, 0f, p.parametricHighlights) { v, c ->
            host.updateParams(host.getParams().copy(parametricHighlights = v), c, if (c) "Param Hi" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Parametric Lights", -100f, 100f, 0f, p.parametricLights) { v, c ->
            host.updateParams(host.getParams().copy(parametricLights = v), c, if (c) "Param Lights" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Parametric Darks", -100f, 100f, 0f, p.parametricDarks) { v, c ->
            host.updateParams(host.getParams().copy(parametricDarks = v), c, if (c) "Param Darks" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Parametric Shadows", -100f, 100f, 0f, p.parametricShadows) { v, c ->
            host.updateParams(host.getParams().copy(parametricShadows = v), c, if (c) "Param Shd" else null)
        }
    }

    private fun buildEffectsPanel(host: PanelHost) {
        val p = host.getParams()
        DarkroomUiFactory.addSliderRow(host.container, "Texture", -100f, 100f, 0f, p.texture) { v, c ->
            host.updateParams(host.getParams().copy(texture = v), c, if (c) "Texture" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Clarity", -100f, 100f, 0f, p.clarity) { v, c ->
            host.updateParams(host.getParams().copy(clarity = v), c, if (c) "Clarity" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Dehaze", -100f, 100f, 0f, p.dehaze) { v, c ->
            host.updateParams(host.getParams().copy(dehaze = v), c, if (c) "Dehaze" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Vignette Amount", -100f, 100f, 0f, p.vignetteAmount) { v, c ->
            host.updateParams(host.getParams().copy(vignetteAmount = v), c, if (c) "Vignette" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Vignette Midpoint", 0f, 100f, 50f, p.vignetteMidpoint) { v, c ->
            host.updateParams(host.getParams().copy(vignetteMidpoint = v), c, if (c) "Vig Midpoint" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Vignette Feather", 0f, 100f, 50f, p.vignetteFeather) { v, c ->
            host.updateParams(host.getParams().copy(vignetteFeather = v), c, if (c) "Vig Feather" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Film Grain Amount", 0f, 100f, 0f, p.grainAmount) { v, c ->
            host.updateParams(host.getParams().copy(grainAmount = v), c, if (c) "Grain" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Grain Size", 0f, 100f, 25f, p.grainSize) { v, c ->
            host.updateParams(host.getParams().copy(grainSize = v), c, if (c) "Grain Size" else null)
        }
    }

    private fun buildDetailPanel(host: PanelHost) {
        val p = host.getParams()
        val tabs = listOf("Sharpening", "Noise Reduction")
        DarkroomUiFactory.addChipBar(
            host.container,
            tabs.mapIndexed { i, t -> t to (i == detailSubTab) }
        ) { idx ->
            detailSubTab = idx
            host.rebuildPanel()
        }
        if (detailSubTab == 0) {
            DarkroomUiFactory.addSliderRow(host.container, "Sharpening", 0f, 150f, 20f, p.sharpness) { v, c ->
                host.updateParams(host.getParams().copy(sharpness = v), c, if (c) "Sharpen" else null)
            }
            DarkroomUiFactory.addSliderRow(host.container, "Radius", 0.5f, 3f, 1f, p.sharpenRadius, 1, " px") { v, c ->
                host.updateParams(host.getParams().copy(sharpenRadius = v), c, if (c) "Sharpen Radius" else null)
            }
        } else {
            DarkroomUiFactory.addSliderRow(host.container, "Luminance Noise Reduction", 0f, 100f, 0f, p.luminanceNr) { v, c ->
                host.updateParams(host.getParams().copy(luminanceNr = v), c, if (c) "Lum NR" else null)
            }
            DarkroomUiFactory.addSliderRow(host.container, "Color Noise Reduction", 0f, 100f, 15f, p.colorNr) { v, c ->
                host.updateParams(host.getParams().copy(colorNr = v), c, if (c) "Color NR" else null)
            }
        }
    }

    private fun buildOpticsPanel(host: PanelHost) {
        val p = host.getParams()
        DarkroomUiFactory.addSliderRow(host.container, "Lens Distortion", -100f, 100f, 0f, p.lensDistortion) { v, c ->
            host.updateParams(host.getParams().copy(lensDistortion = v), c, if (c) "Distortion" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Lens Vignetting", -100f, 100f, 0f, p.lensVignetting) { v, c ->
            host.updateParams(host.getParams().copy(lensVignetting = v), c, if (c) "Lens Vig" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Lens Blur (Depth Bokeh)", 0f, 100f, 0f, p.lensBlurAmount) { v, c ->
            host.updateParams(host.getParams().copy(lensBlurAmount = v), c, if (c) "Lens Blur" else null)
        }
        DarkroomUiFactory.addSliderRow(host.container, "Focal Plane Y Position", 5f, 95f, 50f, p.lensBlurFocalCenter, 0, "%") { v, c ->
            host.updateParams(host.getParams().copy(lensBlurFocalCenter = v), c, if (c) "Focus Y" else null)
        }
    }

    private fun buildCropPanel(host: PanelHost) {
        val ctx = host.context
        val p = host.getParams()
        val cropTabs = listOf("Crop & Straighten", "Geometry Warp")
        DarkroomUiFactory.addChipBar(
            host.container,
            cropTabs.mapIndexed { i, t -> t to (i == cropSubTab) }
        ) { idx ->
            cropSubTab = idx
            host.rebuildPanel()
        }

        if (cropSubTab == 0) {
            DarkroomUiFactory.addChipBar(
                host.container,
                AspectRatioPreset.entries.map { it.label to (it == p.cropAspect) }
            ) { idx ->
                val aspect = AspectRatioPreset.entries[idx]
                val ratio = aspect.ratio
                val updated = if (ratio != null) {
                    val (l, t, r, b) = applyAspectToRect(ratio)
                    host.getParams().copy(cropAspect = aspect, cropLeft = l, cropTop = t, cropRight = r, cropBottom = b)
                } else {
                    host.getParams().copy(cropAspect = aspect)
                }
                host.updateParams(updated, true, "Crop ${aspect.label}")
                host.rebuildPanel()
            }

            val btnRow = LinearLayout(ctx).apply { orientation = LinearLayout.HORIZONTAL }
            fun geoBtn(txt: String, marginEndDp: Int = 6, action: () -> Unit) =
                pillBtn(ctx, txt, heightDp = 38, textSp = 11.5f, weight = 1f, marginEndDp = marginEndDp, onClick = action)
            btnRow.addView(geoBtn("Rotate 90°") {
                val cur = host.getParams()
                host.updateParams(cur.copy(rotationDegrees = (cur.rotationDegrees + 90f) % 360f), true, "Rotate 90°")
            })
            btnRow.addView(geoBtn("Flip H") {
                val cur = host.getParams()
                host.updateParams(cur.copy(flipHorizontal = !cur.flipHorizontal), true, "Flip H")
            })
            btnRow.addView(geoBtn("Flip V") {
                val cur = host.getParams()
                host.updateParams(cur.copy(flipVertical = !cur.flipVertical), true, "Flip V")
            })
            btnRow.addView(geoBtn("Reset Crop", marginEndDp = 0) {
                host.updateParams(
                    host.getParams().copy(
                        cropLeft = 0f, cropTop = 0f, cropRight = 1f, cropBottom = 1f,
                        cropAngle = 0f, rotationDegrees = 0f, flipHorizontal = false, flipVertical = false,
                        cropAspect = AspectRatioPreset.ORIGINAL
                    ),
                    true,
                    "Reset Crop"
                )
                host.rebuildPanel()
            })
            host.container.addView(btnRow)

            DarkroomUiFactory.addSliderRow(host.container, "Straighten Angle", -45f, 45f, 0f, p.cropAngle, 1, "°") { v, c ->
                host.updateParams(host.getParams().copy(cropAngle = v), c, if (c) "Straighten" else null)
            }
        } else {
            DarkroomUiFactory.addSliderRow(host.container, "Vertical Keystone", -100f, 100f, 0f, p.perspectiveVertical) { v, c ->
                host.updateParams(host.getParams().copy(perspectiveVertical = v), c, if (c) "Vertical Transform" else null)
            }
            DarkroomUiFactory.addSliderRow(host.container, "Horizontal Keystone", -100f, 100f, 0f, p.perspectiveHorizontal) { v, c ->
                host.updateParams(host.getParams().copy(perspectiveHorizontal = v), c, if (c) "Horizontal Transform" else null)
            }
            DarkroomUiFactory.addSliderRow(host.container, "Aspect Distortion", -100f, 100f, 0f, p.perspectiveAspect) { v, c ->
                host.updateParams(host.getParams().copy(perspectiveAspect = v), c, if (c) "Aspect Warp" else null)
            }
            DarkroomUiFactory.addSliderRow(host.container, "Geometry Scale", 50f, 150f, 100f, p.geometryScale, 0, "%") { v, c ->
                host.updateParams(host.getParams().copy(geometryScale = v), c, if (c) "Geometry Scale" else null)
            }
            DarkroomUiFactory.addSliderRow(host.container, "X Offset", -100f, 100f, 0f, p.geometryOffsetX) { v, c ->
                host.updateParams(host.getParams().copy(geometryOffsetX = v), c, if (c) "X Offset" else null)
            }
            DarkroomUiFactory.addSliderRow(host.container, "Y Offset", -100f, 100f, 0f, p.geometryOffsetY) { v, c ->
                host.updateParams(host.getParams().copy(geometryOffsetY = v), c, if (c) "Y Offset" else null)
            }
            val btnResetGeo = pillBtn(
                ctx = ctx,
                title = "Reset Geometry",
                heightDp = 36,
                textSp = 11.5f,
                topMarginDp = 6
            ) {
                host.updateParams(
                    host.getParams().copy(
                        perspectiveVertical = 0f,
                        perspectiveHorizontal = 0f,
                        perspectiveAspect = 0f,
                        geometryScale = 100f,
                        geometryOffsetX = 0f,
                        geometryOffsetY = 0f
                    ),
                    true,
                    "Reset Geometry"
                )
                host.rebuildPanel()
            }
            host.container.addView(btnResetGeo)
        }
    }

    private fun applyAspectToRect(ratio: Float): FloatArray {
        return if (ratio >= 1f) {
            val h = (1f / ratio).coerceIn(0.25f, 1f)
            val top = (1f - h) * 0.5f
            floatArrayOf(0f, top, 1f, top + h)
        } else {
            val w = ratio.coerceIn(0.25f, 1f)
            val left = (1f - w) * 0.5f
            floatArrayOf(left, 0f, left + w, 1f)
        }
    }

    private fun buildMaskPanel(host: PanelHost) {
        val ctx = host.context
        val btnRow = LinearLayout(ctx).apply { orientation = LinearLayout.HORIZONTAL }
        fun addMaskBtn(label: String, kind: MaskPrimitiveKind, marginEndDp: Int = 6) =
            pillBtn(ctx, label, heightDp = 38, textSp = 11.5f, weight = 1f, marginEndDp = marginEndDp) {
                val cur = host.getParams()
                val m = MaskGroup(
                    name = "${kind.name.substringBefore('_')} ${cur.masks.size + 1}",
                    kind = kind
                )
                host.setActiveMaskId(m.id)
                host.updateParams(cur.copy(masks = cur.masks + m), true, "Add ${m.name}")
                host.rebuildPanel()
            }
        btnRow.addView(addMaskBtn("+ Radial", MaskPrimitiveKind.RADIAL_GRADIENT))
        btnRow.addView(addMaskBtn("+ Linear", MaskPrimitiveKind.LINEAR_GRADIENT))
        btnRow.addView(addMaskBtn("+ Brush", MaskPrimitiveKind.BRUSH, marginEndDp = 0))
        host.container.addView(btnRow)

        val cur = host.getParams()
        if (cur.masks.isEmpty()) {
            val tv = TextView(ctx).apply {
                text = "Tap + Radial, + Linear, or + Brush above, then drag directly on the photo."
                setTextColor(ctx.getColor(R.color.lr_text_muted))
                textSize = 12f
                setPadding(0, DarkroomUiFactory.dp(ctx, 8), 0, 0)
            }
            host.container.addView(tv)
            return
        }

        val activeId = host.getActiveMaskId() ?: cur.masks.last().id
        host.setActiveMaskId(activeId)

        DarkroomUiFactory.addChipBar(host.container, cur.masks.map { it.name to (it.id == activeId) }) { idx ->
            host.setActiveMaskId(cur.masks[idx].id)
            host.rebuildPanel()
        }

        val active = cur.masks.firstOrNull { it.id == activeId } ?: cur.masks.last()
        val modRow = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            setPadding(0, DarkroomUiFactory.dp(ctx, 4), 0, DarkroomUiFactory.dp(ctx, 4))
        }
        val btnInv = pillBtn(
            ctx = ctx,
            title = if (active.invert) "Invert: ON" else "Invert: OFF",
            accent = active.invert,
            heightDp = 36,
            textSp = 11.5f,
            marginEndDp = 8
        ) {
            val updated = active.copy(invert = !active.invert)
            val pNow = host.getParams()
            host.updateParams(pNow.copy(masks = pNow.masks.map { if (it.id == updated.id) updated else it }), true, "Invert Mask")
            host.rebuildPanel()
        }
        val btnDel = pillBtn(
            ctx = ctx,
            title = "Delete Mask",
            textColorRes = R.color.lr_danger,
            heightDp = 36,
            textSp = 11.5f
        ) {
            val pNow = host.getParams()
            val rem = pNow.masks.filterNot { it.id == active.id }
            host.setActiveMaskId(rem.lastOrNull()?.id)
            host.updateParams(pNow.copy(masks = rem), true, "Delete Mask")
            host.rebuildPanel()
        }
        modRow.addView(btnInv)
        modRow.addView(btnDel)
        host.container.addView(modRow)

        fun updateLocal(commit: Boolean, mutator: (MaskGroup) -> MaskGroup) {
            val pNow = host.getParams()
            val newMasks = pNow.masks.map { if (it.id == active.id) mutator(it) else it }
            host.updateParams(pNow.copy(masks = newMasks), commit, if (commit) "Mask Adjust" else null)
        }

        DarkroomUiFactory.addSliderRow(host.container, "Local Exposure", -3f, 3f, 0f, active.adjust.exposure, 2, " EV") { v, c ->
            updateLocal(c) { it.copy(adjust = it.adjust.copy(exposure = v)) }
        }
        DarkroomUiFactory.addSliderRow(host.container, "Local Contrast", -100f, 100f, 0f, active.adjust.contrast) { v, c ->
            updateLocal(c) { it.copy(adjust = it.adjust.copy(contrast = v)) }
        }
        DarkroomUiFactory.addSliderRow(host.container, "Local Shadows", -100f, 100f, 0f, active.adjust.shadows) { v, c ->
            updateLocal(c) { it.copy(adjust = it.adjust.copy(shadows = v)) }
        }
        DarkroomUiFactory.addSliderRow(host.container, "Local Temperature", -100f, 100f, 0f, active.adjust.temperature) { v, c ->
            updateLocal(c) { it.copy(adjust = it.adjust.copy(temperature = v)) }
        }
        DarkroomUiFactory.addSliderRow(host.container, "Local Saturation", -100f, 100f, 0f, active.adjust.saturation) { v, c ->
            updateLocal(c) { it.copy(adjust = it.adjust.copy(saturation = v)) }
        }
    }

    private fun buildHealPanel(host: PanelHost) {
        val ctx = host.context
        val cur = host.getParams()
        val info = TextView(ctx).apply {
            text = "Tap any blemish or dust spot on the photo to heal (${cur.retouchSpots.size} spot(s) placed)."
            setTextColor(ctx.getColor(R.color.lr_text_muted))
            textSize = 12f
        }
        host.container.addView(info)

        val row = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            setPadding(0, DarkroomUiFactory.dp(ctx, 8), 0, 0)
        }
        val hasSpots = cur.retouchSpots.isNotEmpty()
        val btnUndoSpot = pillBtn(
            ctx = ctx,
            title = "Undo Spot",
            heightDp = 38,
            textSp = 12f,
            marginEndDp = 8,
            enabled = hasSpots
        ) {
            val pNow = host.getParams()
            if (pNow.retouchSpots.isNotEmpty()) {
                host.updateParams(pNow.copy(retouchSpots = pNow.retouchSpots.dropLast(1)), true, "Undo Heal Spot")
                host.rebuildPanel()
            }
        }
        val btnClearSpots = pillBtn(
            ctx = ctx,
            title = "Clear All",
            textColorRes = R.color.lr_danger,
            heightDp = 38,
            textSp = 12f,
            enabled = hasSpots
        ) {
            host.updateParams(host.getParams().copy(retouchSpots = emptyList()), true, "Clear Heal Spots")
            host.rebuildPanel()
        }
        row.addView(btnUndoSpot)
        row.addView(btnClearSpots)
        host.container.addView(row)
    }
}
