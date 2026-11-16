package com.moestret.lightraw.ui

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import com.moestret.lightraw.data.BuiltinPresets
import com.moestret.lightraw.data.XmpPresetCodec
import com.moestret.lightraw.engine.KotlinPv2012Engine
import com.moestret.lightraw.model.AlbumInfo
import com.moestret.lightraw.model.ColorLabel
import com.moestret.lightraw.model.CopySettingsMask
import com.moestret.lightraw.model.CurvePoint
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.DevelopPreset
import com.moestret.lightraw.model.PickFlag
import com.moestret.lightraw.model.WatermarkAnchor
import com.moestret.lightraw.model.WatermarkSettings
import com.moestret.lightraw.model.WatermarkSizeMode
import com.moestret.lightraw.model.WatermarkStyle

class PipelineAndXmpTest {

    @Test
    fun xmpRoundtripPreservesCoreAdjustments() {
        val srcParams = DevelopParams(
            temperature = 6400f,
            tint = 12f,
            exposure = 0.65f,
            contrast = 24f,
            highlights = -30f,
            shadows = 28f,
            clarity = 18f,
            vignetteAmount = -22f,
            toneCurveMaster = listOf(
                CurvePoint(0f, 0.03f),
                CurvePoint(0.5f, 0.54f),
                CurvePoint(1f, 0.98f)
            )
        )
        val preset = DevelopPreset("test1", "Film", "SunsetWarm", true, false, srcParams)
        val xmp = XmpPresetCodec.exportToXmp(preset)
        val parsed = XmpPresetCodec.parseXmp(xmp, "Fallback")

        assertNotNull(parsed)
        assertEquals("SunsetWarm", parsed!!.name)
        assertEquals(6400f, parsed.params.temperature, 1f)
        assertEquals(0.65f, parsed.params.exposure, 0.02f)
        assertEquals(-30f, parsed.params.highlights, 1f)
        assertEquals(3, parsed.params.toneCurveMaster.size)
    }

    @Test
    fun selectiveCopyMaskCopiesOnlyEnabledGroups() {
        val src = DevelopParams(exposure = 1.4f, saturation = 30f, cropLeft = 0.2f)
        val dst = DevelopParams(exposure = -0.3f, saturation = 0f, cropLeft = 0.0f)
        val mask = CopySettingsMask(
            basicTone = true,
            presence = false,
            cropAndTransform = false
        )
        val merged = mask.applyTo(src, dst)
        assertEquals(1.4f, merged.exposure, 0.001f)
        assertEquals(0f, merged.saturation, 0.001f)
        assertEquals(0f, merged.cropLeft, 0.001f)
    }

    @Test
    fun toneCurveLutEvaluatesMonotonicWithSContrast() {
        val pts = listOf(CurvePoint(0f, 0f), CurvePoint(0.25f, 0.18f), CurvePoint(0.75f, 0.85f), CurvePoint(1f, 1f))
        val lut = KotlinPv2012Engine.buildCurveLut(pts)
        assertEquals(256, lut.size)
        assertTrue(lut[64] < 0.25f)
        assertTrue(lut[192] > 0.78f)
    }

    @Test
    fun whiteBalanceWarmAndCoolShiftRelativeGains() {
        val warm = KotlinPv2012Engine.computeWbGains(7500f, 0f)
        val cool = KotlinPv2012Engine.computeWbGains(3500f, 0f)
        assertTrue(warm[0] > warm[2])
        assertTrue(cool[2] > cool[0])
    }

    @Test
    fun builtinPresetsArePopulatedAndNonEmpty() {
        assertTrue(BuiltinPresets.all().size >= 8)
    }

    @Test
    fun xmpRoundtripPreservesGeometryWarpAndLensDistortion() {
        val src = DevelopParams(
            lensDistortion = -18f,
            perspectiveVertical = 24f,
            perspectiveHorizontal = -14f,
            perspectiveAspect = 12f,
            geometryScale = 112f,
            geometryOffsetX = -8f,
            geometryOffsetY = 6f
        )
        val xml = XmpPresetCodec.exportToXmp(
            DevelopPreset(
                id = "geo",
                category = "Architecture",
                name = "Geo Fix",
                isUser = true,
                isFavorite = false,
                params = src
            )
        )
        val parsed = XmpPresetCodec.parseXmp(xml, "Fallback")
        assertNotNull(parsed)
        val p = parsed!!.params
        assertEquals(-18f, p.lensDistortion, 0.05f)
        assertEquals(24f, p.perspectiveVertical, 0.05f)
        assertEquals(-14f, p.perspectiveHorizontal, 0.05f)
        assertEquals(12f, p.perspectiveAspect, 0.05f)
        assertEquals(112f, p.geometryScale, 0.05f)
        assertEquals(-8f, p.geometryOffsetX, 0.05f)
        assertEquals(6f, p.geometryOffsetY, 0.05f)
    }

