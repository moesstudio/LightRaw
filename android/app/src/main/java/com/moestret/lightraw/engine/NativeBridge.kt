package com.moestret.lightraw.engine

import android.graphics.Bitmap
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.DevelopPreset
import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * Optional JNI bridge to `liblightraw_jni.so` (shared Rust core).
 * When packaged without NDK `.so` libraries, `isNativeAvailable` is false and
 * LightRaw automatically executes the exact pure-Kotlin PV2012 engine.
 */
object NativeBridge {
    val isNativeAvailable: Boolean

    init {
        isNativeAvailable = try {
            System.loadLibrary("lightraw_jni")
            true
        } catch (_: UnsatisfiedLinkError) {
            false
        }
    }

    external fun nativeVersion(): String
    external fun nativeProcessRgba(
        rgbaBytes: ByteArray,
        width: Int,
        height: Int,
        paramsJson: String,
        draftInteractive: Boolean,
        applyCrop: Boolean
    ): ByteArray?

    external fun nativeMatchColorRef(
        refRgba: ByteArray,
        refW: Int,
        refH: Int,
        tgtRgba: ByteArray,
        tgtW: Int,
        tgtH: Int,
        tgtParamsJson: String
    ): String?

    external fun nativeParseXmp(xmpXml: String, fallbackName: String): String?
    external fun nativeExportXmp(presetJson: String): String?
    external fun nativeBuiltinPresetsJson(): String?

    fun tryNativeProcessBitmap(
        src: Bitmap,
        params: DevelopParams,
        draftInteractive: Boolean,
        applyCrop: Boolean
    ): RenderedFrame? {
        if (!isNativeAvailable) return null
        return try {
            val argb = IntArray(src.width * src.height)
            src.getPixels(argb, 0, src.width, 0, 0, src.width, src.height)
            val rgba = ByteArray(argb.size * 4)
            for (i in argb.indices) {
                val c = argb[i]
                val b = i * 4
                rgba[b] = ((c ushr 16) and 0xFF).toByte()
                rgba[b + 1] = ((c ushr 8) and 0xFF).toByte()
                rgba[b + 2] = (c and 0xFF).toByte()
                rgba[b + 3] = ((c ushr 24) and 0xFF).toByte()
            }
            val packed = nativeProcessRgba(
                rgba,
                src.width,
                src.height,
                params.toJsonString(),
                draftInteractive,
                applyCrop
            ) ?: return null

            val hdrBytes = 8 + 4 * 256 * 4
            if (packed.size <= hdrBytes) return null
            val bb = ByteBuffer.wrap(packed).order(ByteOrder.LITTLE_ENDIAN)
            val outW = bb.int.coerceAtLeast(1)
            val outH = bb.int.coerceAtLeast(1)
            val histR = IntArray(256) { bb.int }
            val histG = IntArray(256) { bb.int }
            val histB = IntArray(256) { bb.int }
            val histLum = IntArray(256) { bb.int }

            val pxCount = outW * outH
            if (packed.size < hdrBytes + pxCount * 4) return null
            val outArgb = IntArray(pxCount)
            for (i in 0 until pxCount) {
                val off = hdrBytes + i * 4
                val r = packed[off].toInt() and 0xFF
                val g = packed[off + 1].toInt() and 0xFF
                val b = packed[off + 2].toInt() and 0xFF
                val a = packed[off + 3].toInt() and 0xFF
                outArgb[i] = (a shl 24) or (r shl 16) or (g shl 8) or b
            }
            val bmp = Bitmap.createBitmap(outArgb, outW, outH, Bitmap.Config.ARGB_8888)
            RenderedFrame(bmp, histR, histG, histB, histLum)
        } catch (_: Throwable) {
            null
        }
    }
}
