package com.moestret.lightraw.ui

import android.Manifest
import android.app.AlertDialog
import android.content.Intent
import android.content.pm.PackageManager
import android.content.res.ColorStateList
import android.graphics.Bitmap
import android.graphics.drawable.GradientDrawable
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.text.Editable
import android.text.TextWatcher
import android.util.LruCache
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.EditText
import android.widget.ImageView
import android.widget.LinearLayout
import android.widget.ProgressBar
import android.widget.TextView
import android.widget.Toast
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat
import androidx.core.widget.ImageViewCompat
import androidx.recyclerview.widget.GridLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.moestret.lightraw.R
import com.moestret.lightraw.data.AppUpdater
import com.moestret.lightraw.data.CatalogDbHelper
import com.moestret.lightraw.data.MediaStoreImporter
import com.moestret.lightraw.databinding.ActivityMainBinding
import com.moestret.lightraw.engine.KotlinPv2012Engine
import com.moestret.lightraw.model.AlbumInfo
import com.moestret.lightraw.model.ColorLabel
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.ExportConfig
import com.moestret.lightraw.model.PhotoAsset
import com.moestret.lightraw.model.PickFlag
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * S01 Library Grid, Albums & Collections, Search/Filter Culling, Sort & S02 Device MediaStore Importer.
 */
class MainActivity : AppCompatActivity() {

    private lateinit var binding: ActivityMainBinding
    private lateinit var catalog: CatalogDbHelper
    private val scope = CoroutineScope(Dispatchers.Main + Job())

    private var allPhotos: List<PhotoAsset> = emptyList()
    private var visiblePhotos: List<PhotoAsset> = emptyList()
    private var deviceItems: List<MediaStoreImporter.DeviceMediaItem> = emptyList()
    private var albumsList: List<AlbumInfo> = emptyList()

    private var navSection = NavSection.PHOTOS
    private var activeAlbumFilter: String? = null
    private var sortMode = SortMode.IMPORT_NEWEST
    private var spanCount = 3
    private var selectMode = false
    private val selectedIds = linkedSetOf<Long>()
    private var activeFilter = FilterKind.ALL
    private var searchQuery = ""

    private val thumbCache = object : LruCache<String, Bitmap>(72) {}
    private var pendingInstallApk: java.io.File? = null
    private var updateCheckInFlight = false

    enum class NavSection { PHOTOS, ALBUMS, DEVICE }

    enum class SortMode(val label: String) {
        IMPORT_NEWEST("Import Date (Newest)"),
        RATING_DESC("Star Rating (5★ First)"),
        NAME_ASC("File Name (A-Z)"),
        RAW_FIRST("RAW / DNG First")
    }

    enum class FilterKind(val label: String) {
        ALL("All"),
        PICKED("Picked ⚑"),
        UNFLAGGED("Unflagged"),
        REJECTED("Rejected ✕"),
        STARS_3("3★+"),
        STARS_5("5★"),
        RAW_ONLY("RAW / DNG"),
        EDITED("Edited"),
        COLOR_RED("Red"),
        COLOR_YELLOW("Yellow"),
        COLOR_GREEN("Green"),
        COLOR_BLUE("Blue"),
        COLOR_PURPLE("Purple")
    }

