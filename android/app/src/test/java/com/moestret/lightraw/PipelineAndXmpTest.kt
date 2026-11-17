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

    @Test
    fun testAiSemanticDualLutStudioPortraitPreservesHairShirtSkinAndBackdrop() {
        fun argb(r: Float, g: Float, b: Float): Int {
            val ri = (r.coerceIn(0f, 1f) * 255f + 0.5f).toInt()
            val gi = (g.coerceIn(0f, 1f) * 255f + 0.5f).toInt()
            val bi = (b.coerceIn(0f, 1f) * 255f + 0.5f).toInt()
            return (0xFF shl 24) or (ri shl 16) or (gi shl 8) or bi
        }
        val w = 40
        val h = 40
        fun buildPortrait(wall: FloatArray, hair: FloatArray, skin: FloatArray, shirt: FloatArray): IntArray {
            val out = IntArray(w * h)
            for (y in 0 until h) {
                for (x in 0 until w) {
                    val c = if (x in 11..28 && y in 6..37) {
                        when {
                            y <= 11 -> hair
                            y <= 19 -> skin
                            else -> shirt
                        }
                    } else {
                        wall
                    }
                    out[y * w + x] = argb(c[0], c[1], c[2])
                }
            }
            return out
        }

        val refPx = buildPortrait(
            floatArrayOf(0.04f, 0.42f, 0.90f),
            floatArrayOf(0.07f, 0.06f, 0.06f),
            floatArrayOf(0.76f, 0.56f, 0.45f),
            floatArrayOf(0.62f, 0.50f, 0.36f)
        )
        val tgtPx = buildPortrait(
            floatArrayOf(0.05f, 0.36f, 0.80f),
            floatArrayOf(0.08f, 0.08f, 0.08f),
            floatArrayOf(0.70f, 0.50f, 0.39f),
            floatArrayOf(0.86f, 0.86f, 0.86f)
        )

        val matched = com.moestret.lightraw.engine.AiColorMatchEngine.matchPixelsToReference(
            refPixels = refPx, refW = w, refH = h,
            targetPixels = tgtPx, tgtW = w, tgtH = h,
            targetParams = DevelopParams(),
            forceRawFlat = null,
            refName = "Studio_Ref"
        )
        assertTrue(matched.hasColorMatchLut())
        assertTrue(matched.hasColorMatchBgLut())
        assertFalse(matched.matchRawFlat)

        val grid = com.moestret.lightraw.engine.AiColorMatchEngine.extractSemanticGrid(tgtPx, w, h)
        val outRgb = FloatArray(3)
        val t1 = FloatArray(3); val t2 = FloatArray(3); val t3 = FloatArray(3); val t4 = FloatArray(3)
        fun evalAt(nx: Float, ny: Float, r: Float, g: Float, b: Float): Triple<Float, Float, Float> {
            val sw = com.moestret.lightraw.engine.AiColorMatchEngine.sampleBilinear(
                grid.subjectWeights, grid.width, grid.height, nx, ny
            )
            com.moestret.lightraw.engine.AiColorMatchEngine.applyColorMatchLutSemantic(
                r, g, b,
                matched.matchLutData, matched.matchBgLutData, sw,
                matched.matchLutDim, 100f, 100f,
                t1, t2, t3, t4, outRgb
            )
            return Triple(outRgb[0], outRgb[1], outRgb[2])
        }

        // 1. Hair stays neutral dark (no cyan)
        val (hr, hg, hb) = evalAt(0.50f, 0.20f, 0.08f, 0.08f, 0.08f)
        assertTrue("Hair leaked cyan: ($hr, $hg, $hb)", hr < 0.22f && hg < 0.22f && hb < 0.24f && (hb - hr) < 0.06f)

        // 2. White shirt stays bright & neutral
        val (wr, wg, wb) = evalAt(0.50f, 0.72f, 0.86f, 0.86f, 0.86f)
        assertTrue("White shirt too dark: ($wr, $wg, $wb)", wr > 0.70f && wg > 0.70f && wb > 0.70f)
        assertTrue("White shirt stained: ($wr, $wg, $wb)", kotlin.math.abs(wr - wb) < 0.09f && kotlin.math.abs(wg - wb) < 0.08f)

        // 3. Face skin stays warm R > G > B
        val (sr, sg, sb) = evalAt(0.50f, 0.40f, 0.70f, 0.50f, 0.39f)
        assertTrue("Skin skewed: ($sr, $sg, $sb)", sr > sg && sg > sb && sr > 0.55f && (sr - sb) > 0.14f)

        // 4. Studio backdrop wall matches vivid cobalt blue
        val (br, _, bb) = evalAt(0.05f, 0.05f, 0.05f, 0.36f, 0.80f)
        assertTrue("Backdrop faded: ($br, $bb)", bb > 0.65f && bb > br + 0.45f)

        // 5. Base64 uint16 BG LUT encode/decode & CopySettingsMask preserve matchBgLutData
        val bgB64 = DevelopParams.encodeLutB64(matched.matchBgLutData)
        val decodedBg = DevelopParams.decodeLutB64(bgB64, matched.matchBgLutData.size)
        assertEquals(matched.matchBgLutData.size, decodedBg.size)
        assertEquals(matched.matchBgLutData[500], decodedBg[500], 0.001f)
        val copied = com.moestret.lightraw.model.CopySettingsMask(colorGrading = true)
            .applyTo(matched, DevelopParams())
        assertTrue(copied.hasColorMatchBgLut())
    }

    @Test
    fun xmpParsesAdobeLookProfileAndAsShotAnchorsWithoutNameCollision() {
        val xml = """
            <x:xmpmeta xmlns:x="adobe:ns:meta/">
             <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
              <rdf:Description rdf:about="" xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
               crs:PresetType="Normal"
               crs:WhiteBalance="Custom"
               crs:Temperature="3936"
               crs:Tint="+5"
               crs:AsShotTemperature="5050"
               crs:AsShotTint="5">
               <crs:Name><rdf:Alt><rdf:li xml:lang="x-default">XII SMK 26</rdf:li></rdf:Alt></crs:Name>
               <crs:Look>
                <rdf:Description crs:Name="Adobe Portrait" crs:Amount="1">
                 <crs:Group><rdf:Alt><rdf:li xml:lang="x-default">Profiles</rdf:li></rdf:Alt></crs:Group>
                </rdf:Description>
               </crs:Look>
              </rdf:Description>
             </rdf:RDF>
            </x:xmpmeta>
        """.trimIndent()
        val parsed = XmpPresetCodec.parseXmp(xml, "Fallback")!!
        assertEquals("XII SMK 26", parsed.name)
        assertEquals(com.moestret.lightraw.model.CameraProfile.PORTRAIT, parsed.params.cameraProfile)
        assertEquals(5050f, parsed.asShotTempRef!!, 0.5f)
        assertEquals(5f, parsed.asShotTintRef!!, 0.5f)
        assertEquals(3936f, parsed.params.temperature, 0.5f)

        val reExported = XmpPresetCodec.exportToXmp(parsed)
        val reparsed = XmpPresetCodec.parseXmp(reExported, "Fallback2")!!
        assertEquals(5050f, reparsed.asShotTempRef!!, 0.5f)
        assertEquals(5f, reparsed.asShotTintRef!!, 0.5f)
    }
}