    @Test
    fun copyMaskSyncsGeometryCropWhenEnabled() {
        val src = DevelopParams(
            perspectiveVertical = 30f,
            perspectiveHorizontal = -10f,
            perspectiveAspect = 15f,
            geometryScale = 108f,
            geometryOffsetX = 4f,
            geometryOffsetY = -5f
        )
        val dst = DevelopParams(exposure = 0.8f)
        val mask = CopySettingsMask(basicTone = false, cropAndTransform = true)
        val merged = mask.applyTo(src, dst)
        assertEquals(0.8f, merged.exposure, 0.001f)
        assertEquals(30f, merged.perspectiveVertical, 0.001f)
        assertEquals(-10f, merged.perspectiveHorizontal, 0.001f)
        assertEquals(15f, merged.perspectiveAspect, 0.001f)
        assertEquals(108f, merged.geometryScale, 0.001f)
        assertEquals(4f, merged.geometryOffsetX, 0.001f)
        assertEquals(-5f, merged.geometryOffsetY, 0.001f)
    }

    @Test
    fun catalogLabelsAndAlbumsRoundtripCleanly() {
        assertEquals(ColorLabel.RED, ColorLabel.fromCode("red"))
        assertEquals(ColorLabel.PURPLE, ColorLabel.fromCode("Purple"))
        assertEquals(ColorLabel.NONE, ColorLabel.fromCode("unknown"))
        assertEquals(PickFlag.PICKED, PickFlag.fromCode(1))
        assertEquals(PickFlag.REJECTED, PickFlag.fromCode(-1))
        val album = AlbumInfo(name = "Portraits", photoCount = 12, coverUri = null)
        assertEquals("Portraits", album.name)
        assertEquals(12, album.photoCount)
    }

    @Test
    fun graphicWatermarkComputesProportionalFitAndFillBounds() {
        val wmProp = WatermarkSettings(
            enabled = true,
            style = WatermarkStyle.GRAPHIC,
            graphicUriString = com.moestret.lightraw.engine.WatermarkRenderer.BUILTIN_LR_MARK_URI,
            graphicDisplayName = "LightRaw Studio Mark.png",
            anchor = WatermarkAnchor.BOTTOM_RIGHT,
            sizeMode = WatermarkSizeMode.PROPORTIONAL,
            sizePercent = 25f,
            insetPercent = 5f
        )
        val propBox = com.moestret.lightraw.engine.WatermarkRenderer.computeGraphicBox(1000, 800, 500, 250, wmProp)
        // 25% of 1000px = 250px wide, 2:1 aspect => 125px tall; BR inset 5% = (50, 40) -> x = 700, y = 635
        assertEquals(250f, propBox[2], 0.5f)
        assertEquals(125f, propBox[3], 0.5f)
        assertEquals(700f, propBox[0], 0.5f)
        assertEquals(635f, propBox[1], 0.5f)

        val wmFit = wmProp.copy(sizeMode = WatermarkSizeMode.FIT, anchor = WatermarkAnchor.CENTER)
        val fitBox = com.moestret.lightraw.engine.WatermarkRenderer.computeGraphicBox(1000, 800, 500, 250, wmFit)
        assertEquals(950f, fitBox[2], 0.5f)
        assertEquals(475f, fitBox[3], 0.5f)
    }

    @Test
    fun appUpdaterSemverAndAssetSelectionWork() {
        assertTrue(com.moestret.lightraw.data.AppUpdater.isNewerVersion("v1.2.1", "1.2.0"))
        assertTrue(com.moestret.lightraw.data.AppUpdater.isNewerVersion("v1.10.0", "1.2.0"))
        assertFalse(com.moestret.lightraw.data.AppUpdater.isNewerVersion("v1.2.0", "1.2.0"))
        assertFalse(com.moestret.lightraw.data.AppUpdater.isNewerVersion("1.1.0", "1.2.0"))

        val assets = listOf(
            "LightRaw-1.2.0-macOS.dmg",
            "LightRaw-Android-release.apk",
            "LightRaw-Android-debug.apk"
        )
        assertEquals(
            "LightRaw-Android-release.apk",
            com.moestret.lightraw.data.AppUpdater.selectApkAssetName(assets, isDebugPackage = false)
        )
        assertEquals(
            "LightRaw-Android-debug.apk",
            com.moestret.lightraw.data.AppUpdater.selectApkAssetName(assets, isDebugPackage = true)
        )
    }