    private val pickDocumentsLauncher =
        registerForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris: List<Uri> ->
            if (uris.isNotEmpty()) importUris(uris)
        }

    private val permLauncher =
        registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) { result ->
            val ok = result.values.any { it }
            if (navSection == NavSection.DEVICE) {
                if (ok) loadDeviceGallery() else pickDocumentsLauncher.launch(arrayOf("image/*", "*/*"))
            } else {
                pickDocumentsLauncher.launch(arrayOf("image/*", "*/*"))
            }
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        catalog = CatalogDbHelper(this)
        catalog.ensureSeededPresets()

        setupTopBar()
        setupFilters()
        setupAlbumsBar()
        setupBottomNavigation()
        setupBottomActions()
        handleIncomingShareIntent(intent)

        AppUpdater.cleanupUpdateCache(this)
        if (AppUpdater.shouldAutoCheckNow(this)) {
            checkAppUpdates(manual = false)
        }
    }

    override fun onResume() {
        super.onResume()
        pendingInstallApk?.let { apkFile ->
            if (!AppUpdater.needsInstallPermissionPrompt(this) && apkFile.exists()) {
                pendingInstallApk = null
                AppUpdater.launchApkInstaller(this, apkFile)
            }
        }
        thumbCache.evictAll()
        if (navSection == NavSection.DEVICE) {
            loadDeviceGallery()
        } else {
            reloadCatalog()
        }
    }

    private fun setupTopBar() {
        binding.rvPhotos.layoutManager = GridLayoutManager(this, spanCount)
        binding.rvPhotos.adapter = PhotoGridAdapter()

        binding.btnCheckUpdate.setOnClickListener {
            checkAppUpdates(manual = true)
        }

        binding.btnSort.setOnClickListener {
            val labels = SortMode.entries.map {
                if (it == sortMode) "✓ ${it.label}" else it.label
            }.toTypedArray()
            AlertDialog.Builder(this)
                .setTitle("Sort Photos By")
                .setItems(labels) { _, which ->
                    sortMode = SortMode.entries[which]
                    applyFilter()
                }
                .show()
        }

        binding.btnColumns.setOnClickListener {
            spanCount = when (spanCount) {
                2 -> 3
                3 -> 4
                else -> 2
            }
            binding.btnColumns.text = "$spanCount Col"
            (binding.rvPhotos.layoutManager as GridLayoutManager).spanCount = spanCount
            binding.rvPhotos.adapter?.notifyDataSetChanged()
        }

        binding.btnSelectMode.setOnClickListener {
            if (navSection == NavSection.DEVICE) {
                Toast.makeText(this, "Tap any device photo to import & edit", Toast.LENGTH_SHORT).show()
                return@setOnClickListener
            }
            toggleSelectMode(!selectMode)
        }

        binding.etSearch.addTextChangedListener(object : TextWatcher {
            override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) = Unit
            override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) = Unit
            override fun afterTextChanged(s: Editable?) {
                searchQuery = s?.toString()?.trim() ?: ""
                if (navSection == NavSection.DEVICE) {
                    binding.rvPhotos.adapter?.notifyDataSetChanged()
                } else {
                    applyFilter()
                }
            }
        })
    }

    private fun setupFilters() {
        rebuildFilterChips()
    }

    private fun rebuildFilterChips() {
        binding.filterContainer.removeAllViews()
        val options = FilterKind.entries.map { it.label to (it == activeFilter) }
        DarkroomUiFactory.addChipBar(binding.filterContainer, options) { idx ->
            activeFilter = FilterKind.entries[idx]
            rebuildFilterChips()
            applyFilter()
        }
    }

    private fun setupAlbumsBar() {
        binding.btnNewAlbum.setOnClickListener {
            val et = EditText(this).apply { hint = "Album Name (e.g. Wedding Shoot)" }
            AlertDialog.Builder(this)
                .setTitle("Create New Album")
                .setView(et)
                .setNegativeButton("Cancel", null)
                .setPositiveButton("Create") { _, _ ->
                    val name = et.text.toString().trim()
                    if (name.isNotEmpty()) {
                        catalog.createAlbum(name)
                        activeAlbumFilter = name
                        reloadCatalog()
                    }
                }
                .show()
        }
        binding.btnDevicePickSaf.setOnClickListener {
            pickDocumentsLauncher.launch(arrayOf("image/*", "*/*"))
        }
    }

    private fun rebuildAlbumsStrip() {
        binding.albumsContainer.removeAllViews()
        fun addAlbumChip(title: String, count: Int, albumKey: String?, allowDelete: Boolean) {
            val selected = activeAlbumFilter == albumKey
            val chip = Button(this, null, 0, com.google.android.material.R.style.Widget_Material3_Button_TonalButton).apply {
                text = "$title ($count)"
                textSize = 12f
                isAllCaps = false
                setBackgroundResource(if (selected) R.drawable.bg_pill_accent else R.drawable.bg_pill_neutral)
                setTextColor(getColor(if (selected) R.color.lr_on_accent else R.color.lr_text))
                layoutParams = LinearLayout.LayoutParams(
                    LinearLayout.LayoutParams.WRAP_CONTENT,
                    DarkroomUiFactory.dp(this@MainActivity, 40)
                ).apply { marginEnd = DarkroomUiFactory.dp(this@MainActivity, 8) }
                setOnClickListener {
                    activeAlbumFilter = albumKey
                    rebuildAlbumsStrip()
                    applyFilter()
                }
                if (allowDelete && albumKey != null) {
                    setOnLongClickListener {
                        AlertDialog.Builder(this@MainActivity)
                            .setTitle("Delete Album '$albumKey'?")
                            .setMessage("Photos stay safe in All Photos.")
                            .setNegativeButton("Cancel", null)
                            .setPositiveButton("Delete") { _, _ ->
                                catalog.deleteAlbum(albumKey)
                                if (activeAlbumFilter == albumKey) activeAlbumFilter = null
                                reloadCatalog()
                            }
                            .show()
                        true
                    }
                }
            }
            binding.albumsContainer.addView(chip)
        }
        addAlbumChip("All Albums", allPhotos.size, null, allowDelete = false)
        for (a in albumsList) {
            addAlbumChip(a.name, a.photoCount, a.name, allowDelete = !a.isQuickCollection)
        }
    }

    private fun setupBottomNavigation() {
        fun selectNav(target: NavSection) {
            navSection = target
            if (target != NavSection.ALBUMS) activeAlbumFilter = null
            if (selectMode) toggleSelectMode(false)

            val activeCol = getColor(R.color.lr_accent_light)
            val mutedCol = getColor(R.color.lr_text_muted)

            ImageViewCompat.setImageTintList(binding.ivNavPhotos, ColorStateList.valueOf(if (target == NavSection.PHOTOS) activeCol else mutedCol))
            binding.tvNavPhotos.setTextColor(if (target == NavSection.PHOTOS) activeCol else mutedCol)
            ImageViewCompat.setImageTintList(binding.ivNavAlbums, ColorStateList.valueOf(if (target == NavSection.ALBUMS) activeCol else mutedCol))
            binding.tvNavAlbums.setTextColor(if (target == NavSection.ALBUMS) activeCol else mutedCol)
            ImageViewCompat.setImageTintList(binding.ivNavDevice, ColorStateList.valueOf(if (target == NavSection.DEVICE) activeCol else mutedCol))
            binding.tvNavDevice.setTextColor(if (target == NavSection.DEVICE) activeCol else mutedCol)

            binding.filterScroll.visibility = if (target == NavSection.DEVICE) View.GONE else View.VISIBLE
            binding.albumsStrip.visibility = if (target == NavSection.ALBUMS) View.VISIBLE else View.GONE
            binding.deviceStrip.visibility = if (target == NavSection.DEVICE) View.VISIBLE else View.GONE
            binding.quickAddRow.visibility = if (target == NavSection.DEVICE) View.GONE else View.VISIBLE

            binding.tvTopTitle.text = when (target) {
                NavSection.PHOTOS -> "All Photos"
                NavSection.ALBUMS -> activeAlbumFilter ?: "Albums & Collections"
                NavSection.DEVICE -> "Device Storage"
            }

            if (target == NavSection.DEVICE) {
                ensureStoragePermissionAndScan()
            } else {
                reloadCatalog()
            }
        }

        binding.navPhotos.setOnClickListener { selectNav(NavSection.PHOTOS) }
        binding.navAlbums.setOnClickListener { selectNav(NavSection.ALBUMS) }
        binding.navDevice.setOnClickListener { selectNav(NavSection.DEVICE) }
        binding.navCamera.setOnClickListener {
            startActivity(Intent(this, ProCameraActivity::class.java))
        }
    }

    private fun setupBottomActions() {
        val triggerImport = View.OnClickListener { requestGalleryImport() }
        binding.btnImportGallery.setOnClickListener(triggerImport)
        binding.btnEmptyImport.setOnClickListener(triggerImport)

        val triggerSamples = View.OnClickListener {
            scope.launch {
                val created = withContext(Dispatchers.IO) {
                    MediaStoreImporter.generateBuiltInSamples(this@MainActivity, catalog)
                }
                Toast.makeText(this@MainActivity, "Loaded ${created.size} studio samples", Toast.LENGTH_SHORT).show()
                reloadCatalog()
            }
        }
        binding.btnAddSamples.setOnClickListener(triggerSamples)
        binding.btnEmptySample.setOnClickListener(triggerSamples)

        binding.btnBatchRate.setOnClickListener { showBatchRatingDialog() }
        binding.btnBatchPick.setOnClickListener { showBatchPickDialog() }
        binding.btnBatchColor.setOnClickListener { showBatchColorDialog() }
        binding.btnBatchAlbum.setOnClickListener { showBatchAlbumDialog() }
        binding.btnBatchCopy.setOnClickListener {
            val firstId = selectedIds.firstOrNull() ?: return@setOnClickListener
            val photo = allPhotos.firstOrNull { it.idLocal == firstId } ?: return@setOnClickListener
            DarkroomUiFactory.showCopySettingsDialog(this, SessionClipboard.copyMask) { mask ->
                SessionClipboard.copyMask = mask
                SessionClipboard.copiedParams = photo.params.deepCopy()
                Toast.makeText(this, "Settings copied from ${photo.displayName}", Toast.LENGTH_SHORT).show()
            }
        }
        binding.btnBatchPaste.setOnClickListener {
            val copied = SessionClipboard.copiedParams
            if (copied == null) {
                Toast.makeText(this, "Copy settings from a photo first", Toast.LENGTH_SHORT).show()
                return@setOnClickListener
            }
            selectedIds.forEach { pid ->
                val target = allPhotos.firstOrNull { it.idLocal == pid } ?: return@forEach
                val merged = SessionClipboard.copyMask.applyTo(copied, target.params)
                catalog.saveDevelopParams(pid, merged)
            }
            thumbCache.evictAll()
            reloadCatalog()
            Toast.makeText(this, "Pasted settings to ${selectedIds.size} photo(s)", Toast.LENGTH_SHORT).show()
        }
        binding.btnBatchCollage.setOnClickListener {
            if (selectedIds.size < 2) {
                Toast.makeText(this, "Select 2 to 6 photos for Collage", Toast.LENGTH_SHORT).show()
                return@setOnClickListener
            }
            val i = Intent(this, CollageActivity::class.java).apply {
                putExtra("photo_ids", selectedIds.take(6).toLongArray())
            }
            startActivity(i)
        }
        binding.btnBatchExport.setOnClickListener {
            val ids = selectedIds.toList()
            if (ids.isEmpty()) return@setOnClickListener
            scope.launch {
                val count = withContext(Dispatchers.Default) {
                    var ok = 0
                    for (id in ids) {
                        val asset = catalog.getPhotoById(id) ?: continue
                        val rawBmp = MediaStoreImporter.decodeScaledBitmap(this@MainActivity, asset.uriString, 0) ?: continue
                        val rendered = KotlinPv2012Engine.render(rawBmp, asset.params, draftInteractive = false, applyCrop = true)
                        rawBmp.recycle()
                        val wm = SessionClipboard.lastWatermark
                        val stamped = com.moestret.lightraw.engine.WatermarkRenderer.applyWatermarkWithContext(
                            this@MainActivity,
                            rendered.bitmap,
                            wm
                        )
                        if (stamped != rendered.bitmap) rendered.bitmap.recycle()
                        val uri = MediaStoreImporter.exportToGallery(
                            this@MainActivity,
                            stamped,
                            asset.displayName,
                            ExportConfig(watermark = wm)
                        )
                        stamped.recycle()
                        if (uri != null) ok++
                    }
                    ok
                }
                Toast.makeText(this@MainActivity, "Exported $count photo(s) to Pictures/LightRaw", Toast.LENGTH_LONG).show()
            }
        }
        binding.btnBatchDelete.setOnClickListener {
            AlertDialog.Builder(this)
                .setTitle("Remove from LightRaw Catalog?")
                .setMessage("Removes ${selectedIds.size} item(s) from catalog (original gallery files remain safe).")
                .setNegativeButton("Cancel", null)
                .setPositiveButton("Remove") { _, _ ->
                    selectedIds.forEach { catalog.deletePhoto(it) }
                    toggleSelectMode(false)
                    reloadCatalog()
                }
                .show()
        }
    }

    private fun showBatchRatingDialog() {
        val labels = arrayOf("Clear Stars (0)", "★", "★★", "★★★", "★★★★", "★★★★★")
        AlertDialog.Builder(this)
            .setTitle("Set Rating for ${selectedIds.size} photo(s)")
            .setItems(labels) { _, which ->
                selectedIds.forEach { catalog.updateRating(it, which) }
                reloadCatalog()
            }
            .show()
    }

    private fun showBatchPickDialog() {
        val labels = arrayOf("Picked ⚑", "Unflagged", "Rejected ✕")
        val flags = arrayOf(PickFlag.PICKED, PickFlag.UNFLAGGED, PickFlag.REJECTED)
        AlertDialog.Builder(this)
            .setTitle("Set Flag for ${selectedIds.size} photo(s)")
            .setItems(labels) { _, which ->
                selectedIds.forEach { catalog.updatePickFlag(it, flags[which]) }
                reloadCatalog()
            }
            .show()
    }

    private fun showBatchColorDialog() {
        val cols = ColorLabel.entries
        val names = cols.map { if (it == ColorLabel.NONE) "No Color Label" else "${it.code} ●" }.toTypedArray()
        AlertDialog.Builder(this)
            .setTitle("Set Color Label (${selectedIds.size} photo(s))")
            .setItems(names) { _, which ->
                selectedIds.forEach { catalog.updateColorLabel(it, cols[which]) }
                reloadCatalog()
            }
            .show()
    }

    private fun showBatchAlbumDialog() {
        val albums = catalog.getAllAlbums().map { it.name } + "All Photos"
        AlertDialog.Builder(this)
            .setTitle("Add ${selectedIds.size} photo(s) to Album")
            .setItems(albums.toTypedArray()) { _, which ->
                val target = albums[which]
                selectedIds.forEach { catalog.updateAlbum(it, target) }
                Toast.makeText(this, "Moved to $target", Toast.LENGTH_SHORT).show()
                reloadCatalog()
            }
            .show()
    }

    private fun toggleSelectMode(enable: Boolean) {
        selectMode = enable
        if (!enable) selectedIds.clear()
        binding.btnSelectMode.text = if (enable) "Done (${selectedIds.size})" else "Select"
        binding.dockNormal.visibility = if (enable) View.GONE else View.VISIBLE
        binding.dockBatch.visibility = if (enable) View.VISIBLE else View.GONE
        binding.quickAddRow.visibility = if (enable || navSection == NavSection.DEVICE) View.GONE else View.VISIBLE
        binding.rvPhotos.adapter?.notifyDataSetChanged()
    }

    private fun requiredMediaPerms(): Array<String> =
        if (Build.VERSION.SDK_INT >= 33) arrayOf(Manifest.permission.READ_MEDIA_IMAGES)
        else arrayOf(Manifest.permission.READ_EXTERNAL_STORAGE)

    private fun ensureStoragePermissionAndScan() {
        val perms = requiredMediaPerms()
        val granted = perms.all {
            ContextCompat.checkSelfPermission(this, it) == PackageManager.PERMISSION_GRANTED
        }
        if (granted) loadDeviceGallery() else permLauncher.launch(perms)
    }

    private fun loadDeviceGallery() {
        scope.launch {
            deviceItems = withContext(Dispatchers.IO) {
                MediaStoreImporter.scanDeviceMediaStore(this@MainActivity, limit = 100)
            }
            binding.tvCatalogCount.text = "${deviceItems.size} device photos found"
            binding.tvDeviceStatus.text = "Device Storage · ${deviceItems.size} photos (Tap to Edit)"
            binding.emptyState.visibility = if (deviceItems.isEmpty()) View.VISIBLE else View.GONE
            binding.rvPhotos.adapter?.notifyDataSetChanged()
        }
    }

    private fun requestGalleryImport() {
        val perms = requiredMediaPerms()
        val granted = perms.all {
            ContextCompat.checkSelfPermission(this, it) == PackageManager.PERMISSION_GRANTED
        }
        if (granted) {
            pickDocumentsLauncher.launch(arrayOf("image/*", "*/*"))
        } else {
            permLauncher.launch(perms)
        }
    }

    private fun importUris(uris: List<Uri>) {
        scope.launch {
            val count = withContext(Dispatchers.IO) {
                var added = 0
                for (u in uris) {
                    if (MediaStoreImporter.importUri(this@MainActivity, u, catalog) != null) added++
                }
                added
            }
            Toast.makeText(this@MainActivity, "Imported $count photo(s)", Toast.LENGTH_SHORT).show()
            reloadCatalog()
        }
    }

    private fun handleIncomingShareIntent(i: Intent?) {
        if (i == null) return
        val uris = mutableListOf<Uri>()
        when (i.action) {
            Intent.ACTION_SEND, Intent.ACTION_VIEW -> {
                val u = if (Build.VERSION.SDK_INT >= 33) {
                    i.getParcelableExtra(Intent.EXTRA_STREAM, Uri::class.java) ?: i.data
                } else {
                    @Suppress("DEPRECATION")
                    i.getParcelableExtra(Intent.EXTRA_STREAM) ?: i.data
                }
                if (u != null) uris.add(u)
            }
            Intent.ACTION_SEND_MULTIPLE -> {
                val list = if (Build.VERSION.SDK_INT >= 33) {
                    i.getParcelableArrayListExtra(Intent.EXTRA_STREAM, Uri::class.java)
                } else {
                    @Suppress("DEPRECATION")
                    i.getParcelableArrayListExtra(Intent.EXTRA_STREAM)
                }
                if (list != null) uris.addAll(list)
            }
        }
        if (uris.isNotEmpty()) importUris(uris)
    }

    private fun reloadCatalog() {
        scope.launch {
            val (photos, albums) = withContext(Dispatchers.IO) {
                catalog.getAllPhotos() to catalog.getAllAlbums()
            }
            allPhotos = photos
            albumsList = albums
            if (navSection == NavSection.ALBUMS) {
                rebuildAlbumsStrip()
                binding.tvTopTitle.text = activeAlbumFilter ?: "Albums & Collections"
            }
            applyFilter()
        }
    }

    private fun applyFilter() {
        if (navSection == NavSection.DEVICE) return
        val q = searchQuery.lowercase()
        val filtered = allPhotos.filter { item ->
            val matchQ = q.isEmpty() ||
                item.displayName.lowercase().contains(q) ||
                item.albumName.lowercase().contains(q)
            val matchAlbum = activeAlbumFilter == null || item.albumName == activeAlbumFilter
            val matchCat = when (activeFilter) {
                FilterKind.ALL -> true
                FilterKind.PICKED -> item.pickFlag == PickFlag.PICKED
                FilterKind.UNFLAGGED -> item.pickFlag == PickFlag.UNFLAGGED
                FilterKind.REJECTED -> item.pickFlag == PickFlag.REJECTED
                FilterKind.STARS_3 -> item.rating >= 3
                FilterKind.STARS_5 -> item.rating >= 5
                FilterKind.RAW_ONLY -> item.isRaw
                FilterKind.EDITED -> item.params != DevelopParams()
                FilterKind.COLOR_RED -> item.colorLabel == ColorLabel.RED
                FilterKind.COLOR_YELLOW -> item.colorLabel == ColorLabel.YELLOW
                FilterKind.COLOR_GREEN -> item.colorLabel == ColorLabel.GREEN
                FilterKind.COLOR_BLUE -> item.colorLabel == ColorLabel.BLUE
                FilterKind.COLOR_PURPLE -> item.colorLabel == ColorLabel.PURPLE
            }
            matchQ && matchAlbum && matchCat
        }
        visiblePhotos = when (sortMode) {
            SortMode.IMPORT_NEWEST -> filtered
            SortMode.RATING_DESC -> filtered.sortedByDescending { it.rating }
            SortMode.NAME_ASC -> filtered.sortedBy { it.displayName.lowercase() }
            SortMode.RAW_FIRST -> filtered.sortedByDescending { it.isRaw }
        }
        binding.tvCatalogCount.text = "${visiblePhotos.size} of ${allPhotos.size} photos · .lrcat"
        binding.emptyState.visibility = if (visiblePhotos.isEmpty()) View.VISIBLE else View.GONE
        binding.rvPhotos.adapter?.notifyDataSetChanged()
    }

    inner class PhotoGridAdapter : RecyclerView.Adapter<PhotoGridAdapter.VH>() {

        inner class VH(view: View) : RecyclerView.ViewHolder(view) {
            val ivThumb: ImageView = view.findViewById(R.id.ivThumb)
            val vSelectOverlay: View = view.findViewById(R.id.vSelectOverlay)
            val tvBadgeRaw: TextView = view.findViewById(R.id.tvBadgeRaw)
            val vColorLabelDot: View = view.findViewById(R.id.vColorLabelDot)
            val tvBadgeFlag: TextView = view.findViewById(R.id.tvBadgeFlag)
            val tvBadgeStars: TextView = view.findViewById(R.id.tvBadgeStars)
            val tvBadgeEdited: TextView = view.findViewById(R.id.tvBadgeEdited)
        }

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): VH {
            val v = LayoutInflater.from(parent.context).inflate(R.layout.item_photo_grid, parent, false)
            return VH(v)
        }

        private fun filteredDeviceItems(): List<MediaStoreImporter.DeviceMediaItem> {
            val q = searchQuery.lowercase()
            return if (q.isEmpty()) deviceItems else deviceItems.filter { it.displayName.lowercase().contains(q) }
        }

        override fun getItemCount(): Int =
            if (navSection == NavSection.DEVICE) filteredDeviceItems().size else visiblePhotos.size

        override fun onBindViewHolder(holder: VH, position: Int) {
            val cellSize = (resources.displayMetrics.widthPixels - 20) / spanCount
            holder.ivThumb.layoutParams.height = cellSize
            holder.vSelectOverlay.layoutParams.height = cellSize

            if (navSection == NavSection.DEVICE) {
                val dev = filteredDeviceItems()[position]
                holder.tvBadgeRaw.visibility = if (dev.isRaw) View.VISIBLE else View.GONE
                holder.tvBadgeEdited.visibility = View.GONE
                holder.tvBadgeFlag.visibility = View.GONE
                holder.tvBadgeStars.visibility = View.GONE
                holder.vColorLabelDot.visibility = View.GONE
                holder.vSelectOverlay.visibility = View.GONE

                val cacheKey = "dev_${dev.uri}"
                val cached = thumbCache.get(cacheKey)
                holder.ivThumb.tag = cacheKey
                if (cached != null) {
                    holder.ivThumb.setImageBitmap(cached)
                } else {
                    holder.ivThumb.setImageBitmap(null)
                    scope.launch {
                        val bmp = withContext(Dispatchers.IO) {
                            MediaStoreImporter.decodeScaledBitmap(this@MainActivity, dev.uri.toString(), 300)
                        }
                        if (bmp != null) {
                            thumbCache.put(cacheKey, bmp)
                            if (holder.ivThumb.tag == cacheKey) holder.ivThumb.setImageBitmap(bmp)
                        }
                    }
                }
                holder.itemView.setOnClickListener {
                    scope.launch {
                        val imported = withContext(Dispatchers.IO) {
                            MediaStoreImporter.importUri(this@MainActivity, dev.uri, catalog)
                        }
                        if (imported != null) {
                            startActivity(
                                Intent(this@MainActivity, LoupeEditorActivity::class.java).apply {
                                    putExtra("photo_id", imported.idLocal)
                                }
                            )
                        }
                    }
                }
                holder.itemView.setOnLongClickListener(null)
                return
            }

            val item = visiblePhotos[position]
            holder.tvBadgeRaw.visibility = if (item.isRaw) View.VISIBLE else View.GONE
            holder.tvBadgeEdited.visibility = if (item.params != DevelopParams()) View.VISIBLE else View.GONE
            holder.tvBadgeFlag.visibility = if (item.pickFlag == PickFlag.UNFLAGGED) View.GONE else View.VISIBLE
            holder.tvBadgeFlag.text = if (item.pickFlag == PickFlag.PICKED) "⚑" else "✕"
            holder.tvBadgeFlag.setTextColor(
                getColor(if (item.pickFlag == PickFlag.PICKED) R.color.lr_success else R.color.lr_danger)
            )
            if (item.colorLabel != ColorLabel.NONE) {
                holder.vColorLabelDot.visibility = View.VISIBLE
                holder.vColorLabelDot.background = GradientDrawable().apply {
                    shape = GradientDrawable.OVAL
                    setColor(item.colorLabel.colorHex)
                    setStroke(2, 0xFF080808.toInt())
                }
            } else {
                holder.vColorLabelDot.visibility = View.GONE
            }
            holder.tvBadgeStars.visibility = if (item.rating > 0) View.VISIBLE else View.GONE
            holder.tvBadgeStars.text = "★".repeat(item.rating.coerceIn(0, 5))

            val isSelected = selectMode && item.idLocal in selectedIds
            holder.vSelectOverlay.visibility = if (isSelected) View.VISIBLE else View.GONE

            val cacheKey = "${item.idLocal}_${item.params.hashCode()}"
            val cached = thumbCache.get(cacheKey)
            holder.ivThumb.tag = cacheKey
            if (cached != null) {
                holder.ivThumb.setImageBitmap(cached)
            } else {
                holder.ivThumb.setImageBitmap(null)
                scope.launch {
                    val bmp = withContext(Dispatchers.Default) {
                        val decoded = MediaStoreImporter.decodeScaledBitmap(this@MainActivity, item.uriString, 320)
                            ?: return@withContext null
                        if (item.params == DevelopParams()) {
                            decoded
                        } else {
                            val r = KotlinPv2012Engine.render(decoded, item.params, draftInteractive = true, applyCrop = true)
                            decoded.recycle()
                            r.bitmap
                        }
                    }
                    if (bmp != null) {
                        thumbCache.put(cacheKey, bmp)
                        if (holder.ivThumb.tag == cacheKey) {
                            holder.ivThumb.setImageBitmap(bmp)
                        }
                    }
                }
            }

            holder.itemView.setOnClickListener {
                if (selectMode) {
                    if (item.idLocal in selectedIds) selectedIds.remove(item.idLocal) else selectedIds.add(item.idLocal)
                    binding.btnSelectMode.text = "Done (${selectedIds.size})"
                    notifyItemChanged(holder.bindingAdapterPosition)
                } else {
                    startActivity(
                        Intent(this@MainActivity, LoupeEditorActivity::class.java).apply {
                            putExtra("photo_id", item.idLocal)
                            putExtra("photo_ids", visiblePhotos.map { it.idLocal }.toLongArray())
                        }
                    )
                }
            }

            holder.itemView.setOnLongClickListener {
                if (!selectMode) toggleSelectMode(true)
                selectedIds.add(item.idLocal)
                binding.btnSelectMode.text = "Done (${selectedIds.size})"
                notifyItemChanged(holder.bindingAdapterPosition)
                true
            }
        }
    }

    private fun checkAppUpdates(manual: Boolean) {
        if (updateCheckInFlight) return
        updateCheckInFlight = true
        if (manual) {
            Toast.makeText(this, "Checking for updates...", Toast.LENGTH_SHORT).show()
        }
        scope.launch {
            val result = AppUpdater.fetchLatestRelease(this@MainActivity)
            updateCheckInFlight = false
            if (isFinishing || isDestroyed) return@launch

            when (result) {
                is AppUpdater.CheckResult.UpdateAvailable -> {
                    ImageViewCompat.setImageTintList(
                        binding.btnCheckUpdate,
                        ColorStateList.valueOf(ContextCompat.getColor(this@MainActivity, R.color.lr_accent_light))
                    )
                    if (manual || !AppUpdater.isTagSkipped(this@MainActivity, result.release.tagName)) {
                        showUpdateAvailableDialog(result.currentVersion, result.release)
                    }
                }
                is AppUpdater.CheckResult.UpToDate -> {
                    if (manual) {
                        AlertDialog.Builder(this@MainActivity)
                            .setTitle("LightRaw Is Up To Date")
                            .setMessage("Installed version: v${result.currentVersion}\nLatest GitHub release: ${result.latestTag}")
                            .setPositiveButton("OK", null)
                            .show()
                    }
                }
                is AppUpdater.CheckResult.Failed -> {
                    if (manual) {
                        AlertDialog.Builder(this@MainActivity)
                            .setTitle("Update Check Failed")
                            .setMessage("Could not reach GitHub Releases:\n${result.reason}")
                            .setNegativeButton("Close", null)
                            .setPositiveButton("Open Releases") { _, _ ->
                                startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(AppUpdater.GITHUB_RELEASES_PAGE)))
                            }
                            .show()
                    }
                }
            }
        }
    }

    private fun showUpdateAvailableDialog(currentVer: String, release: AppUpdater.ReleaseInfo) {
        val notes = release.releaseNotes.ifBlank { "Bug fixes, real-time performance & enhancements." }
        val msg = "Installed: v$currentVer\nLatest: ${release.tagName}\n\n$notes"
        val builder = AlertDialog.Builder(this)
            .setTitle("Update Available: ${release.title}")
            .setMessage(msg)
            .setNegativeButton("Skip Version") { _, _ ->
                AppUpdater.skipTag(this, release.tagName)
            }
            .setNeutralButton("Later", null)

        val asset = release.asset
        if (asset != null) {
            builder.setPositiveButton("Update Now") { _, _ ->
                startApkDownloadAndInstall(asset)
            }
        } else {
            builder.setPositiveButton("Download Page") { _, _ ->
                startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(release.htmlUrl)))
            }
        }
        builder.show()
    }

    private fun startApkDownloadAndInstall(asset: AppUpdater.ReleaseAsset) {
        val pad = DarkroomUiFactory.dp(this, 18)
        val container = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(pad, pad, pad, pad)
        }
        val tvStatus = TextView(this).apply {
            text = "Downloading ${asset.name}..."
            textSize = 13f
            setTextColor(ContextCompat.getColor(this@MainActivity, R.color.lr_text))
        }
        val progressBar = ProgressBar(this, null, android.R.attr.progressBarStyleHorizontal).apply {
            max = 100
            progress = 0
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.WRAP_CONTENT
            ).apply { topMargin = DarkroomUiFactory.dp(this@MainActivity, 12) }
        }
        container.addView(tvStatus)
        container.addView(progressBar)

        var downloadJob: Job? = null
        val dialog = AlertDialog.Builder(this)
            .setTitle("Downloading Update")
            .setView(container)
            .setCancelable(false)
            .setNegativeButton("Cancel") { _, _ -> downloadJob?.cancel() }
            .create()
        dialog.show()

        downloadJob = scope.launch {
            val downloadRes = AppUpdater.downloadApk(this@MainActivity, asset) { downloaded, total ->
                val mbDone = downloaded / (1024f * 1024f)
                if (total > 0L) {
                    val pct = ((downloaded * 100L) / total).toInt().coerceIn(0, 100)
                    val mbTotal = total / (1024f * 1024f)
                    progressBar.isIndeterminate = false
                    progressBar.progress = pct
                    tvStatus.text = "Downloading ${asset.name}: %.1f / %.1f MB (%d%%)".format(mbDone, mbTotal, pct)
                } else {
                    progressBar.isIndeterminate = true
                    tvStatus.text = "Downloading ${asset.name}: %.1f MB".format(mbDone)
                }
            }
            if (dialog.isShowing) dialog.dismiss()
            if (isFinishing || isDestroyed) return@launch

            downloadRes.onSuccess { apkFile ->
                proceedWithDownloadedApk(apkFile)
            }.onFailure { err ->
                AlertDialog.Builder(this@MainActivity)
                    .setTitle("Download Failed")
                    .setMessage(err.message ?: "Failed to download update package.")
                    .setNegativeButton("Close", null)
                    .setPositiveButton("Open GitHub") { _, _ ->
                        startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(AppUpdater.GITHUB_RELEASES_PAGE)))
                    }
                    .show()
            }
        }
    }

    private fun proceedWithDownloadedApk(apkFile: java.io.File) {
        val sigMatch = AppUpdater.checkSignatureMatchesInstalled(this, apkFile)
        if (sigMatch == false) {
            AlertDialog.Builder(this)
                .setTitle("Signing Key Notice")
                .setMessage(
                    "The downloaded APK uses a different signing key than the currently installed build " +
                        "(common when switching from a local USB/Android Studio debug build to a GitHub Release build).\n\n" +
                        "Android only allows in-place updates without uninstalling when both APKs share the same signing key."
                )
                .setNegativeButton("Cancel", null)
                .setNeutralButton("Open Releases") { _, _ ->
                    startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(AppUpdater.GITHUB_RELEASES_PAGE)))
                }
                .setPositiveButton("Try Installer Anyway") { _, _ ->
                    requestPermissionOrInstall(apkFile)
                }
                .show()
            return
        }
        requestPermissionOrInstall(apkFile)
    }

    private fun requestPermissionOrInstall(apkFile: java.io.File) {
        if (AppUpdater.needsInstallPermissionPrompt(this)) {
            pendingInstallApk = apkFile
            AlertDialog.Builder(this)
                .setTitle("Allow App Installs")
                .setMessage(
                    "Grant LightRaw permission to install updates so this version upgrades directly in place without uninstalling your catalog or photos."
                )
                .setNegativeButton("Cancel") { _, _ -> pendingInstallApk = null }
                .setPositiveButton("Open Settings") { _, _ ->
                    AppUpdater.launchInstallPermissionSettings(this)
                }
                .show()
        } else {
            AppUpdater.launchApkInstaller(this, apkFile)
        }
    }
}
