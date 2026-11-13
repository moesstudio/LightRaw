package com.moestret.lightraw.data

import android.content.ContentValues
import android.content.Context
import android.database.sqlite.SQLiteDatabase
import android.database.sqlite.SQLiteOpenHelper
import com.moestret.lightraw.model.AlbumInfo
import com.moestret.lightraw.model.ColorLabel
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.DevelopPreset
import com.moestret.lightraw.model.HistoryEntry
import com.moestret.lightraw.model.PhotoAsset
import com.moestret.lightraw.model.PhotoVersion
import com.moestret.lightraw.model.PickFlag
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.UUID

/**
 * Local-first SQLite Catalog mirroring LightRaw Desktop's `LightRawCatalog.lrcat` schema.
 */
class CatalogDbHelper(context: Context) :
    SQLiteOpenHelper(context, "LightRawCatalog.lrcat", null, 3) {

    override fun onConfigure(db: SQLiteDatabase) {
        super.onConfigure(db)
        db.enableWriteAheadLogging()
        db.setForeignKeyConstraintsEnabled(true)
    }

    override fun onCreate(db: SQLiteDatabase) {
        db.execSQL(
            """
            CREATE TABLE IF NOT EXISTS Adobe_images (
                id_local INTEGER PRIMARY KEY AUTOINCREMENT,
                id_global TEXT UNIQUE NOT NULL,
                rootFolder INTEGER NOT NULL DEFAULT 1,
                pathFromRoot TEXT NOT NULL,
                real_file_path TEXT UNIQUE NOT NULL,
                displayName TEXT NOT NULL,
                captureTime TEXT,
                rating INTEGER DEFAULT 0,
                colorLabels TEXT DEFAULT '',
                pick INTEGER DEFAULT 0,
                fileWidth INTEGER DEFAULT 0,
                fileHeight INTEGER DEFAULT 0,
                orientation INTEGER DEFAULT 1,
                isRaw INTEGER DEFAULT 0,
                albumName TEXT DEFAULT 'All Photos'
            )
            """.trimIndent()
        )
        db.execSQL(
            """
            CREATE TABLE IF NOT EXISTS Adobe_imageDevelopSettings (
                id_local INTEGER PRIMARY KEY AUTOINCREMENT,
                image INTEGER UNIQUE NOT NULL REFERENCES Adobe_images(id_local) ON DELETE CASCADE,
                hasDevelopAdjustments INTEGER DEFAULT 0,
                exposure REAL DEFAULT 0.0,
                temperature REAL DEFAULT 5500.0,
                text TEXT NOT NULL
            )
            """.trimIndent()
        )
        db.execSQL(
            """
            CREATE TABLE IF NOT EXISTS AgDevelopPreset (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                name TEXT NOT NULL,
                is_user INTEGER NOT NULL DEFAULT 1,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                xmpPath TEXT,
                presetJson TEXT NOT NULL
            )
            """.trimIndent()
        )
        db.execSQL(
            """
            CREATE TABLE IF NOT EXISTS Adobe_libraryImageDevelopHistoryStep (
                id_local INTEGER PRIMARY KEY AUTOINCREMENT,
                image INTEGER NOT NULL REFERENCES Adobe_images(id_local) ON DELETE CASCADE,
                name TEXT NOT NULL,
                valueString TEXT NOT NULL,
                dateCreated TEXT NOT NULL,
                paramsJson TEXT NOT NULL
            )
            """.trimIndent()
        )
        db.execSQL(
            """
            CREATE TABLE IF NOT EXISTS Adobe_imageVersions (
                id_local INTEGER PRIMARY KEY AUTOINCREMENT,
                image INTEGER NOT NULL REFERENCES Adobe_images(id_local) ON DELETE CASCADE,
                name TEXT NOT NULL,
                createdAt TEXT NOT NULL,
                paramsJson TEXT NOT NULL
            )
            """.trimIndent()
        )
        db.execSQL(
            """
            CREATE TABLE IF NOT EXISTS AgLibraryCollection (
                id_local INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                isQuickCollection INTEGER DEFAULT 0,
                createdAt TEXT NOT NULL
            )
            """.trimIndent()
        )
        seedDefaultCollections(db)
        seedBuiltinPresets(db)
    }

    override fun onUpgrade(db: SQLiteDatabase, oldVersion: Int, newVersion: Int) {
        onCreate(db)
    }

    private fun seedDefaultCollections(db: SQLiteDatabase) {
        val now = nowString()
        listOf("Quick Collection" to 1, "Portraits" to 0, "Raw DNG" to 0).forEach { (name, quick) ->
            val cv = ContentValues().apply {
                put("name", name)
                put("isQuickCollection", quick)
                put("createdAt", now)
            }
            db.insertWithOnConflict("AgLibraryCollection", null, cv, SQLiteDatabase.CONFLICT_IGNORE)
        }
    }

    private fun seedBuiltinPresets(db: SQLiteDatabase) {
        for (preset in BuiltinPresets.all()) {
            val cv = ContentValues().apply {
                put("id", preset.id)
                put("category", preset.category)
                put("name", preset.name)
                put("is_user", 0)
                put("is_favorite", if (preset.isFavorite) 1 else 0)
                put("xmpPath", preset.xmpPath)
                put("presetJson", preset.params.toJsonString())
            }
            db.insertWithOnConflict("AgDevelopPreset", null, cv, SQLiteDatabase.CONFLICT_IGNORE)
        }
    }

    fun ensureSeededPresets() {
        seedBuiltinPresets(writableDatabase)
    }

    fun upsertPhoto(asset: PhotoAsset): PhotoAsset {
        val db = writableDatabase
        val existingId = db.rawQuery(
            "SELECT id_local FROM Adobe_images WHERE real_file_path = ? LIMIT 1",
            arrayOf(asset.uriString)
        ).use { c -> if (c.moveToFirst()) c.getLong(0) else -1L }

        val idLocal = if (existingId > 0L) {
            existingId
        } else {
            val cv = ContentValues().apply {
                put("id_global", asset.idGlobal)
                put("rootFolder", 1)
                put("pathFromRoot", asset.displayName)
                put("real_file_path", asset.uriString)
                put("displayName", asset.displayName)
                put("captureTime", asset.captureTime.ifBlank { nowString() })
                put("rating", asset.rating)
                put("colorLabels", asset.colorLabel.code)
                put("pick", asset.pickFlag.code)
                put("fileWidth", asset.width)
                put("fileHeight", asset.height)
                put("orientation", asset.orientation)
                put("isRaw", if (asset.isRaw) 1 else 0)
                put("albumName", asset.albumName)
            }
            val rowId = db.insert("Adobe_images", null, cv)
            val devCv = ContentValues().apply {
                put("image", rowId)
                put("hasDevelopAdjustments", 0)
                put("exposure", asset.params.exposure.toDouble())
                put("temperature", asset.params.temperature.toDouble())
                put("text", asset.params.toJsonString())
            }
            db.insert("Adobe_imageDevelopSettings", null, devCv)
            rowId
        }
        return getPhotoById(idLocal) ?: asset.copy(idLocal = idLocal)
    }

    fun getAllPhotos(): List<PhotoAsset> {
        val db = readableDatabase
        val list = mutableListOf<PhotoAsset>()
        val sql = """
            SELECT i.id_local, i.id_global, i.real_file_path, i.displayName, i.captureTime,
                   i.rating, i.colorLabels, i.pick, i.fileWidth, i.fileHeight, i.orientation,
                   i.isRaw, i.albumName, d.text
            FROM Adobe_images i
            LEFT JOIN Adobe_imageDevelopSettings d ON d.image = i.id_local
            ORDER BY i.id_local DESC
        """.trimIndent()
        db.rawQuery(sql, null).use { c ->
            while (c.moveToNext()) {
                list.add(readAssetRow(c))
            }
        }
        return list
    }

    fun getPhotoById(idLocal: Long): PhotoAsset? {
        val sql = """
            SELECT i.id_local, i.id_global, i.real_file_path, i.displayName, i.captureTime,
                   i.rating, i.colorLabels, i.pick, i.fileWidth, i.fileHeight, i.orientation,
                   i.isRaw, i.albumName, d.text
            FROM Adobe_images i
            LEFT JOIN Adobe_imageDevelopSettings d ON d.image = i.id_local
            WHERE i.id_local = ? LIMIT 1
        """.trimIndent()
        readableDatabase.rawQuery(sql, arrayOf(idLocal.toString())).use { c ->
            return if (c.moveToFirst()) readAssetRow(c) else null
        }
    }

    private fun readAssetRow(c: android.database.Cursor): PhotoAsset = PhotoAsset(
        idLocal = c.getLong(0),
        idGlobal = c.getString(1) ?: UUID.randomUUID().toString(),
        uriString = c.getString(2) ?: "",
        displayName = c.getString(3) ?: "Photo",
        captureTime = c.getString(4) ?: "",
        rating = c.getInt(5),
        colorLabel = ColorLabel.fromCode(c.getString(6)),
        pickFlag = PickFlag.fromCode(c.getInt(7)),
        width = c.getInt(8),
        height = c.getInt(9),
        orientation = c.getInt(10),
        isRaw = c.getInt(11) == 1,
        albumName = c.getString(12) ?: "All Photos",
        params = DevelopParams.fromJsonString(c.getString(13))
    )

    fun saveDevelopParams(photoId: Long, params: DevelopParams) {
        val cv = ContentValues().apply {
            put("image", photoId)
            put("hasDevelopAdjustments", if (params != DevelopParams()) 1 else 0)
            put("exposure", params.exposure.toDouble())
            put("temperature", params.temperature.toDouble())
            put("text", params.toJsonString())
        }
        writableDatabase.insertWithOnConflict(
            "Adobe_imageDevelopSettings",
            null,
            cv,
            SQLiteDatabase.CONFLICT_REPLACE
        )
    }

    fun updateRating(photoId: Long, rating: Int) {
        val cv = ContentValues().apply { put("rating", rating.coerceIn(0, 5)) }
        writableDatabase.update("Adobe_images", cv, "id_local = ?", arrayOf(photoId.toString()))
    }

    fun updatePickFlag(photoId: Long, flag: PickFlag) {
        val cv = ContentValues().apply { put("pick", flag.code) }
        writableDatabase.update("Adobe_images", cv, "id_local = ?", arrayOf(photoId.toString()))
    }

    fun updateColorLabel(photoId: Long, label: ColorLabel) {
        val cv = ContentValues().apply { put("colorLabels", label.code) }
        writableDatabase.update("Adobe_images", cv, "id_local = ?", arrayOf(photoId.toString()))
    }

    fun updateAlbum(photoId: Long, albumName: String) {
        val cv = ContentValues().apply { put("albumName", albumName) }
        writableDatabase.update("Adobe_images", cv, "id_local = ?", arrayOf(photoId.toString()))
    }

    fun deletePhoto(photoId: Long) {
        writableDatabase.delete("Adobe_images", "id_local = ?", arrayOf(photoId.toString()))
    }

    fun addHistoryStep(photoId: Long, name: String, valueSummary: String, params: DevelopParams) {
        val cv = ContentValues().apply {
            put("image", photoId)
            put("name", name)
            put("valueString", valueSummary)
            put("dateCreated", nowString())
            put("paramsJson", params.toJsonString())
        }
        writableDatabase.insert("Adobe_libraryImageDevelopHistoryStep", null, cv)
    }

    fun getHistorySteps(photoId: Long): List<HistoryEntry> {
        val out = mutableListOf<HistoryEntry>()
        readableDatabase.rawQuery(
            """
            SELECT id_local, name, valueString, dateCreated, paramsJson
            FROM Adobe_libraryImageDevelopHistoryStep
            WHERE image = ? ORDER BY id_local DESC LIMIT 40
            """.trimIndent(),
            arrayOf(photoId.toString())
        ).use { c ->
            while (c.moveToNext()) {
                out.add(
                    HistoryEntry(
                        id = c.getLong(0),
                        name = c.getString(1) ?: "",
                        valueSummary = c.getString(2) ?: "",
                        timestamp = c.getString(3) ?: "",
                        params = DevelopParams.fromJsonString(c.getString(4))
                    )
                )
            }
        }
        return out
    }

    fun saveVersion(photoId: Long, name: String, params: DevelopParams) {
        val cv = ContentValues().apply {
            put("image", photoId)
            put("name", name)
            put("createdAt", nowString())
            put("paramsJson", params.toJsonString())
        }
        writableDatabase.insert("Adobe_imageVersions", null, cv)
    }

    fun getVersions(photoId: Long): List<PhotoVersion> {
        val out = mutableListOf<PhotoVersion>()
        readableDatabase.rawQuery(
            "SELECT id_local, name, createdAt, paramsJson FROM Adobe_imageVersions WHERE image = ? ORDER BY id_local DESC",
            arrayOf(photoId.toString())
        ).use { c ->
            while (c.moveToNext()) {
                out.add(
                    PhotoVersion(
                        id = c.getLong(0),
                        name = c.getString(1) ?: "Version",
                        createdAt = c.getString(2) ?: "",
                        params = DevelopParams.fromJsonString(c.getString(3))
                    )
                )
            }
        }
        return out
    }

    fun getAllPresets(): List<DevelopPreset> {
        val out = mutableListOf<DevelopPreset>()
        readableDatabase.rawQuery(
            "SELECT id, category, name, is_user, is_favorite, xmpPath, presetJson FROM AgDevelopPreset ORDER BY is_favorite DESC, is_user DESC, category ASC, name ASC",
            null
        ).use { c ->
            while (c.moveToNext()) {
                out.add(
                    DevelopPreset(
                        id = c.getString(0),
                        category = c.getString(1),
                        name = c.getString(2),
                        isUser = c.getInt(3) == 1,
                        isFavorite = c.getInt(4) == 1,
                        xmpPath = c.getString(5),
                        params = DevelopParams.fromJsonString(c.getString(6))
                    )
                )
            }
        }
        return out
    }

    fun upsertPreset(preset: DevelopPreset) {
        val cv = ContentValues().apply {
            put("id", preset.id)
            put("category", preset.category)
            put("name", preset.name)
            put("is_user", if (preset.isUser) 1 else 0)
            put("is_favorite", if (preset.isFavorite) 1 else 0)
            put("xmpPath", preset.xmpPath)
            put("presetJson", preset.params.toJsonString())
        }
        writableDatabase.insertWithOnConflict("AgDevelopPreset", null, cv, SQLiteDatabase.CONFLICT_REPLACE)
    }

    fun deletePreset(id: String) {
        writableDatabase.delete("AgDevelopPreset", "id = ? AND is_user = 1", arrayOf(id))
    }

    fun renamePreset(id: String, newName: String) {
        val cv = ContentValues().apply { put("name", newName) }
        writableDatabase.update("AgDevelopPreset", cv, "id = ? AND is_user = 1", arrayOf(id))
    }

    fun togglePresetFavorite(id: String, isFavorite: Boolean) {
        val cv = ContentValues().apply { put("is_favorite", if (isFavorite) 1 else 0) }
        writableDatabase.update("AgDevelopPreset", cv, "id = ?", arrayOf(id))
    }

    fun createAlbum(name: String) {
        val clean = name.trim()
        if (clean.isEmpty()) return
        val cv = ContentValues().apply {
            put("name", clean)
            put("isQuickCollection", 0)
            put("createdAt", nowString())
        }
        writableDatabase.insertWithOnConflict("AgLibraryCollection", null, cv, SQLiteDatabase.CONFLICT_IGNORE)
    }

    fun deleteAlbum(name: String) {
        if (name == "Quick Collection" || name == "All Photos") return
        val cv = ContentValues().apply { put("albumName", "All Photos") }
        writableDatabase.update("Adobe_images", cv, "albumName = ?", arrayOf(name))
        writableDatabase.delete("AgLibraryCollection", "name = ?", arrayOf(name))
    }

    fun getAllAlbums(): List<AlbumInfo> {
        val albumNames = linkedMapOf<String, Boolean>()
        readableDatabase.rawQuery(
            "SELECT name, isQuickCollection FROM AgLibraryCollection ORDER BY isQuickCollection DESC, name ASC",
            null
        ).use { c ->
            while (c.moveToNext()) {
                albumNames[c.getString(0)] = c.getInt(1) == 1
            }
        }
        readableDatabase.rawQuery(
            "SELECT DISTINCT albumName FROM Adobe_images WHERE albumName IS NOT NULL AND albumName != '' AND albumName != 'All Photos'",
            null
        ).use { c ->
            while (c.moveToNext()) {
                val n = c.getString(0)
                if (!albumNames.containsKey(n)) albumNames[n] = false
            }
        }
        return albumNames.map { (name, isQuick) ->
            var cnt = 0
            var cover: String? = null
            readableDatabase.rawQuery(
                "SELECT COUNT(*), MAX(real_file_path) FROM Adobe_images WHERE albumName = ?",
                arrayOf(name)
            ).use { c ->
                if (c.moveToFirst()) {
                    cnt = c.getInt(0)
                    cover = c.getString(1)
                }
            }
            AlbumInfo(name = name, photoCount = cnt, coverUri = cover, isQuickCollection = isQuick)
        }
    }

    private fun nowString(): String =
        SimpleDateFormat("yyyy-MM-dd HH:mm:ss", Locale.US).format(Date())
}
