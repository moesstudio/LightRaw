package com.moestret.lightraw.data

import com.moestret.lightraw.model.CameraProfile
import com.moestret.lightraw.model.CurvePoint
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.DevelopPreset
import com.moestret.lightraw.model.WhiteBalanceMode
import java.util.Locale
import java.util.UUID

/**
 * Lossless Adobe Camera Raw `.xmp` preset parser & exporter for LightRaw Android,
 * schema-compatible with LightRaw Desktop `.xmp` files.
 */
object XmpPresetCodec {

    private fun extractAttr(xml: String, key: String): String? {
        val attrPattern = Regex("""\b$key\s*=\s*"([^"]*)"""")
        attrPattern.find(xml)?.let { return it.groupValues[1].trim() }
        val tagPattern = Regex("""<$key>(.*?)</$key>""", RegexOption.DOT_MATCHES_ALL)
        tagPattern.find(xml)?.let { return it.groupValues[1].trim() }
        return null
    }

    private fun extractFloat(xml: String, key: String, default: Float): Float =
        extractAttr(xml, key)?.replace(',', '.')?.toFloatOrNull() ?: default

    private fun extractCurve(xml: String, tagName: String): List<CurvePoint>? {
        val blockRegex = Regex("""<$tagName\b[^>]*>(.*?)</$tagName>""", RegexOption.DOT_MATCHES_ALL)
        val block = blockRegex.find(xml)?.groupValues?.getOrNull(1) ?: return null
        val liRegex = Regex("""<rdf:li>\s*([+-]?\d+(?:\.\d+)?)\s*,\s*([+-]?\d+(?:\.\d+)?)\s*</rdf:li>""")
        val pts = mutableListOf<CurvePoint>()
        for (m in liRegex.findAll(block)) {
            val x = m.groupValues[1].toFloatOrNull() ?: continue
            val y = m.groupValues[2].toFloatOrNull() ?: continue
            pts.add(CurvePoint((x / 255f).coerceIn(0f, 1f), (y / 255f).coerceIn(0f, 1f)))
        }
        return if (pts.size >= 2) pts.sortedBy { it.x } else null
    }

    private fun escapeXml(s: String): String = s
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")

    fun parseXmp(xml: String, fallbackName: String): DevelopPreset? {
        if (!xml.contains("crs:") && !xml.contains("xmpmeta")) return null

        val bandNames = arrayOf("Red", "Orange", "Yellow", "Green", "Aqua", "Blue", "Purple", "Magenta")
        val hueArr = FloatArray(8)
        val satArr = FloatArray(8)
        val lumArr = FloatArray(8)
        val grayArr = FloatArray(8)
        for (i in bandNames.indices) {
            hueArr[i] = extractFloat(xml, "crs:HueAdjustment${bandNames[i]}", 0f)
            satArr[i] = extractFloat(xml, "crs:SaturationAdjustment${bandNames[i]}", 0f)
            lumArr[i] = extractFloat(xml, "crs:LuminanceAdjustment${bandNames[i]}", 0f)
            grayArr[i] = extractFloat(xml, "crs:GrayMixer${bandNames[i]}", 0f)
        }

        val lookRegex = Regex("""<crs:Look\b[^>]*>.*?</crs:Look>""", RegexOption.DOT_MATCHES_ALL)
        val lookBlock = lookRegex.find(xml)?.value
        val mainXml = if (lookBlock != null) xml.replace(lookBlock, "") else xml

        val name = Regex("""<crs:Name>\s*<rdf:Alt>\s*<rdf:li[^>]*>([^<]+)</rdf:li>""", RegexOption.DOT_MATCHES_ALL)
            .find(mainXml)?.groupValues?.getOrNull(1)
            ?: extractAttr(mainXml, "crs:Name")
            ?: fallbackName

        val wbStr = extractAttr(mainXml, "crs:WhiteBalance") ?: "As Shot"
        val wbMode = WhiteBalanceMode.entries.firstOrNull {
            it.label.equals(wbStr, ignoreCase = true) || it.name.equals(wbStr, ignoreCase = true)
        } ?: WhiteBalanceMode.AS_SHOT

        val lookName = lookBlock?.let { extractAttr(it, "crs:Name") } ?: ""
        val profStr = extractAttr(mainXml, "crs:CameraProfile") ?: lookName
        val profile = when {
            profStr.contains("Vivid", ignoreCase = true) -> CameraProfile.VIVID
            profStr.contains("Landscape", ignoreCase = true) -> CameraProfile.LANDSCAPE
            profStr.contains("Portrait", ignoreCase = true) -> CameraProfile.PORTRAIT
            profStr.contains("Neutral", ignoreCase = true) -> CameraProfile.NEUTRAL
            profStr.contains("Monochrome", ignoreCase = true) -> CameraProfile.MONOCHROME
            profStr.contains("Film", ignoreCase = true) -> CameraProfile.FILM
            else -> CameraProfile.NATURAL
        }

        val asShotTempRef = extractAttr(mainXml, "crs:AsShotTemperature")?.replace(',', '.')?.toFloatOrNull()
            ?.takeIf { it in 1500f..50000f }
        val asShotTintRef = extractAttr(mainXml, "crs:AsShotTint")?.replace(',', '.')?.toFloatOrNull()
            ?.coerceIn(-150f, 150f)
        val nativeWb = extractAttr(mainXml, "crs:LightRawNativeWB").equals("True", ignoreCase = true)

        val params = DevelopParams(
            convertToGrayscale = extractAttr(mainXml, "crs:ConvertToGrayscale").equals("True", ignoreCase = true),
            cameraProfile = profile,
            whiteBalance = wbMode,
            temperature = extractFloat(mainXml, "crs:Temperature", 5500f).coerceIn(2000f, 50000f),
            tint = extractFloat(mainXml, "crs:Tint", 0f).coerceIn(-150f, 150f),
            exposure = extractFloat(mainXml, "crs:Exposure2012", 0f).coerceIn(-5f, 5f),
            contrast = extractFloat(mainXml, "crs:Contrast2012", 0f).coerceIn(-100f, 100f),
            highlights = extractFloat(mainXml, "crs:Highlights2012", 0f).coerceIn(-100f, 100f),
            shadows = extractFloat(mainXml, "crs:Shadows2012", 0f).coerceIn(-100f, 100f),
            whites = extractFloat(mainXml, "crs:Whites2012", 0f).coerceIn(-100f, 100f),
            blacks = extractFloat(mainXml, "crs:Blacks2012", 0f).coerceIn(-100f, 100f),
            texture = extractFloat(mainXml, "crs:Texture", 0f).coerceIn(-100f, 100f),
            clarity = extractFloat(mainXml, "crs:Clarity2012", 0f).coerceIn(-100f, 100f),
            dehaze = extractFloat(mainXml, "crs:Dehaze", 0f).coerceIn(-100f, 100f),
            vibrance = extractFloat(mainXml, "crs:Vibrance", 0f).coerceIn(-100f, 100f),
            saturation = extractFloat(mainXml, "crs:Saturation", 0f).coerceIn(-100f, 100f),
            parametricHighlights = extractFloat(mainXml, "crs:ParametricHighlights", 0f),
            parametricLights = extractFloat(mainXml, "crs:ParametricLights", 0f),
            parametricDarks = extractFloat(mainXml, "crs:ParametricDarks", 0f),
            parametricShadows = extractFloat(mainXml, "crs:ParametricShadows", 0f),
            toneCurveMaster = extractCurve(mainXml, "crs:ToneCurvePV2012") ?: DevelopParams.defaultLinearCurve(),
            toneCurveRed = extractCurve(mainXml, "crs:ToneCurvePV2012Red") ?: DevelopParams.defaultLinearCurve(),
            toneCurveGreen = extractCurve(mainXml, "crs:ToneCurvePV2012Green") ?: DevelopParams.defaultLinearCurve(),
            toneCurveBlue = extractCurve(mainXml, "crs:ToneCurvePV2012Blue") ?: DevelopParams.defaultLinearCurve(),
            hueAdjustment = hueArr,
            saturationAdjustment = satArr,
            luminanceAdjustment = lumArr,
            grayMixer = grayArr,
            cgShadowHue = extractFloat(mainXml, "crs:SplitToningShadowHue", 0f),
            cgShadowSat = extractFloat(mainXml, "crs:SplitToningShadowSaturation", 0f),
            cgShadowLum = extractFloat(mainXml, "crs:ColorGradeShadowLum", 0f),
            cgMidHue = extractFloat(mainXml, "crs:ColorGradeMidtoneHue", 0f),
            cgMidSat = extractFloat(mainXml, "crs:ColorGradeMidtoneSat", 0f),
            cgMidLum = extractFloat(mainXml, "crs:ColorGradeMidtoneLum", 0f),
            cgHighHue = extractFloat(mainXml, "crs:SplitToningHighlightHue", 0f),
            cgHighSat = extractFloat(mainXml, "crs:SplitToningHighlightSaturation", 0f),
            cgHighLum = extractFloat(mainXml, "crs:ColorGradeHighlightLum", 0f),
            cgGlobalHue = extractFloat(mainXml, "crs:ColorGradeGlobalHue", 0f),
            cgGlobalSat = extractFloat(mainXml, "crs:ColorGradeGlobalSat", 0f),
            cgGlobalLum = extractFloat(mainXml, "crs:ColorGradeGlobalLum", 0f),
            cgBlending = extractFloat(mainXml, "crs:ColorGradeBlending", 50f),
            cgBalance = extractFloat(mainXml, "crs:SplitToningBalance", 0f),
            sharpness = extractFloat(mainXml, "crs:Sharpness", 20f).coerceIn(0f, 150f),
            sharpenRadius = extractFloat(mainXml, "crs:SharpenRadius", 1.0f).coerceIn(0.5f, 3.0f),
            luminanceNr = extractFloat(mainXml, "crs:LuminanceSmoothing", 0f).coerceIn(0f, 100f),
            colorNr = extractFloat(mainXml, "crs:ColorNoiseReduction", 15f).coerceIn(0f, 100f),
            vignetteAmount = extractFloat(mainXml, "crs:PostCropVignetteAmount", 0f).coerceIn(-100f, 100f),
            vignetteMidpoint = extractFloat(mainXml, "crs:PostCropVignetteMidpoint", 50f).coerceIn(0f, 100f),
            vignetteRoundness = extractFloat(mainXml, "crs:PostCropVignetteRoundness", 0f).coerceIn(-100f, 100f),
            vignetteFeather = extractFloat(mainXml, "crs:PostCropVignetteFeather", 50f).coerceIn(0f, 100f),
            grainAmount = extractFloat(mainXml, "crs:GrainAmount", 0f).coerceIn(0f, 100f),
            grainSize = extractFloat(mainXml, "crs:GrainSize", 25f).coerceIn(0f, 100f),
            grainRoughness = extractFloat(mainXml, "crs:GrainFrequency", 50f).coerceIn(0f, 100f),
            lensDistortion = extractFloat(mainXml, "crs:LensManualDistortionAmount", 0f).coerceIn(-100f, 100f),
            perspectiveVertical = extractFloat(mainXml, "crs:PerspectiveVertical", 0f).coerceIn(-100f, 100f),
            perspectiveHorizontal = extractFloat(mainXml, "crs:PerspectiveHorizontal", 0f).coerceIn(-100f, 100f),
            perspectiveAspect = extractFloat(mainXml, "crs:PerspectiveAspect", 0f).coerceIn(-100f, 100f),
            geometryScale = extractFloat(mainXml, "crs:PerspectiveScale", 100f).coerceIn(50f, 150f),
            geometryOffsetX = extractFloat(mainXml, "crs:PerspectiveX", 0f).coerceIn(-100f, 100f),
            geometryOffsetY = extractFloat(mainXml, "crs:PerspectiveY", 0f).coerceIn(-100f, 100f)
        )

        return DevelopPreset(
            id = "xmp_${UUID.randomUUID()}",
            category = "Imported XMP",
            name = name.trim().ifEmpty { fallbackName },
            isUser = true,
            isFavorite = false,
            params = params,
            asShotTempRef = asShotTempRef,
            asShotTintRef = asShotTintRef,
            nativeLightRawWb = nativeWb
        )
    }

    fun exportToXmp(preset: DevelopPreset): String {
        val p = preset.params
        val bandNames = arrayOf("Red", "Orange", "Yellow", "Green", "Aqua", "Blue", "Purple", "Magenta")
        val sb = StringBuilder()
        sb.appendLine("""<x:xmpmeta xmlns:x="adobe:ns:meta/">""")
        sb.appendLine(""" <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">""")
        sb.appendLine("""  <rdf:Description rdf:about="" xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"""")
        sb.appendLine("""   crs:PresetType="Normal"""")
        sb.appendLine("""   crs:Version="17.0"""")
        sb.appendLine("""   crs:ProcessVersion="15.4"""")
        sb.appendLine("""   crs:ConvertToGrayscale="${if (p.convertToGrayscale) "True" else "False"}"""")
        sb.appendLine("""   crs:CameraProfile="${escapeXml(p.cameraProfile.label)}"""")
        sb.appendLine("""   crs:WhiteBalance="${escapeXml(p.whiteBalance.label)}"""")
        sb.appendLine("""   crs:Temperature="${p.temperature.toInt()}"""")
        sb.appendLine("""   crs:Tint="${p.tint.toInt()}"""")
        preset.asShotTempRef?.let {
            sb.appendLine("""   crs:AsShotTemperature="${it.toInt()}"""")
        }
        preset.asShotTintRef?.let {
            sb.appendLine("""   crs:AsShotTint="${it.toInt()}"""")
        }
        if (preset.nativeLightRawWb) {
            sb.appendLine("""   crs:LightRawNativeWB="True"""")
        }
        sb.appendLine("""   crs:Exposure2012="${"%.2f".format(Locale.US, p.exposure)}"""")
        sb.appendLine("""   crs:Contrast2012="${p.contrast.toInt()}"""")
        sb.appendLine("""   crs:Highlights2012="${p.highlights.toInt()}"""")
        sb.appendLine("""   crs:Shadows2012="${p.shadows.toInt()}"""")
        sb.appendLine("""   crs:Whites2012="${p.whites.toInt()}"""")
        sb.appendLine("""   crs:Blacks2012="${p.blacks.toInt()}"""")
        sb.appendLine("""   crs:Texture="${p.texture.toInt()}"""")
        sb.appendLine("""   crs:Clarity2012="${p.clarity.toInt()}"""")
        sb.appendLine("""   crs:Dehaze="${p.dehaze.toInt()}"""")
        sb.appendLine("""   crs:Vibrance="${p.vibrance.toInt()}"""")
        sb.appendLine("""   crs:Saturation="${p.saturation.toInt()}"""")
        sb.appendLine("""   crs:ParametricHighlights="${p.parametricHighlights.toInt()}"""")
        sb.appendLine("""   crs:ParametricLights="${p.parametricLights.toInt()}"""")
        sb.appendLine("""   crs:ParametricDarks="${p.parametricDarks.toInt()}"""")
        sb.appendLine("""   crs:ParametricShadows="${p.parametricShadows.toInt()}"""")
        for (i in bandNames.indices) {
            sb.appendLine("""   crs:HueAdjustment${bandNames[i]}="${p.hueAdjustment[i].toInt()}"""")
            sb.appendLine("""   crs:SaturationAdjustment${bandNames[i]}="${p.saturationAdjustment[i].toInt()}"""")
            sb.appendLine("""   crs:LuminanceAdjustment${bandNames[i]}="${p.luminanceAdjustment[i].toInt()}"""")
            sb.appendLine("""   crs:GrayMixer${bandNames[i]}="${p.grayMixer[i].toInt()}"""")
        }
        sb.appendLine("""   crs:SplitToningShadowHue="${p.cgShadowHue.toInt()}"""")
        sb.appendLine("""   crs:SplitToningShadowSaturation="${p.cgShadowSat.toInt()}"""")
        sb.appendLine("""   crs:ColorGradeShadowLum="${p.cgShadowLum.toInt()}"""")
        sb.appendLine("""   crs:ColorGradeMidtoneHue="${p.cgMidHue.toInt()}"""")
        sb.appendLine("""   crs:ColorGradeMidtoneSat="${p.cgMidSat.toInt()}"""")
        sb.appendLine("""   crs:ColorGradeMidtoneLum="${p.cgMidLum.toInt()}"""")
        sb.appendLine("""   crs:SplitToningHighlightHue="${p.cgHighHue.toInt()}"""")
        sb.appendLine("""   crs:SplitToningHighlightSaturation="${p.cgHighSat.toInt()}"""")
        sb.appendLine("""   crs:ColorGradeHighlightLum="${p.cgHighLum.toInt()}"""")
        sb.appendLine("""   crs:ColorGradeGlobalHue="${p.cgGlobalHue.toInt()}"""")
        sb.appendLine("""   crs:ColorGradeGlobalSat="${p.cgGlobalSat.toInt()}"""")
        sb.appendLine("""   crs:ColorGradeGlobalLum="${p.cgGlobalLum.toInt()}"""")
        sb.appendLine("""   crs:ColorGradeBlending="${p.cgBlending.toInt()}"""")
        sb.appendLine("""   crs:SplitToningBalance="${p.cgBalance.toInt()}"""")
        sb.appendLine("""   crs:Sharpness="${p.sharpness.toInt()}"""")
        sb.appendLine("""   crs:SharpenRadius="${"%.2f".format(Locale.US, p.sharpenRadius)}"""")
        sb.appendLine("""   crs:LuminanceSmoothing="${p.luminanceNr.toInt()}"""")
        sb.appendLine("""   crs:ColorNoiseReduction="${p.colorNr.toInt()}"""")
        sb.appendLine("""   crs:PostCropVignetteAmount="${p.vignetteAmount.toInt()}"""")
        sb.appendLine("""   crs:PostCropVignetteMidpoint="${p.vignetteMidpoint.toInt()}"""")
        sb.appendLine("""   crs:PostCropVignetteRoundness="${p.vignetteRoundness.toInt()}"""")
        sb.appendLine("""   crs:PostCropVignetteFeather="${p.vignetteFeather.toInt()}"""")
        sb.appendLine("""   crs:GrainAmount="${p.grainAmount.toInt()}"""")
        sb.appendLine("""   crs:GrainSize="${p.grainSize.toInt()}"""")
        sb.appendLine("""   crs:GrainFrequency="${p.grainRoughness.toInt()}"""")
        sb.appendLine("""   crs:LensManualDistortionAmount="${p.lensDistortion.toInt()}"""")
        sb.appendLine("""   crs:PerspectiveVertical="${p.perspectiveVertical.toInt()}"""")
        sb.appendLine("""   crs:PerspectiveHorizontal="${p.perspectiveHorizontal.toInt()}"""")
        sb.appendLine("""   crs:PerspectiveAspect="${p.perspectiveAspect.toInt()}"""")
        sb.appendLine("""   crs:PerspectiveScale="${p.geometryScale.toInt()}"""")
        sb.appendLine("""   crs:PerspectiveX="${p.geometryOffsetX.toInt()}"""")
        sb.appendLine("""   crs:PerspectiveY="${p.geometryOffsetY.toInt()}"""")
        sb.appendLine("""   >""")
        sb.appendLine("""   <crs:Name><rdf:Alt><rdf:li xml:lang="x-default">${escapeXml(preset.name)}</rdf:li></rdf:Alt></crs:Name>""")
        appendCurveXml(sb, "crs:ToneCurvePV2012", p.toneCurveMaster)
        appendCurveXml(sb, "crs:ToneCurvePV2012Red", p.toneCurveRed)
        appendCurveXml(sb, "crs:ToneCurvePV2012Green", p.toneCurveGreen)
        appendCurveXml(sb, "crs:ToneCurvePV2012Blue", p.toneCurveBlue)
        sb.appendLine("""  </rdf:Description>""")
        sb.appendLine(""" </rdf:RDF>""")
        sb.appendLine("""</x:xmpmeta>""")
        return sb.toString()
    }

    private fun appendCurveXml(sb: StringBuilder, tag: String, pts: List<CurvePoint>) {
        sb.appendLine("""   <$tag><rdf:Seq>""")
        for (pt in pts) {
            val x = (pt.x * 255f).toInt().coerceIn(0, 255)
            val y = (pt.y * 255f).toInt().coerceIn(0, 255)
            sb.appendLine("""    <rdf:li>$x, $y</rdf:li>""")
        }
        sb.appendLine("""   </rdf:Seq></$tag>""")
    }
}
