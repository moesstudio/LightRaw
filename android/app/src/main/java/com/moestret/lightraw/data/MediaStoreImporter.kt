package com.moestret.lightraw.data

import android.content.ContentResolver
import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.LinearGradient
import android.graphics.Matrix
import android.graphics.Paint
import android.graphics.Shader
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import android.provider.OpenableColumns
import androidx.exifinterface.media.ExifInterface
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.ExportConfig
import com.moestret.lightraw.model.ExportFormat
import com.moestret.lightraw.model.PhotoAsset
import java.io.File
import java.io.FileOutputStream
import kotlin.math.max
import kotlin.math.roundToInt

/**
 * Handles device gallery import (`content://` URIs, RAW DNG/ARW/CR3/NEF extensions),
 * orientation-aware decoding, full-resolution export to `Pictures/LightRaw`, and built-in studio samples.
 */
object MediaStoreImporter {

    private val RAW_EXTENSIONS = setOf(
        "dng", "arw", "cr2", "cr3", "nef", "raf", "orf", "rw2", "pef", "srw", "3fr", "iiq"
    )

    fun importUri(context: Context, uri: Uri, catalog: CatalogDbHelper): PhotoAsset? {
        try {
            context.contentResolver.takePersistableUriPermission(
                uri,
                Intent.FLAG_GRANT_READ_URI_PERMISSION
            )
        } catch (_: SecurityException) {
            // Transient picker grants can still be copied into app storage if needed
        }

        val displayName = resolveDisplayName(context.contentResolver, uri) ?: "IMG_${System.currentTimeMillis()}.jpg"
        val ext = displayName.substringAfterLast('.', "").lowercase()
        val isRaw = ext in RAW_EXTENSIONS

        // Copy safely to internal app mirror only when transient permission might expire, OR read bounds first
        val targetUriString = persistReadableCopyIfNeeded(context, uri, displayName)
        val (w, h, orient) = readBounds(context.contentResolver, Uri.parse(targetUriString))

        val asset = PhotoAsset(
            uriString = targetUriString,
            displayName = displayName,
            width = w,
            height = h,
            orientation = orient,
            isRaw = isRaw,
            params = DevelopParams()
        )
        return catalog.upsertPhoto(asset)
    }

    /** Copies a picked watermark image (PNG/JPG/WebP) into private app storage so permissions never expire. */
    fun persistWatermarkGraphic(context: Context, uri: Uri): Pair<String, String>? {
        val displayName = resolveDisplayName(context.contentResolver, uri) ?: "watermark_logo.png"
        val copied = copyUriToPrivateFile(context, uri, "watermarks", displayName) ?: return null
        return copied to displayName
    }

    private fun persistReadableCopyIfNeeded(context: Context, uri: Uri, name: String): String {
        val hasPersistent = context.contentResolver.persistedUriPermissions.any {
            it.uri == uri && it.isReadPermission
        }
        if (hasPersistent || uri.scheme == "file") return uri.toString()
        return copyUriToPrivateFile(context, uri, "imported_photos", name) ?: uri.toString()
    }

    private fun copyUriToPrivateFile(context: Context, uri: Uri, folder: String, name: String): String? {
        val dir = File(context.filesDir, folder).apply { mkdirs() }
        val safeName = name.replace(Regex("[^a-zA-Z0-9._-]"), "_")
        val dest = File(dir, "${System.currentTimeMillis()}_$safeName")
        return runCatching {
            context.contentResolver.openInputStream(uri)?.use { input ->
                FileOutputStream(dest).use { output -> input.copyTo(output) }
                Uri.fromFile(dest).toString()
            }
        }.getOrNull()
    }

    private fun resolveDisplayName(cr: ContentResolver, uri: Uri): String? {
        if (uri.scheme == "file") return File(uri.path ?: "").name
        cr.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { c ->
            if (c.moveToFirst()) return c.getString(0)
        }
        return uri.lastPathSegment
    }

    private fun readBounds(cr: ContentResolver, uri: Uri): Triple<Int, Int, Int> {
        var w = 0
        var h = 0
        var orientation = 1
        cr.openInputStream(uri)?.use { stream ->
            val opts = BitmapFactory.Options().apply { inJustDecodeBounds = true }
            BitmapFactory.decodeStream(stream, null, opts)
            w = opts.outWidth.coerceAtLeast(0)
            h = opts.outHeight.coerceAtLeast(0)
        }
        cr.openInputStream(uri)?.use { stream ->
            orientation = runCatching {
                ExifInterface(stream).getAttributeInt(
                    ExifInterface.TAG_ORIENTATION,
                    ExifInterface.ORIENTATION_NORMAL
                )
            }.getOrDefault(ExifInterface.ORIENTATION_NORMAL)
        }
        return Triple(w, h, orientation)
    }