    @Test
    fun mobileColorMatchGenerates3dLutPaletteAndSerializesCleanly() {
        fun argb(r: Float, g: Float, b: Float): Int {
            val ri = (r * 255f).toInt().coerceIn(0, 255)
            val gi = (g * 255f).toInt().coerceIn(0, 255)
            val bi = (b * 255f).toInt().coerceIn(0, 255)
            return (0xFF shl 24) or (ri shl 16) or (gi shl 8) or bi
        }

        // Target: Flat washed-out S-Log3 portrait (lifted gray ~0.26, dull skin, muted highlights)
        val n = 900
        val tgtPx = IntArray(n) { i ->
            when {
                i < n / 3 -> argb(0.25f, 0.28f, 0.26f)
                i < (2 * n) / 3 -> argb(0.62f, 0.58f, 0.54f)
                else -> argb(0.74f, 0.74f, 0.72f)
            }
        }
        // Reference: Moody olive-emerald Joker palette with rich shadows & warm gold highlights
        val refPx = IntArray(n) { i ->
            when {
                i < n / 3 -> argb(0.04f, 0.11f, 0.06f)
                i < (2 * n) / 3 -> argb(0.28f, 0.30f, 0.15f)
                else -> argb(0.80f, 0.70f, 0.48f)
            }
        }

        val matched = com.moestret.lightraw.engine.AiColorMatchEngine.matchPixelsToReference(
            refPixels = refPx,
            targetPixels = tgtPx,
            targetParams = DevelopParams(),
            forceRawFlat = null,
            refName = "Joker_Ref"
        )

        assertTrue(matched.hasColorMatchLut())
        assertTrue(matched.matchRawFlat)
        assertEquals(10, matched.matchPalette.size)
        assertEquals("Joker_Ref", matched.matchRefName)

        // Check shadow crush & olive mood on sampled 3D LUT output
        val outRgb = FloatArray(3)
        val tmp1 = FloatArray(3); val tmp2 = FloatArray(3); val tmp3 = FloatArray(3); val tmp4 = FloatArray(3)
        com.moestret.lightraw.engine.AiColorMatchEngine.applyColorMatchLut(
            0.25f, 0.28f, 0.26f,
            matched.matchLutData, matched.matchLutDim,
            100f, 100f,
            tmp1, tmp2, tmp3, tmp4, outRgb
        )
        assertTrue("Expected dark crushed shadow output: ${outRgb[0]}", outRgb[0] < 0.16f)
        assertTrue("Expected olive/green mood G>R: ${outRgb[1]} vs ${outRgb[0]}", outRgb[1] > outRgb[0])

        // Check Tonal=0, Warna=0 restores exact identity
        com.moestret.lightraw.engine.AiColorMatchEngine.applyColorMatchLut(
            0.25f, 0.28f, 0.26f,
            matched.matchLutData, matched.matchLutDim,
            0f, 0f,
            tmp1, tmp2, tmp3, tmp4, outRgb
        )
        assertEquals(0.25f, outRgb[0], 0.005f)

        // Check Base64 uint16 LUT roundtrip, deepCopy, interpolate, and CopySettingsMask preservation
        val b64 = DevelopParams.encodeLutB64(matched.matchLutData)
        val decodedLut = DevelopParams.decodeLutB64(b64, matched.matchLutData.size)
        assertEquals(matched.matchLutData.size, decodedLut.size)
        assertEquals(matched.matchLutData[500], decodedLut[500], 0.001f)

        val copied = com.moestret.lightraw.model.CopySettingsMask(colorGrading = true)
            .applyTo(matched, DevelopParams())
        assertTrue(copied.hasColorMatchLut())
        assertEquals("Joker_Ref", copied.matchRefName)

        val halfStrength = DevelopParams.interpolate(DevelopParams(), matched, 50f)
        assertEquals(50f, halfStrength.matchToneAmount, 0.1f)
        assertEquals(50f, halfStrength.matchColorAmount, 0.1f)
    }
}
