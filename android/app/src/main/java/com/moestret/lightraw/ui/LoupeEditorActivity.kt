package com.moestret.lightraw.ui

import android.app.AlertDialog
import android.graphics.Bitmap
import android.graphics.drawable.GradientDrawable
import android.net.Uri
import android.os.Bundle
import android.view.Gravity
import android.view.View
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.TextView
import android.widget.Toast
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat
import com.moestret.lightraw.R
import com.moestret.lightraw.data.CatalogDbHelper
import com.moestret.lightraw.data.MediaStoreImporter
import com.moestret.lightraw.data.XmpPresetCodec
import com.moestret.lightraw.databinding.ActivityLoupeBinding
import com.moestret.lightraw.engine.AiColorMatchEngine
import com.moestret.lightraw.engine.KotlinPv2012Engine
import com.moestret.lightraw.engine.NativeBridge
import com.moestret.lightraw.model.ColorLabel
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.DevelopPreset
import com.moestret.lightraw.model.PhotoAsset
import com.moestret.lightraw.model.PickFlag
import com.moestret.lightraw.model.RetouchArea
import com.moestret.lightraw.ui.views.LoupeImageView
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * S03 Touch Loupe Studio integrating real-time PV2012 pipeline, AI Auto Tone, AI Color Match,
 * Tone Curve, 8-Band HSL, Color Grading, Presets & XMP, Crop, Masking, Heal, Copy/Paste, Versions, and Export.
 */
class LoupeEditorActivity : AppCompatActivity() {

    private lateinit var binding: ActivityLoupeBinding
    private lateinit var catalog: CatalogDbHelper
    private val scope = CoroutineScope(Dispatchers.Main + Job())

    private var catalogIds: List<Long> = emptyList()
    private var currentPhotoIndex = 0
    private var asset: PhotoAsset? = null
    private var previewSource: Bitmap? = null
    private var currentParams = DevelopParams()

    private val undoStack = ArrayDeque<DevelopParams>()
    private val redoStack = ArrayDeque<DevelopParams>()

    private var activeTab = LoupePanels.ToolTab.LIGHT
    private var renderJob: Job? = null
    private var pendingXmpExport: DevelopPreset? = null
    private var pendingWatermarkGraphicCallback: ((String, String) -> Unit)? = null
    private var isInfoHudVisible = false
    private var isRateBarVisible = false