    fun decodeScaledBitmap(context: Context, uriString: String, maxEdgePx: Int): Bitmap? {
        val uri = Uri.parse(uriString)
        val cr = context.contentResolver
        val (rawW, rawH, exifOrient) = readBounds(cr, uri)

        val opts = BitmapFactory.Options().apply {
            inPreferredConfig = Bitmap.Config.ARGB_8888
            if (rawW > 0 && rawH > 0 && maxEdgePx > 0) {
                var sample = 1
                var cur = max(rawW, rawH)
                while (cur / (sample * 2) >= maxEdgePx) sample *= 2
                inSampleSize = sample
            }
        }
        val decoded = cr.openInputStream(uri)?.use { stream ->
            BitmapFactory.decodeStream(stream, null, opts)
        } ?: return null

        val rotated = applyExifRotation(decoded, exifOrient)
        val maxSide = max(rotated.width, rotated.height)
        if (maxEdgePx in 1 until maxSide) {
            val scale = maxEdgePx.toFloat() / maxSide
            val targetW = (rotated.width * scale).roundToInt().coerceAtLeast(16)
            val targetH = (rotated.height * scale).roundToInt().coerceAtLeast(16)
            val scaled = Bitmap.createScaledBitmap(rotated, targetW, targetH, true)
            if (scaled != rotated) rotated.recycle()
            return scaled
        }
        return rotated
    }

    private fun applyExifRotation(bmp: Bitmap, exifOrientation: Int): Bitmap {
        val mat = Matrix()
        when (exifOrientation) {
            ExifInterface.ORIENTATION_ROTATE_90 -> mat.postRotate(90f)
            ExifInterface.ORIENTATION_ROTATE_180 -> mat.postRotate(180f)
            ExifInterface.ORIENTATION_ROTATE_270 -> mat.postRotate(270f)
            ExifInterface.ORIENTATION_FLIP_HORIZONTAL -> mat.postScale(-1f, 1f)
            ExifInterface.ORIENTATION_FLIP_VERTICAL -> mat.postScale(1f, -1f)
            else -> return bmp
        }
        val out = Bitmap.createBitmap(bmp, 0, 0, bmp.width, bmp.height, mat, true)
        if (out != bmp) bmp.recycle()
        return out
    }

