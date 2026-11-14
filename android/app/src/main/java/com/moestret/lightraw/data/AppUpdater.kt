package com.moestret.lightraw.data

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.content.pm.PackageInfo
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.core.content.FileProvider
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest

/**
 * Over-the-air GitHub Releases auto-updater supporting seamless in-place APK upgrades
 * without uninstalling existing catalog, presets, or imported photo edits.
 */
object AppUpdater {

    const val GITHUB_LATEST_API = "https://api.github.com/repos/moesstudio/LightRaw/releases/latest"
    const val GITHUB_RELEASES_PAGE = "https://github.com/moesstudio/LightRaw/releases/latest"
    private const val PREFS_NAME = "lightraw_updater"
    private const val KEY_LAST_CHECK_MS = "last_auto_check_ms"
    private const val KEY_SKIPPED_TAG = "skipped_tag"
    private const val AUTO_CHECK_INTERVAL_MS = 6L * 60L * 60L * 1000L

    data class ReleaseAsset(
        val name: String,
        val downloadUrl: String,
        val sizeBytes: Long
    )

    data class ReleaseInfo(
        val tagName: String,
        val version: String,
        val title: String,
        val releaseNotes: String,
        val htmlUrl: String,
        val asset: ReleaseAsset?
    )

    sealed class CheckResult {
        data class UpdateAvailable(val currentVersion: String, val release: ReleaseInfo) : CheckResult()
        data class UpToDate(val currentVersion: String, val latestTag: String) : CheckResult()
        data class Failed(val reason: String) : CheckResult()
    }

    fun parseVersionTuple(version: String): List<Int> {
        val cleaned = version.trim()
            .removePrefix("v")
            .removePrefix("V")
            .substringBefore("-")
            .substringBefore("+")
        if (cleaned.isEmpty()) return listOf(0, 0, 0)
        return cleaned.split(".")
            .take(4)
            .map { seg -> seg.takeWhile { it.isDigit() }.toIntOrNull() ?: 0 }
    }

    fun isNewerVersion(remoteVersion: String, currentVersion: String): Boolean {
        val r = parseVersionTuple(remoteVersion)
        val c = parseVersionTuple(currentVersion)
        val maxLen = maxOf(r.size, c.size)
        for (i in 0 until maxLen) {
            val rv = r.getOrElse(i) { 0 }
            val cv = c.getOrElse(i) { 0 }
            if (rv != cv) return rv > cv
        }
        return false
    }

    fun selectApkAssetName(assetNames: List<String>, isDebugPackage: Boolean): String? {
        val preferred = if (isDebugPackage) {
            listOf("LightRaw-Android-debug.apk", "LightRaw-Android-release.apk")
        } else {
            listOf("LightRaw-Android-release.apk", "LightRaw-Android-debug.apk")
        }
        for (target in preferred) {
            assetNames.firstOrNull { it.equals(target, ignoreCase = true) }?.let { return it }
        }
        return assetNames.firstOrNull { it.endsWith(".apk", ignoreCase = true) }
    }

    fun getInstalledVersionName(ctx: Context): String {
        val info = getPackageInfoCompat(ctx.packageManager, ctx.packageName, 0)
        return info?.versionName ?: "1.2.0"
    }

    fun shouldAutoCheckNow(ctx: Context): Boolean {
        val prefs = ctx.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        val last = prefs.getLong(KEY_LAST_CHECK_MS, 0L)
        return (System.currentTimeMillis() - last) >= AUTO_CHECK_INTERVAL_MS
    }

    fun markChecked(ctx: Context) {
        ctx.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            .edit()
            .putLong(KEY_LAST_CHECK_MS, System.currentTimeMillis())
            .apply()
    }

    fun isTagSkipped(ctx: Context, tagName: String): Boolean {
        val skipped = ctx.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            .getString(KEY_SKIPPED_TAG, null)
        return skipped == tagName
    }

    fun skipTag(ctx: Context, tagName: String) {
        ctx.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            .edit()
            .putString(KEY_SKIPPED_TAG, tagName)
            .apply()
    }