    private val importXmpLauncher =
        registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri: Uri? ->
            uri ?: return@registerForActivityResult
            val xml = runCatching {
                contentResolver.openInputStream(uri)?.bufferedReader()?.use { it.readText() }
            }.getOrNull() ?: return@registerForActivityResult
            val fallback = uri.lastPathSegment?.substringAfterLast('/')?.substringBeforeLast('.') ?: "Imported XMP"
            val preset = XmpPresetCodec.parseXmp(xml, fallback)
            if (preset != null) {
                catalog.upsertPreset(preset)
                updateParams(preset.params, commit = true, historyLabel = "XMP: ${preset.name}")
                rebuildActivePanel()
                Toast.makeText(this, "Imported preset '${preset.name}'", Toast.LENGTH_SHORT).show()
            } else {
                Toast.makeText(this, "Invalid Camera Raw .xmp file", Toast.LENGTH_SHORT).show()
            }
        }

    private val exportXmpLauncher =
        registerForActivityResult(ActivityResultContracts.CreateDocument("application/rdf+xml")) { uri: Uri? ->
            val preset = pendingXmpExport ?: return@registerForActivityResult
            uri ?: return@registerForActivityResult
            val xml = XmpPresetCodec.exportToXmp(preset)
            runCatching {
                contentResolver.openOutputStream(uri)?.bufferedWriter()?.use { it.write(xml) }
            }.onSuccess {
                Toast.makeText(this, "Exported '${preset.name}.xmp'", Toast.LENGTH_SHORT).show()
            }
        }

    private val pickReferenceImageLauncher =
        registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri: Uri? ->
            uri ?: return@registerForActivityResult
            runColorMatchFromUri(uri)
        }

    private val pickWatermarkGraphicLauncher =
        registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri: Uri? ->
            uri ?: return@registerForActivityResult
            val persisted = MediaStoreImporter.persistWatermarkGraphic(this, uri)
            if (persisted != null) {
                pendingWatermarkGraphicCallback?.invoke(persisted.first, persisted.second)
            } else {
                Toast.makeText(this, "Could not load selected watermark image", Toast.LENGTH_SHORT).show()
            }
            pendingWatermarkGraphicCallback = null
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityLoupeBinding.inflate(layoutInflater)
        setContentView(binding.root)
        catalog = CatalogDbHelper(this)

        val photoId = intent.getLongExtra("photo_id", -1L)
        val extraIds = intent.getLongArrayExtra("photo_ids")?.toList()?.takeIf { it.isNotEmpty() }
        catalogIds = extraIds ?: catalog.getAllPhotos().map { it.idLocal }.ifEmpty { listOf(photoId) }
        currentPhotoIndex = catalogIds.indexOf(photoId).coerceAtLeast(0)

        asset = catalog.getPhotoById(photoId)
        if (asset == null) {
            finish()
            return
        }
        currentParams = asset!!.params.deepCopy()
        binding.tvEngineStatus.text = if (NativeBridge.isNativeAvailable) "RUST JNI + PV2012" else "PV2012 ENGINE"

        setupTopBar()
        setupCanvasCallbacks()
        setupQuickStrip()
        setupToolRail()
        updateInfoHud()
        updateFilmstripUi()
        updateHistoryButtons()
        loadSourceBitmap()
    }

    private fun setupTopBar() {
        binding.btnBack.setOnClickListener { finish() }
        binding.btnUndo.setOnClickListener { performUndo() }
        binding.btnRedo.setOnClickListener { performRedo() }

        binding.btnInfoToggle.setOnClickListener {
            isInfoHudVisible = !isInfoHudVisible
            binding.loupeInfoHud.visibility = if (isInfoHudVisible) View.VISIBLE else View.GONE
            binding.btnInfoToggle.setColorFilter(
                ContextCompat.getColor(this, if (isInfoHudVisible) R.color.lr_accent_light else R.color.lr_text_muted)
            )
            updateInfoHud()
        }

        binding.btnHistToggle.setOnClickListener {
            val show = binding.histogramView.visibility != View.VISIBLE
            binding.histogramView.visibility = if (show) View.VISIBLE else View.GONE
            binding.btnHistToggle.setColorFilter(
                ContextCompat.getColor(this, if (show) R.color.lr_accent_light else R.color.lr_text_muted)
            )
        }

        binding.btnRateMode.setOnClickListener {
            isRateBarVisible = !isRateBarVisible
            binding.rateBar.visibility = if (isRateBarVisible) View.VISIBLE else View.GONE
            binding.btnRateMode.setColorFilter(
                ContextCompat.getColor(this, if (isRateBarVisible) R.color.lr_accent_light else R.color.lr_text_muted)
            )
            if (isRateBarVisible) rebuildRateBar()
        }

        binding.btnPrevPhoto.setOnClickListener { switchToPhotoIndex(currentPhotoIndex - 1) }
        binding.btnNextPhoto.setOnClickListener { switchToPhotoIndex(currentPhotoIndex + 1) }

        binding.btnMoreMenu.setOnClickListener { showMoreMenu() }
        binding.btnExport.setOnClickListener { launchExportDialog() }
    }

    private fun switchToPhotoIndex(newIdx: Int) {
        if (newIdx !in catalogIds.indices) return
        val nextAsset = catalog.getPhotoById(catalogIds[newIdx]) ?: return
        renderJob?.cancel()
        currentPhotoIndex = newIdx
        asset = nextAsset
        currentParams = nextAsset.params.deepCopy()
        undoStack.clear()
        redoStack.clear()
        binding.loupeView.setBeforePreview(false)
        binding.loupeView.updateParamsOverlay(currentParams)
        updateInfoHud()
        updateFilmstripUi()
        updateHistoryButtons()
        if (isRateBarVisible) rebuildRateBar()
        loadSourceBitmap()
    }

    private fun updateFilmstripUi() {
        val total = catalogIds.size.coerceAtLeast(1)
        binding.tvFilmstripIndex.text = "${currentPhotoIndex + 1} / $total"
        val hasPrev = currentPhotoIndex > 0
        val hasNext = currentPhotoIndex < catalogIds.lastIndex
        binding.btnPrevPhoto.isEnabled = hasPrev
        binding.btnNextPhoto.isEnabled = hasNext
        binding.btnPrevPhoto.alpha = if (hasPrev) 1f else 0.35f
        binding.btnNextPhoto.alpha = if (hasNext) 1f else 0.35f
    }

    private fun updateInfoHud() {
        val cur = asset ?: return
        binding.tvInfoTitle.text = "${cur.displayName}${if (cur.isRaw) " · RAW" else ""}"
        val dimText = if (cur.width > 0 && cur.height > 0) "${cur.width}×${cur.height}" else "Catalog Photo"
        val starsText = if (cur.rating > 0) "★".repeat(cur.rating) else "0★"
        val pickText = when (cur.pickFlag) {
            PickFlag.PICKED -> "⚑ Pick"
            PickFlag.REJECTED -> "✕ Reject"
            PickFlag.UNFLAGGED -> "Unflagged"
        }
        val colorText = if (cur.colorLabel != ColorLabel.NONE) " · ${cur.colorLabel.code}" else ""
        val presetText = currentParams.appliedPresetName.takeIf { it.isNotBlank() }?.let { " · $it" } ?: ""
        binding.tvInfoDetails.text = "$dimText · $starsText · $pickText$colorText · ${cur.albumName}$presetText"
    }

    private fun updateHistoryButtons() {
        binding.btnUndo.alpha = if (undoStack.isNotEmpty()) 1f else 0.38f
        binding.btnRedo.alpha = if (redoStack.isNotEmpty()) 1f else 0.38f
    }

    private fun rebuildRateBar() {
        val cur = asset ?: return
        binding.rateBarContent.removeAllViews()

        fun addRatePill(textLabel: String, selected: Boolean, accentHex: Int? = null, onClick: () -> Unit) {
            val pill = TextView(this).apply {
                text = textLabel
                textSize = 11.5f
                gravity = Gravity.CENTER
                setTextColor(
                    ContextCompat.getColor(
                        this@LoupeEditorActivity,
                        if (selected) R.color.lr_on_accent else R.color.lr_text
                    )
                )
                setPadding(
                    DarkroomUiFactory.dp(this@LoupeEditorActivity, 10),
                    DarkroomUiFactory.dp(this@LoupeEditorActivity, 6),
                    DarkroomUiFactory.dp(this@LoupeEditorActivity, 10),
                    DarkroomUiFactory.dp(this@LoupeEditorActivity, 6)
                )
                if (accentHex != null) {
                    background = GradientDrawable().apply {
                        cornerRadius = DarkroomUiFactory.dp(this@LoupeEditorActivity, 16).toFloat()
                        setColor(if (selected) accentHex else 0xFF2C2C2C.toInt())
                        setStroke(
                            DarkroomUiFactory.dp(this@LoupeEditorActivity, if (selected) 2 else 1),
                            accentHex
                        )
                    }
                } else {
                    setBackgroundResource(if (selected) R.drawable.bg_pill_accent else R.drawable.bg_pill_neutral)
                }
                layoutParams = LinearLayout.LayoutParams(
                    LinearLayout.LayoutParams.WRAP_CONTENT,
                    DarkroomUiFactory.dp(this@LoupeEditorActivity, 32)
                ).apply { marginEnd = DarkroomUiFactory.dp(this@LoupeEditorActivity, 6) }
                setOnClickListener { onClick() }
            }
            binding.rateBarContent.addView(pill)
        }

        for (stars in 1..5) {
            addRatePill("${stars}★", cur.rating == stars) {
                val next = if (cur.rating == stars) 0 else stars
                catalog.updateRating(cur.idLocal, next)
                asset = cur.copy(rating = next)
                updateInfoHud()
                rebuildRateBar()
            }
        }

        addRatePill("⚑ Pick", cur.pickFlag == PickFlag.PICKED) {
            val next = if (cur.pickFlag == PickFlag.PICKED) PickFlag.UNFLAGGED else PickFlag.PICKED
            catalog.updatePickFlag(cur.idLocal, next)
            asset = cur.copy(pickFlag = next)
            updateInfoHud()
            rebuildRateBar()
        }

        addRatePill("✕ Reject", cur.pickFlag == PickFlag.REJECTED) {
            val next = if (cur.pickFlag == PickFlag.REJECTED) PickFlag.UNFLAGGED else PickFlag.REJECTED
            catalog.updatePickFlag(cur.idLocal, next)
            asset = cur.copy(pickFlag = next)
            updateInfoHud()
            rebuildRateBar()
        }

        for (lbl in ColorLabel.entries.filter { it != ColorLabel.NONE }) {
            addRatePill(lbl.code, cur.colorLabel == lbl, lbl.colorHex) {
                val next = if (cur.colorLabel == lbl) ColorLabel.NONE else lbl
                catalog.updateColorLabel(cur.idLocal, next)
                asset = cur.copy(colorLabel = next)
                updateInfoHud()
                rebuildRateBar()
            }
        }
    }

    private fun setupCanvasCallbacks() {
        binding.loupeView.onBeforeStateChanged = { isBefore ->
            binding.tvBeforeBadge.visibility = if (isBefore) View.VISIBLE else View.GONE
        }
        binding.loupeView.onCropChanged = { l, t, r, b, commit ->
            val upd = currentParams.copy(cropLeft = l, cropTop = t, cropRight = r, cropBottom = b)
            updateParams(upd, commit, if (commit) "Crop" else null)
        }
        binding.loupeView.onMaskUpdated = { updatedMask, commit ->
            val upd = currentParams.copy(
                masks = currentParams.masks.map { if (it.id == updatedMask.id) updatedMask else it }
            )
            updateParams(upd, commit, if (commit) "Mask Geometry" else null)
        }
        binding.loupeView.onHealTap = { nx, ny ->
            val spot = RetouchArea(x = nx, y = ny)
            val upd = currentParams.copy(retouchSpots = currentParams.retouchSpots + spot)
            updateParams(upd, commit = true, historyLabel = "Heal Spot")
            rebuildActivePanel()
        }
    }

    private fun setupQuickStrip() {
        binding.btnQuickAuto.setOnClickListener { runAutoTone() }
        binding.btnQuickMatchRef.setOnClickListener { showMatchReferencePicker() }
        binding.btnQuickBeforeAfter.setOnClickListener {
            val showing = binding.tvBeforeBadge.visibility != View.VISIBLE
            binding.loupeView.setBeforePreview(showing)
        }
        binding.btnQuickCopy.setOnClickListener {
            DarkroomUiFactory.showCopySettingsDialog(this, SessionClipboard.copyMask) { mask ->
                SessionClipboard.copyMask = mask
                SessionClipboard.copiedParams = currentParams.deepCopy()
                Toast.makeText(this, "Settings copied", Toast.LENGTH_SHORT).show()
            }
        }
        binding.btnQuickPaste.setOnClickListener {
            val copied = SessionClipboard.copiedParams
            if (copied == null) {
                Toast.makeText(this, "Nothing copied yet", Toast.LENGTH_SHORT).show()
            } else {
                val merged = SessionClipboard.copyMask.applyTo(copied, currentParams)
                updateParams(merged, commit = true, historyLabel = "Paste Settings")
                rebuildActivePanel()
            }
        }
    }

    private fun setupToolRail() {
        rebuildToolRail()
    }

    private fun rebuildToolRail() {
        binding.toolRailContainer.removeAllViews()
        for (tab in LoupePanels.ToolTab.entries) {
            val selected = tab == activeTab
            val btn = Button(this).apply {
                text = tab.label
                textSize = 11.5f
                isAllCaps = false
                gravity = Gravity.CENTER
                minHeight = DarkroomUiFactory.dp(this@LoupeEditorActivity, 48)
                setPadding(
                    DarkroomUiFactory.dp(this@LoupeEditorActivity, 12),
                    0,
                    DarkroomUiFactory.dp(this@LoupeEditorActivity, 12),
                    0
                )
                setTextColor(getColor(if (selected) R.color.lr_on_accent else R.color.lr_text))
                setBackgroundResource(if (selected) R.drawable.bg_pill_accent else R.drawable.bg_pill_neutral)
                layoutParams = LinearLayout.LayoutParams(
                    LinearLayout.LayoutParams.WRAP_CONTENT,
                    DarkroomUiFactory.dp(this@LoupeEditorActivity, 40)
                ).apply { marginEnd = DarkroomUiFactory.dp(this@LoupeEditorActivity, 6) }
                setOnClickListener {
                    activeTab = tab
                    binding.loupeView.mode = when (tab) {
                        LoupePanels.ToolTab.CROP -> LoupeImageView.InteractionMode.CROP
                        LoupePanels.ToolTab.MASK -> LoupeImageView.InteractionMode.MASK
                        LoupePanels.ToolTab.HEAL -> LoupeImageView.InteractionMode.HEAL
                        else -> LoupeImageView.InteractionMode.VIEW
                    }
                    rebuildToolRail()
                    rebuildActivePanel()
                    schedulePreviewRender(draft = true)
                }
            }
            binding.toolRailContainer.addView(btn)
        }
    }

    private fun rebuildActivePanel() {
        val host = LoupePanels.PanelHost(
            context = this,
            scope = scope,
            container = binding.panelContainer,
            thumbBitmap = previewSource?.let { Bitmap.createScaledBitmap(it, 120, 120, true) },
            getParams = { currentParams },
            updateParams = { newP, commit, label -> updateParams(newP, commit, label) },
            triggerAutoTone = { runAutoTone() },
            triggerMatchRef = { showMatchReferencePicker() },
            triggerImportXmp = { importXmpLauncher.launch(arrayOf("*/*")) },
            triggerExportXmp = { preset ->
                pendingXmpExport = preset
                exportXmpLauncher.launch("${preset.name.replace(" ", "_")}.xmp")
            },
            getPresets = { catalog.getAllPresets() },
            saveNewPreset = { name, cat, params ->
                val p = DevelopPreset(
                    id = "user_${System.currentTimeMillis()}",
                    category = cat,
                    name = name,
                    isUser = true,
                    isFavorite = false,
                    params = params
                )
                catalog.upsertPreset(p)
                Toast.makeText(this, "Saved preset '$name'", Toast.LENGTH_SHORT).show()
            },
            renamePreset = { id, newName ->
                catalog.renamePreset(id, newName)
                Toast.makeText(this, "Renamed preset to '$newName'", Toast.LENGTH_SHORT).show()
            },
            toggleFavoritePreset = { id, isFav ->
                catalog.togglePresetFavorite(id, isFav)
            },
            updateUserPreset = { updatedPreset ->
                catalog.upsertPreset(updatedPreset)
                Toast.makeText(this, "Updated preset '${updatedPreset.name}'", Toast.LENGTH_SHORT).show()
            },
            deletePreset = { id -> catalog.deletePreset(id) },
            setActiveMaskId = { id ->
                binding.loupeView.activeMaskId = id
                binding.loupeView.invalidate()
            },
            getActiveMaskId = { binding.loupeView.activeMaskId },
            rebuildPanel = { rebuildActivePanel() }
        )
        LoupePanels.renderTab(activeTab, host)
    }

    private fun loadSourceBitmap() {
        val curAsset = asset ?: return
        val requestedId = curAsset.idLocal
        scope.launch {
            val bmp = withContext(Dispatchers.IO) {
                MediaStoreImporter.decodeScaledBitmap(this@LoupeEditorActivity, curAsset.uriString, 1280)
            }
            if (bmp == null) {
                if (asset?.idLocal == requestedId) {
                    Toast.makeText(this@LoupeEditorActivity, "Unable to decode photo file", Toast.LENGTH_LONG).show()
                    finish()
                }
                return@launch
            }
            if (asset?.idLocal != requestedId) {
                bmp.recycle()
                return@launch
            }
            previewSource = bmp
            binding.loupeView.setImages(bmp, bmp, currentParams)
            rebuildActivePanel()
            schedulePreviewRender(draft = false)
        }
    }

    private fun updateParams(newParams: DevelopParams, commit: Boolean, historyLabel: String?) {
        if (commit && newParams != currentParams) {
            undoStack.addLast(currentParams.deepCopy())
            if (undoStack.size > 40) undoStack.removeFirst()
            redoStack.clear()
        }
        currentParams = newParams
        binding.loupeView.updateParamsOverlay(currentParams)
        if (commit) {
            val pid = asset?.idLocal ?: return
            catalog.saveDevelopParams(pid, currentParams)
            if (!historyLabel.isNullOrBlank()) {
                catalog.addHistoryStep(pid, historyLabel, "", currentParams)
            }
        }
        updateHistoryButtons()
        if (isInfoHudVisible) updateInfoHud()
        schedulePreviewRender(draft = !commit)
    }

    private fun schedulePreviewRender(draft: Boolean) {
        val src = previewSource ?: return
        val paramsSnapshot = currentParams.deepCopy()
        val applyCrop = binding.loupeView.mode != LoupeImageView.InteractionMode.CROP
        renderJob?.cancel()
        renderJob = scope.launch {
            val result = withContext(Dispatchers.Default) {
                KotlinPv2012Engine.render(src, paramsSnapshot, draftInteractive = draft, applyCrop = applyCrop)
            }
            binding.loupeView.setImages(result.bitmap, src, paramsSnapshot)
            binding.histogramView.updateHistograms(result.histR, result.histG, result.histB, result.histLum)
        }
    }

    private fun performUndo() {
        if (undoStack.isEmpty()) {
            Toast.makeText(this, "Nothing to undo", Toast.LENGTH_SHORT).show()
            return
        }
        redoStack.addLast(currentParams.deepCopy())
        currentParams = undoStack.removeLast()
        asset?.let { catalog.saveDevelopParams(it.idLocal, currentParams) }
        updateHistoryButtons()
        if (isInfoHudVisible) updateInfoHud()
        rebuildActivePanel()
        schedulePreviewRender(draft = false)
    }

    private fun performRedo() {
        if (redoStack.isEmpty()) {
            Toast.makeText(this, "Nothing to redo", Toast.LENGTH_SHORT).show()
            return
        }
        undoStack.addLast(currentParams.deepCopy())
        currentParams = redoStack.removeLast()
        asset?.let { catalog.saveDevelopParams(it.idLocal, currentParams) }
        updateHistoryButtons()
        if (isInfoHudVisible) updateInfoHud()
        rebuildActivePanel()
        schedulePreviewRender(draft = false)
    }

    private fun runAutoTone() {
        val src = previewSource ?: return
        scope.launch {
            val auto = withContext(Dispatchers.Default) {
                AiColorMatchEngine.autoTone(src, currentParams)
            }
            updateParams(auto, commit = true, historyLabel = "AI Auto Tone")
            rebuildActivePanel()
            Toast.makeText(this@LoupeEditorActivity, "Applied AI Auto Tone", Toast.LENGTH_SHORT).show()
        }
    }

    private fun showMatchReferencePicker() {
        val catalogPhotos = catalog.getAllPhotos().filter { it.idLocal != asset?.idLocal }
        val options = mutableListOf("Browse Device Gallery for Reference…")
        options.addAll(catalogPhotos.map { "Catalog: ${it.displayName}" })

        AlertDialog.Builder(this)
            .setTitle("AI Color Match — Select Reference Photo")
            .setItems(options.toTypedArray()) { _, which ->
                if (which == 0) {
                    pickReferenceImageLauncher.launch(arrayOf("image/*"))
                } else {
                    val refAsset = catalogPhotos[which - 1]
                    scope.launch {
                        val refBmp = withContext(Dispatchers.IO) {
                            MediaStoreImporter.decodeScaledBitmap(this@LoupeEditorActivity, refAsset.uriString, 720)
                        }
                        if (refBmp != null) {
                            applyColorMatch(refBmp, refAsset.params)
                            refBmp.recycle()
                        }
                    }
                }
            }
            .show()
    }

    private fun runColorMatchFromUri(uri: Uri) {
        scope.launch {
            val refBmp = withContext(Dispatchers.IO) {
                MediaStoreImporter.decodeScaledBitmap(this@LoupeEditorActivity, uri.toString(), 720)
            }
            if (refBmp != null) {
                applyColorMatch(refBmp, null)
                refBmp.recycle()
            } else {
                Toast.makeText(this@LoupeEditorActivity, "Failed to read reference image", Toast.LENGTH_SHORT).show()
            }
        }
    }

    private suspend fun applyColorMatch(refBmp: Bitmap, refParams: DevelopParams?) {
        val tgt = previewSource ?: return
        val matched = withContext(Dispatchers.Default) {
            AiColorMatchEngine.matchToReference(refBmp, tgt, refParams, currentParams)
        }
        updateParams(matched, commit = true, historyLabel = "AI Color Match")
        rebuildActivePanel()
        Toast.makeText(this, "Matched tone, WB & palette to reference", Toast.LENGTH_SHORT).show()
    }

    private fun showMoreMenu() {
        val cur = asset ?: return
        val items = arrayOf(
            "Save Snapshot / Version…",
            "Restore Saved Version…",
            "History Log (${catalog.getHistorySteps(cur.idLocal).size})",
            "Photo Info & EXIF",
            "Reset All Adjustments"
        )
        AlertDialog.Builder(this)
            .setTitle(cur.displayName)
            .setItems(items) { _, which ->
                when (which) {
                    0 -> {
                        val et = EditText(this).apply { hint = "Version Name (e.g. Warm Cut)" }
                        AlertDialog.Builder(this)
                            .setTitle("Save Version Snapshot")
                            .setView(et)
                            .setNegativeButton("Cancel", null)
                            .setPositiveButton("Save") { _, _ ->
                                val vName = et.text.toString().trim().ifEmpty { "Snapshot" }
                                catalog.saveVersion(cur.idLocal, vName, currentParams)
                                Toast.makeText(this, "Saved version '$vName'", Toast.LENGTH_SHORT).show()
                            }
                            .show()
                    }
                    1 -> {
                        val vers = catalog.getVersions(cur.idLocal)
                        if (vers.isEmpty()) {
                            Toast.makeText(this, "No saved versions yet", Toast.LENGTH_SHORT).show()
                        } else {
                            AlertDialog.Builder(this)
                                .setTitle("Restore Version")
                                .setItems(vers.map { "${it.name} (${it.createdAt})" }.toTypedArray()) { _, idx ->
                                    updateParams(vers[idx].params, commit = true, historyLabel = "Version: ${vers[idx].name}")
                                    rebuildActivePanel()
                                }
                                .show()
                        }
                    }
                    2 -> {
                        val hist = catalog.getHistorySteps(cur.idLocal)
                        AlertDialog.Builder(this)
                            .setTitle("Develop History")
                            .setItems(hist.map { "${it.name} · ${it.timestamp}" }.toTypedArray()) { _, idx ->
                                updateParams(hist[idx].params, commit = true, historyLabel = null)
                                rebuildActivePanel()
                            }
                            .show()
                    }
                    3 -> {
                        val msg = "File: ${cur.displayName}\nResolution: ${cur.width} x ${cur.height}\nRAW: ${cur.isRaw}\nPreset: ${currentParams.appliedPresetName.ifBlank { "Custom" }}"
                        AlertDialog.Builder(this)
                            .setTitle("Photo Metadata")
                            .setMessage(msg)
                            .setPositiveButton("OK", null)
                            .show()
                    }
                    4 -> {
                        updateParams(DevelopParams(), commit = true, historyLabel = "Reset All")
                        rebuildActivePanel()
                    }
                }
            }
            .show()
    }

    private fun launchExportDialog() {
        val cur = asset ?: return
        DarkroomUiFactory.showExportAndWatermarkDialog(
            context = this,
            previewThumb = previewSource?.let {
                KotlinPv2012Engine.render(it, currentParams, draftInteractive = true, applyCrop = true).bitmap
            },
            baseFileName = cur.displayName,
            renderFullBitmap = {
                val fullSrc = MediaStoreImporter.decodeScaledBitmap(this, cur.uriString, 0)
                    ?: return@showExportAndWatermarkDialog null
                val out = KotlinPv2012Engine.render(fullSrc, currentParams, draftInteractive = false, applyCrop = true)
                fullSrc.recycle()
                out.bitmap
            },
            onPickWatermarkGraphic = { onPicked ->
                pendingWatermarkGraphicCallback = onPicked
                pickWatermarkGraphicLauncher.launch(arrayOf("image/png", "image/jpeg", "image/webp"))
            },
            onExportComplete = { }
        )
    }

    override fun onDestroy() {
        super.onDestroy()
        renderJob?.cancel()
        previewSource?.recycle()
    }
}