    fun exportToGallery(
        context: Context,
        bitmap: Bitmap,
        baseFileName: String,
        config: ExportConfig
    ): Uri? {
        val cleanBase = baseFileName.substringBeforeLast('.')
        val outName = "${cleanBase}_LightRaw.${config.format.ext}"
        val compressFmt = when (config.format) {
            ExportFormat.JPEG -> Bitmap.CompressFormat.JPEG
            ExportFormat.PNG -> Bitmap.CompressFormat.PNG
            ExportFormat.WEBP -> if (Build.VERSION.SDK_INT >= 30) Bitmap.CompressFormat.WEBP_LOSSY else Bitmap.CompressFormat.WEBP
        }

        val values = ContentValues().apply {
            put(MediaStore.Images.Media.DISPLAY_NAME, outName)
            put(MediaStore.Images.Media.MIME_TYPE, config.format.mime)
            if (Build.VERSION.SDK_INT >= 29) {
                put(MediaStore.Images.Media.RELATIVE_PATH, "${Environment.DIRECTORY_PICTURES}/LightRaw")
                put(MediaStore.Images.Media.IS_PENDING, 1)
            }
        }
        val resolver = context.contentResolver
        val uri = resolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values) ?: return null
        resolver.openOutputStream(uri)?.use { stream ->
            bitmap.compress(compressFmt, config.quality.coerceIn(10, 100), stream)
        }
        if (Build.VERSION.SDK_INT >= 29) {
            values.clear()
            values.put(MediaStore.Images.Media.IS_PENDING, 0)
            resolver.update(uri, values, null, null)
        }
        return uri
    }

    /**
     * Generates two high-contrast photographic studio sample files locally when users test
     * immediately on an empty gallery/emulator without existing camera roll photos.
     */
    fun generateBuiltInSamples(context: Context, catalog: CatalogDbHelper): List<PhotoAsset> {
        val sampleDir = File(context.filesDir, "samples").apply { mkdirs() }
        val specs = listOf(
            Triple("Studio_Portrait_Sample.jpg", false, true),
            Triple("Alpine_Sunrise_RAW.dng.jpg", true, false)
        )
        val inserted = mutableListOf<PhotoAsset>()
        for ((idx, spec) in specs.withIndex()) {
            val file = File(sampleDir, spec.first)
            if (!file.exists()) {
                val bmp = Bitmap.createBitmap(1200, 1500, Bitmap.Config.ARGB_8888)
                val canvas = Canvas(bmp)
                val p = Paint(Paint.ANTI_ALIAS_FLAG)
                if (spec.third) {
                    // Studio portrait card: cobalt backdrop + warm skin-tone silhouette + highlight key light
                    p.shader = LinearGradient(0f, 0f, 0f, 1500f, 0xFF1D4ED8.toInt(), 0xFF1E3A8A.toInt(), Shader.TileMode.CLAMP)
                    canvas.drawRect(0f, 0f, 1200f, 1500f, p)
                    p.shader = null
                    p.color = 0xFFD99B72.toInt()
                    canvas.drawCircle(600f, 540f, 185f, p)
                    p.color = 0xFFF4F6FA.toInt()
                    canvas.drawRoundRect(340f, 760f, 860f, 1480f, 120f, 120f, p)
                } else {
                    // Alpine golden-hour landscape gradient with mountain ridges
                    p.shader = LinearGradient(0f, 0f, 0f, 920f, 0xFF1E293B.toInt(), 0xFFF59E0B.toInt(), Shader.TileMode.CLAMP)
                    canvas.drawRect(0f, 0f, 1200f, 920f, p)
                    p.shader = null
                    p.color = 0xFF334155.toInt()
                    val path = android.graphics.Path().apply {
                        moveTo(0f, 920f); lineTo(280f, 640f); lineTo(560f, 840f)
                        lineTo(860f, 560f); lineTo(1200f, 880f); lineTo(1200f, 1500f); lineTo(0f, 1500f); close()
                    }
                    canvas.drawPath(path, p)
                    p.color = 0xFF0F172A.toInt()
                    canvas.drawRect(0f, 1080f, 1200f, 1500f, p)
                }
                FileOutputStream(file).use { out -> bmp.compress(Bitmap.CompressFormat.JPEG, 92, out) }
                bmp.recycle()
            }
            val asset = PhotoAsset(
                uriString = Uri.fromFile(file).toString(),
                displayName = if (idx == 0) "Studio Portrait Sample" else "Alpine Sunrise (RAW Sim)",
                width = 1200,
                height = 1500,
                isRaw = spec.second,
                rating = if (idx == 0) 4 else 5,
                albumName = "Sample Studio"
            )
            catalog.upsertPhoto(asset)?.let { inserted.add(it) }
        }
        return inserted
    }

    data class DeviceMediaItem(
        val uri: Uri,
        val displayName: String,
        val width: Int,
        val height: Int,
        val isRaw: Boolean
    )

    fun scanDeviceMediaStore(context: Context, limit: Int = 80): List<DeviceMediaItem> {
        val out = mutableListOf<DeviceMediaItem>()
        val proj = arrayOf(
            MediaStore.Images.Media._ID,
            MediaStore.Images.Media.DISPLAY_NAME,
            MediaStore.Images.Media.WIDTH,
            MediaStore.Images.Media.HEIGHT
        )
        runCatching {
            context.contentResolver.query(
                MediaStore.Images.Media.EXTERNAL_CONTENT_URI,
                proj,
                null,
                null,
                "${MediaStore.Images.Media.DATE_MODIFIED} DESC"
            )?.use { c ->
                val idCol = c.getColumnIndexOrThrow(MediaStore.Images.Media._ID)
                val nameCol = c.getColumnIndexOrThrow(MediaStore.Images.Media.DISPLAY_NAME)
                val wCol = c.getColumnIndexOrThrow(MediaStore.Images.Media.WIDTH)
                val hCol = c.getColumnIndexOrThrow(MediaStore.Images.Media.HEIGHT)
                while (c.moveToNext() && out.size < limit) {
                    val id = c.getLong(idCol)
                    val name = c.getString(nameCol) ?: "IMG_$id"
                    val uri = android.content.ContentUris.withAppendedId(
                        MediaStore.Images.Media.EXTERNAL_CONTENT_URI,
                        id
                    )
                    val ext = name.substringAfterLast('.', "").lowercase()
                    out.add(
                        DeviceMediaItem(
                            uri = uri,
                            displayName = name,
                            width = c.getInt(wCol),
                            height = c.getInt(hCol),
                            isRaw = ext in RAW_EXTENSIONS
                        )
                    )
                }
            }
        }
        return out
    }
}