    fun cleanupUpdateCache(ctx: Context) {
        val dir = File(ctx.cacheDir, "updates")
        if (dir.exists()) {
            dir.listFiles()?.forEach { it.delete() }
        }
    }

    suspend fun fetchLatestRelease(ctx: Context): CheckResult = withContext(Dispatchers.IO) {
        val currentVersion = getInstalledVersionName(ctx)
        val isDebugPkg = ctx.packageName.endsWith(".debug")
        try {
            val conn = (URL(GITHUB_LATEST_API).openConnection() as HttpURLConnection).apply {
                requestMethod = "GET"
                connectTimeout = 10_000
                readTimeout = 15_000
                setRequestProperty("Accept", "application/vnd.github+json")
                setRequestProperty("User-Agent", "LightRaw-Android/$currentVersion")
            }
            val code = conn.responseCode
            val body = if (code in 200..299) {
                conn.inputStream.bufferedReader().use { it.readText() }
            } else {
                conn.disconnect()
                return@withContext CheckResult.Failed("GitHub API returned HTTP $code")
            }
            conn.disconnect()

            val json = JSONObject(body)
            val tagName = json.optString("tag_name", "").ifBlank {
                return@withContext CheckResult.Failed("Latest release missing tag_name")
            }
            val remoteVer = tagName.removePrefix("v").removePrefix("V")
            val assetsArr = json.optJSONArray("assets") ?: JSONArray()
            val assets = buildList {
                for (i in 0 until assetsArr.length()) {
                    val obj = assetsArr.getJSONObject(i)
                    val name = obj.optString("name", "")
                    val url = obj.optString("browser_download_url", "")
                    val size = obj.optLong("size", -1L)
                    if (name.isNotBlank() && url.isNotBlank()) {
                        add(ReleaseAsset(name, url, size))
                    }
                }
            }
            val pickedName = selectApkAssetName(assets.map { it.name }, isDebugPkg)
            val pickedAsset = pickedName?.let { n -> assets.firstOrNull { it.name == n } }

            val release = ReleaseInfo(
                tagName = tagName,
                version = remoteVer,
                title = json.optString("name", "LightRaw $tagName").ifBlank { "LightRaw $tagName" },
                releaseNotes = json.optString("body", "").take(900),
                htmlUrl = json.optString("html_url", GITHUB_RELEASES_PAGE),
                asset = pickedAsset
            )
            markChecked(ctx)

            if (isNewerVersion(remoteVer, currentVersion)) {
                CheckResult.UpdateAvailable(currentVersion, release)
            } else {
                CheckResult.UpToDate(currentVersion, tagName)
            }
        } catch (t: Throwable) {
            CheckResult.Failed(t.message ?: "Network error while checking updates")
        }
    }

    suspend fun downloadApk(
        ctx: Context,
        asset: ReleaseAsset,
        onProgress: (downloadedBytes: Long, totalBytes: Long) -> Unit
    ): Result<File> = withContext(Dispatchers.IO) {
        val updatesDir = File(ctx.cacheDir, "updates").apply { mkdirs() }
        val partFile = File(updatesDir, "${asset.name}.part")
        val finalFile = File(updatesDir, asset.name)
        if (partFile.exists()) partFile.delete()
        if (finalFile.exists()) finalFile.delete()

        var conn: HttpURLConnection? = null
        try {
            var currentUrl = asset.downloadUrl
            var redirects = 0
            while (true) {
                conn = (URL(currentUrl).openConnection() as HttpURLConnection).apply {
                    instanceFollowRedirects = true
                    connectTimeout = 15_000
                    readTimeout = 30_000
                    setRequestProperty("User-Agent", "LightRaw-Android")
                }
                val code = conn.responseCode
                if (code in 300..399 && redirects < 6) {
                    val loc = conn.getHeaderField("Location") ?: break
                    conn.disconnect()
                    currentUrl = loc
                    redirects++
                } else {
                    break
                }
            }
            val activeConn = conn ?: return@withContext Result.failure(IllegalStateException("No connection"))
            if (activeConn.responseCode !in 200..299) {
                return@withContext Result.failure(IllegalStateException("Download HTTP ${activeConn.responseCode}"))
            }

            val total = activeConn.contentLengthLong.takeIf { it > 0L } ?: asset.sizeBytes
            activeConn.inputStream.use { input ->
                FileOutputStream(partFile).use { output ->
                    val buf = ByteArray(16 * 1024)
                    var copied = 0L
                    while (true) {
                        val read = input.read(buf)
                        if (read <= 0) break
                        output.write(buf, 0, read)
                        copied += read
                        withContext(Dispatchers.Main) {
                            onProgress(copied, total)
                        }
                    }
                    output.flush()
                }
            }
            if (!partFile.renameTo(finalFile)) {
                partFile.copyTo(finalFile, overwrite = true)
                partFile.delete()
            }
            Result.success(finalFile)
        } catch (t: Throwable) {
            partFile.delete()
            Result.failure(t)
        } finally {
            conn?.disconnect()
        }
    }

    /**
     * Verifies whether the downloaded APK shares the exact signing certificate with the currently
     * installed package so Android can perform an in-place upgrade without uninstalling.
     * Returns null if archive signatures cannot be inspected.
     */
    fun checkSignatureMatchesInstalled(ctx: Context, apkFile: File): Boolean? {
        return try {
            val pm = ctx.packageManager
            val installedDigests = getInstalledSigningDigests(pm, ctx.packageName)
            val archiveDigests = getArchiveSigningDigests(pm, apkFile.absolutePath)
            if (installedDigests.isEmpty() || archiveDigests.isEmpty()) {
                null
            } else {
                installedDigests == archiveDigests
            }
        } catch (_: Throwable) {
            null
        }
    }

    fun needsInstallPermissionPrompt(ctx: Context): Boolean {
        return Build.VERSION.SDK_INT >= Build.VERSION_CODES.O && !ctx.packageManager.canRequestPackageInstalls()
    }

    fun launchInstallPermissionSettings(activity: Activity) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val intent = Intent(
                Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES,
                Uri.parse("package:${activity.packageName}")
            )
            activity.startActivity(intent)
        }
    }

    fun launchApkInstaller(activity: Activity, apkFile: File) {
        val uri = FileProvider.getUriForFile(
            activity,
            "${activity.packageName}.fileprovider",
            apkFile
        )
        val installIntent = Intent(Intent.ACTION_VIEW).apply {
            setDataAndType(uri, "application/vnd.android.package-archive")
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK)
        }
        activity.startActivity(installIntent)
    }

    @Suppress("DEPRECATION")
    private fun getPackageInfoCompat(pm: PackageManager, pkg: String, flags: Int): PackageInfo? {
        return try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                pm.getPackageInfo(pkg, PackageManager.PackageInfoFlags.of(flags.toLong()))
            } else {
                pm.getPackageInfo(pkg, flags)
            }
        } catch (_: Throwable) {
            null
        }
    }

    @Suppress("DEPRECATION")
    private fun getInstalledSigningDigests(pm: PackageManager, pkg: String): Set<String> {
        val flags = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            PackageManager.GET_SIGNING_CERTIFICATES
        } else {
            PackageManager.GET_SIGNATURES
        }
        val info = getPackageInfoCompat(pm, pkg, flags) ?: return emptySet()
        return extractCertDigests(info)
    }

    @Suppress("DEPRECATION")
    private fun getArchiveSigningDigests(pm: PackageManager, apkPath: String): Set<String> {
        val flags = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            PackageManager.GET_SIGNING_CERTIFICATES
        } else {
            PackageManager.GET_SIGNATURES
        }
        val info = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            pm.getPackageArchiveInfo(apkPath, PackageManager.PackageInfoFlags.of(flags.toLong()))
        } else {
            pm.getPackageArchiveInfo(apkPath, flags)
        } ?: return emptySet()
        return extractCertDigests(info)
    }

    @Suppress("DEPRECATION")
    private fun extractCertDigests(info: PackageInfo): Set<String> {
        val sigs = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            info.signingInfo?.apkContentsSigners
        } else {
            info.signatures
        } ?: return emptySet()
        val md = MessageDigest.getInstance("SHA-256")
        return sigs.map { sig ->
            md.digest(sig.toByteArray()).joinToString("") { "%02x".format(it) }
        }.toSet()
    }
}
