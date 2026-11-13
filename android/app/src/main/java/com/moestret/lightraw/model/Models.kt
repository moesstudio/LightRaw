package com.moestret.lightraw.model

import org.json.JSONArray
import org.json.JSONObject
import java.util.UUID

enum class WhiteBalanceMode(val label: String) {
    AS_SHOT("As Shot"),
    AUTO("Auto"),
    DAYLIGHT("Daylight"),
    CLOUDY("Cloudy"),
    SHADE("Shade"),
    TUNGSTEN("Tungsten"),
    FLUORESCENT("Fluorescent"),
    CUSTOM("Custom");

    fun defaultTempTint(): Pair<Float, Float>? = when (this) {
        AS_SHOT -> 5500f to 0f
        AUTO -> null
        DAYLIGHT -> 5500f to 10f
        CLOUDY -> 6500f to 10f
        SHADE -> 7500f to 10f
        TUNGSTEN -> 2850f to 0f
        FLUORESCENT -> 3800f to 21f
        CUSTOM -> null
    }
}

enum class CameraProfile(val label: String) {
    NATURAL("LightRaw Natural"),
    VIVID("LightRaw Vivid"),
    LANDSCAPE("LightRaw Landscape"),
    PORTRAIT("LightRaw Portrait"),
    NEUTRAL("LightRaw Neutral"),
    MONOCHROME("LightRaw Monochrome"),
    FILM("Film Positive")
}

enum class AspectRatioPreset(val label: String, val ratio: Float?) {
    ORIGINAL("Original", null),
    FREE("Free", null),
    R_1_1("1:1", 1f),
    R_4_5("4:5", 4f / 5f),
    R_2_3("2:3", 2f / 3f),
    R_3_4("3:4", 3f / 4f),
    R_9_16("9:16", 9f / 16f),
    R_16_9("16:9", 16f / 9f)
}

enum class PickFlag(val code: Int) {
    PICKED(1),
    UNFLAGGED(0),
    REJECTED(-1);

    companion object {
        fun fromCode(code: Int): PickFlag = entries.firstOrNull { it.code == code } ?: UNFLAGGED
    }
}

enum class ColorLabel(val code: String, val colorHex: Int) {
    NONE("", 0x00000000),
    RED("Red", 0xFFEF4444.toInt()),
    YELLOW("Yellow", 0xFFF59E0B.toInt()),
    GREEN("Green", 0xFF10B981.toInt()),
    BLUE("Blue", 0xFF3B82F6.toInt()),
    PURPLE("Purple", 0xFFA855F7.toInt());

    companion object {
        fun fromCode(code: String?): ColorLabel =
            entries.firstOrNull { it.code.equals(code ?: "", ignoreCase = true) } ?: NONE
    }
}

data class CurvePoint(val x: Float, val y: Float)

enum class MaskPrimitiveKind {
    LINEAR_GRADIENT,
    RADIAL_GRADIENT,
    BRUSH
}

data class LocalAdjustParams(
    val exposure: Float = 0f,
    val contrast: Float = 0f,
    val highlights: Float = 0f,
    val shadows: Float = 0f,
    val temperature: Float = 0f,
    val tint: Float = 0f,
    val clarity: Float = 0f,
    val saturation: Float = 0f
)

data class MaskGroup(
    val id: String = UUID.randomUUID().toString(),
    val name: String = "Mask",
    val kind: MaskPrimitiveKind = MaskPrimitiveKind.RADIAL_GRADIENT,
    val invert: Boolean = false,
    val feather: Float = 50f,
    val x1: Float = 0.5f,
    val y1: Float = 0.5f,
    val x2: Float = 0.5f,
    val y2: Float = 0.85f,
    val radiusX: Float = 0.28f,
    val radiusY: Float = 0.28f,
    val brushPoints: List<CurvePoint> = emptyList(),
    val adjust: LocalAdjustParams = LocalAdjustParams()
)

data class RetouchArea(
    val id: String = UUID.randomUUID().toString(),
    val x: Float,
    val y: Float,
    val radius: Float = 0.035f,
    val opacity: Float = 100f
)

data class DevelopParams(
    val convertToGrayscale: Boolean = false,
    val cameraProfile: CameraProfile = CameraProfile.NATURAL,
    val lookAmount: Float = 100f,
    val whiteBalance: WhiteBalanceMode = WhiteBalanceMode.AS_SHOT,
    val temperature: Float = 5500f,
    val tint: Float = 0f,
    val exposure: Float = 0f,
    val contrast: Float = 0f,
    val highlights: Float = 0f,
    val shadows: Float = 0f,
    val whites: Float = 0f,
    val blacks: Float = 0f,
    val texture: Float = 0f,
    val clarity: Float = 0f,
    val dehaze: Float = 0f,
    val vibrance: Float = 0f,
    val saturation: Float = 0f,
    val parametricHighlights: Float = 0f,
    val parametricLights: Float = 0f,
    val parametricDarks: Float = 0f,
    val parametricShadows: Float = 0f,
    val toneCurveMaster: List<CurvePoint> = defaultLinearCurve(),
    val toneCurveRed: List<CurvePoint> = defaultLinearCurve(),
    val toneCurveGreen: List<CurvePoint> = defaultLinearCurve(),
    val toneCurveBlue: List<CurvePoint> = defaultLinearCurve(),
    // 8 bands: Red, Orange, Yellow, Green, Aqua, Blue, Purple, Magenta
    val hueAdjustment: FloatArray = FloatArray(8),
    val saturationAdjustment: FloatArray = FloatArray(8),
    val luminanceAdjustment: FloatArray = FloatArray(8),
    val grayMixer: FloatArray = floatArrayOf(0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f),
    // Color Grading & Split Toning
    val cgShadowHue: Float = 0f,
    val cgShadowSat: Float = 0f,
    val cgShadowLum: Float = 0f,
    val cgMidHue: Float = 0f,
    val cgMidSat: Float = 0f,
    val cgMidLum: Float = 0f,
    val cgHighHue: Float = 0f,
    val cgHighSat: Float = 0f,
    val cgHighLum: Float = 0f,
    val cgGlobalHue: Float = 0f,
    val cgGlobalSat: Float = 0f,
    val cgGlobalLum: Float = 0f,
    val cgBlending: Float = 50f,
    val cgBalance: Float = 0f,
    // Detail & Optics & Blur
    val sharpness: Float = 20f,
    val sharpenRadius: Float = 1.0f,
    val luminanceNr: Float = 0f,
    val colorNr: Float = 15f,
    val lensDistortion: Float = 0f,
    val lensVignetting: Float = 0f,
    val lensCa: Float = 0f,
    val lensBlurAmount: Float = 0f,
    val lensBlurFocalCenter: Float = 50f,
    // Effects
    val vignetteAmount: Float = 0f,
    val vignetteMidpoint: Float = 50f,
    val vignetteRoundness: Float = 0f,
    val vignetteFeather: Float = 50f,
    val grainAmount: Float = 0f,
    val grainSize: Float = 25f,
    val grainRoughness: Float = 50f,
    // Geometry & Crop
    val cropLeft: Float = 0f,
    val cropTop: Float = 0f,
    val cropRight: Float = 1f,
    val cropBottom: Float = 1f,
    val cropAngle: Float = 0f,
    val cropAspect: AspectRatioPreset = AspectRatioPreset.ORIGINAL,
    val rotationDegrees: Float = 0f,
    val flipHorizontal: Boolean = false,
    val flipVertical: Boolean = false,
    val perspectiveVertical: Float = 0f,
    val perspectiveHorizontal: Float = 0f,
    val perspectiveAspect: Float = 0f,
    val geometryScale: Float = 100f,
    val geometryOffsetX: Float = 0f,
    val geometryOffsetY: Float = 0f,
    // Selective Local Edits & Spot Heal
    val masks: List<MaskGroup> = emptyList(),
    val retouchSpots: List<RetouchArea> = emptyList(),
    val appliedPresetName: String = "",
    val presetAmount: Float = 100f
) {
    fun deepCopy(): DevelopParams = copy(
        toneCurveMaster = toneCurveMaster.toList(),
        toneCurveRed = toneCurveRed.toList(),
        toneCurveGreen = toneCurveGreen.toList(),
        toneCurveBlue = toneCurveBlue.toList(),
        hueAdjustment = hueAdjustment.copyOf(),
        saturationAdjustment = saturationAdjustment.copyOf(),
        luminanceAdjustment = luminanceAdjustment.copyOf(),
        grayMixer = grayMixer.copyOf(),
        masks = masks.map { it.copy(brushPoints = it.brushPoints.toList()) },
        retouchSpots = retouchSpots.toList()
    )

    fun toJsonString(): String {
        val o = JSONObject()
        o.put("convertToGrayscale", convertToGrayscale)
        o.put("cameraProfile", cameraProfile.name)
        o.put("lookAmount", lookAmount.toDouble())
        o.put("whiteBalance", whiteBalance.name)
        o.put("temperature", temperature.toDouble())
        o.put("tint", tint.toDouble())
        o.put("exposure", exposure.toDouble())
        o.put("contrast", contrast.toDouble())
        o.put("highlights", highlights.toDouble())
        o.put("shadows", shadows.toDouble())
        o.put("whites", whites.toDouble())
        o.put("blacks", blacks.toDouble())
        o.put("texture", texture.toDouble())
        o.put("clarity", clarity.toDouble())
        o.put("dehaze", dehaze.toDouble())
        o.put("vibrance", vibrance.toDouble())
        o.put("saturation", saturation.toDouble())
        o.put("parametricHighlights", parametricHighlights.toDouble())
        o.put("parametricLights", parametricLights.toDouble())
        o.put("parametricDarks", parametricDarks.toDouble())
        o.put("parametricShadows", parametricShadows.toDouble())
        o.put("toneCurveMaster", encodeCurve(toneCurveMaster))
        o.put("toneCurveRed", encodeCurve(toneCurveRed))
        o.put("toneCurveGreen", encodeCurve(toneCurveGreen))
        o.put("toneCurveBlue", encodeCurve(toneCurveBlue))
        o.put("hueAdjustment", encodeFloatArray(hueAdjustment))
        o.put("saturationAdjustment", encodeFloatArray(saturationAdjustment))
        o.put("luminanceAdjustment", encodeFloatArray(luminanceAdjustment))
        o.put("grayMixer", encodeFloatArray(grayMixer))
        o.put("cgShadowHue", cgShadowHue.toDouble())
        o.put("cgShadowSat", cgShadowSat.toDouble())
        o.put("cgShadowLum", cgShadowLum.toDouble())
        o.put("cgMidHue", cgMidHue.toDouble())
        o.put("cgMidSat", cgMidSat.toDouble())
        o.put("cgMidLum", cgMidLum.toDouble())
        o.put("cgHighHue", cgHighHue.toDouble())
        o.put("cgHighSat", cgHighSat.toDouble())
        o.put("cgHighLum", cgHighLum.toDouble())
        o.put("cgGlobalHue", cgGlobalHue.toDouble())
        o.put("cgGlobalSat", cgGlobalSat.toDouble())
        o.put("cgGlobalLum", cgGlobalLum.toDouble())
        o.put("cgBlending", cgBlending.toDouble())
        o.put("cgBalance", cgBalance.toDouble())
        o.put("sharpness", sharpness.toDouble())
        o.put("sharpenRadius", sharpenRadius.toDouble())
        o.put("luminanceNr", luminanceNr.toDouble())
        o.put("colorNr", colorNr.toDouble())
        o.put("lensDistortion", lensDistortion.toDouble())
        o.put("lensVignetting", lensVignetting.toDouble())
        o.put("lensCa", lensCa.toDouble())
        o.put("lensBlurAmount", lensBlurAmount.toDouble())
        o.put("lensBlurFocalCenter", lensBlurFocalCenter.toDouble())
        o.put("vignetteAmount", vignetteAmount.toDouble())
        o.put("vignetteMidpoint", vignetteMidpoint.toDouble())
        o.put("vignetteRoundness", vignetteRoundness.toDouble())
        o.put("vignetteFeather", vignetteFeather.toDouble())
        o.put("grainAmount", grainAmount.toDouble())
        o.put("grainSize", grainSize.toDouble())
        o.put("grainRoughness", grainRoughness.toDouble())
        o.put("cropLeft", cropLeft.toDouble())
        o.put("cropTop", cropTop.toDouble())
        o.put("cropRight", cropRight.toDouble())
        o.put("cropBottom", cropBottom.toDouble())
        o.put("cropAngle", cropAngle.toDouble())
        o.put("cropAspect", cropAspect.name)
        o.put("rotationDegrees", rotationDegrees.toDouble())
        o.put("flipHorizontal", flipHorizontal)
        o.put("flipVertical", flipVertical)
        o.put("perspectiveVertical", perspectiveVertical.toDouble())
        o.put("perspectiveHorizontal", perspectiveHorizontal.toDouble())
        o.put("perspectiveAspect", perspectiveAspect.toDouble())
        o.put("geometryScale", geometryScale.toDouble())
        o.put("geometryOffsetX", geometryOffsetX.toDouble())
        o.put("geometryOffsetY", geometryOffsetY.toDouble())
        val mArr = JSONArray()
        for (m in masks) {
            val mo = JSONObject()
            mo.put("id", m.id)
            mo.put("name", m.name)
            mo.put("kind", m.kind.name)
            mo.put("invert", m.invert)
            mo.put("feather", m.feather.toDouble())
            mo.put("x1", m.x1.toDouble())
            mo.put("y1", m.y1.toDouble())
            mo.put("x2", m.x2.toDouble())
            mo.put("y2", m.y2.toDouble())
            mo.put("radiusX", m.radiusX.toDouble())
            mo.put("radiusY", m.radiusY.toDouble())
            mo.put("brushPoints", encodeCurve(m.brushPoints))
            mo.put("exposure", m.adjust.exposure.toDouble())
            mo.put("contrast", m.adjust.contrast.toDouble())
            mo.put("highlights", m.adjust.highlights.toDouble())
            mo.put("shadows", m.adjust.shadows.toDouble())
            mo.put("temperature", m.adjust.temperature.toDouble())
            mo.put("tint", m.adjust.tint.toDouble())
            mo.put("clarity", m.adjust.clarity.toDouble())
            mo.put("saturation", m.adjust.saturation.toDouble())
            mArr.put(mo)
        }
        o.put("masks", mArr)
        val rArr = JSONArray()
        for (r in retouchSpots) {
            val ro = JSONObject()
            ro.put("id", r.id)
            ro.put("x", r.x.toDouble())
            ro.put("y", r.y.toDouble())
            ro.put("radius", r.radius.toDouble())
            ro.put("opacity", r.opacity.toDouble())
            rArr.put(ro)
        }
        o.put("retouchSpots", rArr)
        o.put("appliedPresetName", appliedPresetName)
        o.put("presetAmount", presetAmount.toDouble())
        return o.toString()
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is DevelopParams) return false
        return toJsonString() == other.toJsonString()
    }

    override fun hashCode(): Int = toJsonString().hashCode()

    companion object {
        fun defaultLinearCurve(): List<CurvePoint> = listOf(CurvePoint(0f, 0f), CurvePoint(1f, 1f))

        private fun encodeCurve(pts: List<CurvePoint>): JSONArray {
            val arr = JSONArray()
            for (p in pts) {
                arr.put(JSONArray().apply {
                    put(p.x.toDouble())
                    put(p.y.toDouble())
                })
            }
            return arr
        }

        private fun decodeCurve(arr: JSONArray?, fallback: List<CurvePoint>): List<CurvePoint> {
            if (arr == null || arr.length() == 0) return fallback
            val out = mutableListOf<CurvePoint>()
            for (i in 0 until arr.length()) {
                val p = arr.optJSONArray(i) ?: continue
                if (p.length() >= 2) {
                    out.add(CurvePoint(p.optDouble(0, 0.0).toFloat(), p.optDouble(1, 0.0).toFloat()))
                }
            }
            return if (out.isEmpty()) fallback else out.sortedBy { it.x }
        }

        private fun encodeFloatArray(arr: FloatArray): JSONArray =
            JSONArray().apply { arr.forEach { put(it.toDouble()) } }

        private fun decodeFloatArray(arr: JSONArray?, size: Int): FloatArray {
            val out = FloatArray(size)
            if (arr == null) return out
            for (i in 0 until minOf(size, arr.length())) {
                out[i] = arr.optDouble(i, 0.0).toFloat()
            }
            return out
        }

        fun fromJsonString(json: String?): DevelopParams {
            if (json.isNullOrBlank()) return DevelopParams()
            return try {
                val o = JSONObject(json)
                val maskList = mutableListOf<MaskGroup>()
                val mArr = o.optJSONArray("masks")
                if (mArr != null) {
                    for (i in 0 until mArr.length()) {
                        val mo = mArr.optJSONObject(i) ?: continue
                        val kind = runCatching {
                            MaskPrimitiveKind.valueOf(mo.optString("kind", "RADIAL_GRADIENT"))
                        }.getOrDefault(MaskPrimitiveKind.RADIAL_GRADIENT)
                        maskList.add(
                            MaskGroup(
                                id = mo.optString("id", UUID.randomUUID().toString()),
                                name = mo.optString("name", "Mask ${i + 1}"),
                                kind = kind,
                                invert = mo.optBoolean("invert", false),
                                feather = mo.optDouble("feather", 50.0).toFloat(),
                                x1 = mo.optDouble("x1", 0.5).toFloat(),
                                y1 = mo.optDouble("y1", 0.5).toFloat(),
                                x2 = mo.optDouble("x2", 0.5).toFloat(),
                                y2 = mo.optDouble("y2", 0.85).toFloat(),
                                radiusX = mo.optDouble("radiusX", 0.28).toFloat(),
                                radiusY = mo.optDouble("radiusY", 0.28).toFloat(),
                                brushPoints = decodeCurve(mo.optJSONArray("brushPoints"), emptyList()),
                                adjust = LocalAdjustParams(
                                    exposure = mo.optDouble("exposure", 0.0).toFloat(),
                                    contrast = mo.optDouble("contrast", 0.0).toFloat(),
                                    highlights = mo.optDouble("highlights", 0.0).toFloat(),
                                    shadows = mo.optDouble("shadows", 0.0).toFloat(),
                                    temperature = mo.optDouble("temperature", 0.0).toFloat(),
                                    tint = mo.optDouble("tint", 0.0).toFloat(),
                                    clarity = mo.optDouble("clarity", 0.0).toFloat(),
                                    saturation = mo.optDouble("saturation", 0.0).toFloat()
                                )
                            )
                        )
                    }
                }

                val retouchList = mutableListOf<RetouchArea>()
                val rArr = o.optJSONArray("retouchSpots")
                if (rArr != null) {
                    for (i in 0 until rArr.length()) {
                        val ro = rArr.optJSONObject(i) ?: continue
                        retouchList.add(
                            RetouchArea(
                                id = ro.optString("id", UUID.randomUUID().toString()),
                                x = ro.optDouble("x", 0.5).toFloat(),
                                y = ro.optDouble("y", 0.5).toFloat(),
                                radius = ro.optDouble("radius", 0.035).toFloat(),
                                opacity = ro.optDouble("opacity", 100.0).toFloat()
                            )
                        )
                    }
                }

                DevelopParams(
                    convertToGrayscale = o.optBoolean("convertToGrayscale", false),
                    cameraProfile = runCatching {
                        CameraProfile.valueOf(o.optString("cameraProfile", "NATURAL"))
                    }.getOrDefault(CameraProfile.NATURAL),
                    lookAmount = o.optDouble("lookAmount", 100.0).toFloat(),
                    whiteBalance = runCatching {
                        WhiteBalanceMode.valueOf(o.optString("whiteBalance", "AS_SHOT"))
                    }.getOrDefault(WhiteBalanceMode.AS_SHOT),
                    temperature = o.optDouble("temperature", 5500.0).toFloat(),
                    tint = o.optDouble("tint", 0.0).toFloat(),
                    exposure = o.optDouble("exposure", 0.0).toFloat(),
                    contrast = o.optDouble("contrast", 0.0).toFloat(),
                    highlights = o.optDouble("highlights", 0.0).toFloat(),
                    shadows = o.optDouble("shadows", 0.0).toFloat(),
                    whites = o.optDouble("whites", 0.0).toFloat(),
                    blacks = o.optDouble("blacks", 0.0).toFloat(),
                    texture = o.optDouble("texture", 0.0).toFloat(),
                    clarity = o.optDouble("clarity", 0.0).toFloat(),
                    dehaze = o.optDouble("dehaze", 0.0).toFloat(),
                    vibrance = o.optDouble("vibrance", 0.0).toFloat(),
                    saturation = o.optDouble("saturation", 0.0).toFloat(),
                    parametricHighlights = o.optDouble("parametricHighlights", 0.0).toFloat(),
                    parametricLights = o.optDouble("parametricLights", 0.0).toFloat(),
                    parametricDarks = o.optDouble("parametricDarks", 0.0).toFloat(),
                    parametricShadows = o.optDouble("parametricShadows", 0.0).toFloat(),
                    toneCurveMaster = decodeCurve(o.optJSONArray("toneCurveMaster"), defaultLinearCurve()),
                    toneCurveRed = decodeCurve(o.optJSONArray("toneCurveRed"), defaultLinearCurve()),
                    toneCurveGreen = decodeCurve(o.optJSONArray("toneCurveGreen"), defaultLinearCurve()),
                    toneCurveBlue = decodeCurve(o.optJSONArray("toneCurveBlue"), defaultLinearCurve()),
                    hueAdjustment = decodeFloatArray(o.optJSONArray("hueAdjustment"), 8),
                    saturationAdjustment = decodeFloatArray(o.optJSONArray("saturationAdjustment"), 8),
                    luminanceAdjustment = decodeFloatArray(o.optJSONArray("luminanceAdjustment"), 8),
                    grayMixer = decodeFloatArray(o.optJSONArray("grayMixer"), 8),
                    cgShadowHue = o.optDouble("cgShadowHue", 0.0).toFloat(),
                    cgShadowSat = o.optDouble("cgShadowSat", 0.0).toFloat(),
                    cgShadowLum = o.optDouble("cgShadowLum", 0.0).toFloat(),
                    cgMidHue = o.optDouble("cgMidHue", 0.0).toFloat(),
                    cgMidSat = o.optDouble("cgMidSat", 0.0).toFloat(),
                    cgMidLum = o.optDouble("cgMidLum", 0.0).toFloat(),
                    cgHighHue = o.optDouble("cgHighHue", 0.0).toFloat(),
                    cgHighSat = o.optDouble("cgHighSat", 0.0).toFloat(),
                    cgHighLum = o.optDouble("cgHighLum", 0.0).toFloat(),
                    cgGlobalHue = o.optDouble("cgGlobalHue", 0.0).toFloat(),
                    cgGlobalSat = o.optDouble("cgGlobalSat", 0.0).toFloat(),
                    cgGlobalLum = o.optDouble("cgGlobalLum", 0.0).toFloat(),
                    cgBlending = o.optDouble("cgBlending", 50.0).toFloat(),
                    cgBalance = o.optDouble("cgBalance", 0.0).toFloat(),
                    sharpness = o.optDouble("sharpness", 20.0).toFloat(),
                    sharpenRadius = o.optDouble("sharpenRadius", 1.0).toFloat(),
                    luminanceNr = o.optDouble("luminanceNr", 0.0).toFloat(),
                    colorNr = o.optDouble("colorNr", 15.0).toFloat(),
                    lensDistortion = o.optDouble("lensDistortion", 0.0).toFloat(),
                    lensVignetting = o.optDouble("lensVignetting", 0.0).toFloat(),
                    lensCa = o.optDouble("lensCa", 0.0).toFloat(),
                    lensBlurAmount = o.optDouble("lensBlurAmount", 0.0).toFloat(),
                    lensBlurFocalCenter = o.optDouble("lensBlurFocalCenter", 50.0).toFloat(),
                    vignetteAmount = o.optDouble("vignetteAmount", 0.0).toFloat(),
                    vignetteMidpoint = o.optDouble("vignetteMidpoint", 50.0).toFloat(),
                    vignetteRoundness = o.optDouble("vignetteRoundness", 0.0).toFloat(),
                    vignetteFeather = o.optDouble("vignetteFeather", 50.0).toFloat(),
                    grainAmount = o.optDouble("grainAmount", 0.0).toFloat(),
                    grainSize = o.optDouble("grainSize", 25.0).toFloat(),
                    grainRoughness = o.optDouble("grainRoughness", 50.0).toFloat(),
                    cropLeft = o.optDouble("cropLeft", 0.0).toFloat(),
                    cropTop = o.optDouble("cropTop", 0.0).toFloat(),
                    cropRight = o.optDouble("cropRight", 1.0).toFloat(),
                    cropBottom = o.optDouble("cropBottom", 1.0).toFloat(),
                    cropAngle = o.optDouble("cropAngle", 0.0).toFloat(),
                    cropAspect = runCatching {
                        AspectRatioPreset.valueOf(o.optString("cropAspect", "ORIGINAL"))
                    }.getOrDefault(AspectRatioPreset.ORIGINAL),
                    rotationDegrees = o.optDouble("rotationDegrees", 0.0).toFloat(),
                    flipHorizontal = o.optBoolean("flipHorizontal", false),
                    flipVertical = o.optBoolean("flipVertical", false),
                    perspectiveVertical = o.optDouble("perspectiveVertical", 0.0).toFloat(),
                    perspectiveHorizontal = o.optDouble("perspectiveHorizontal", 0.0).toFloat(),
                    perspectiveAspect = o.optDouble("perspectiveAspect", 0.0).toFloat(),
                    geometryScale = o.optDouble("geometryScale", 100.0).toFloat(),
                    geometryOffsetX = o.optDouble("geometryOffsetX", 0.0).toFloat(),
                    geometryOffsetY = o.optDouble("geometryOffsetY", 0.0).toFloat(),
                    masks = maskList,
                    retouchSpots = retouchList,
                    appliedPresetName = o.optString("appliedPresetName", ""),
                    presetAmount = o.optDouble("presetAmount", 100.0).toFloat()
                )
            } catch (_: Exception) {
                DevelopParams()
            }
        }

        fun interpolate(base: DevelopParams, target: DevelopParams, amountPct: Float): DevelopParams {
            val t = (amountPct / 100f).coerceIn(0f, 2f)
            fun mix(a: Float, b: Float) = a + (b - a) * t
            fun mixArr(a: FloatArray, b: FloatArray) = FloatArray(minOf(a.size, b.size)) { i -> mix(a[i], b[i]) }
            return target.copy(
                temperature = mix(base.temperature, target.temperature).coerceIn(2000f, 18000f),
                tint = mix(base.tint, target.tint).coerceIn(-150f, 150f),
                exposure = mix(base.exposure, target.exposure).coerceIn(-5f, 5f),
                contrast = mix(base.contrast, target.contrast).coerceIn(-100f, 100f),
                highlights = mix(base.highlights, target.highlights).coerceIn(-100f, 100f),
                shadows = mix(base.shadows, target.shadows).coerceIn(-100f, 100f),
                whites = mix(base.whites, target.whites).coerceIn(-100f, 100f),
                blacks = mix(base.blacks, target.blacks).coerceIn(-100f, 100f),
                texture = mix(base.texture, target.texture).coerceIn(-100f, 100f),
                clarity = mix(base.clarity, target.clarity).coerceIn(-100f, 100f),
                dehaze = mix(base.dehaze, target.dehaze).coerceIn(-100f, 100f),
                vibrance = mix(base.vibrance, target.vibrance).coerceIn(-100f, 100f),
                saturation = mix(base.saturation, target.saturation).coerceIn(-100f, 100f),
                parametricHighlights = mix(base.parametricHighlights, target.parametricHighlights),
                parametricLights = mix(base.parametricLights, target.parametricLights),
                parametricDarks = mix(base.parametricDarks, target.parametricDarks),
                parametricShadows = mix(base.parametricShadows, target.parametricShadows),
                hueAdjustment = mixArr(base.hueAdjustment, target.hueAdjustment),
                saturationAdjustment = mixArr(base.saturationAdjustment, target.saturationAdjustment),
                luminanceAdjustment = mixArr(base.luminanceAdjustment, target.luminanceAdjustment),
                cgShadowSat = mix(base.cgShadowSat, target.cgShadowSat).coerceIn(0f, 100f),
                cgMidSat = mix(base.cgMidSat, target.cgMidSat).coerceIn(0f, 100f),
                cgHighSat = mix(base.cgHighSat, target.cgHighSat).coerceIn(0f, 100f),
                vignetteAmount = mix(base.vignetteAmount, target.vignetteAmount).coerceIn(-100f, 100f),
                grainAmount = mix(base.grainAmount, target.grainAmount).coerceIn(0f, 100f),
                presetAmount = amountPct
            )
        }
    }
}

data class CopySettingsMask(
    var profileAndColorTreatment: Boolean = true,
    var whiteBalance: Boolean = true,
    var basicTone: Boolean = true,
    var presence: Boolean = true,
    var toneCurve: Boolean = true,
    var colorMixer: Boolean = true,
    var colorGrading: Boolean = true,
    var detail: Boolean = true,
    var effects: Boolean = true,
    var optics: Boolean = true,
    var cropAndTransform: Boolean = false,
    var masksAndHeal: Boolean = false
) {
    fun applyTo(src: DevelopParams, dst: DevelopParams): DevelopParams {
        var out = dst.deepCopy()
        if (profileAndColorTreatment) {
            out = out.copy(
                convertToGrayscale = src.convertToGrayscale,
                cameraProfile = src.cameraProfile,
                lookAmount = src.lookAmount
            )
        }
        if (whiteBalance) {
            out = out.copy(whiteBalance = src.whiteBalance, temperature = src.temperature, tint = src.tint)
        }
        if (basicTone) {
            out = out.copy(
                exposure = src.exposure,
                contrast = src.contrast,
                highlights = src.highlights,
                shadows = src.shadows,
                whites = src.whites,
                blacks = src.blacks
            )
        }
        if (presence) {
            out = out.copy(
                texture = src.texture,
                clarity = src.clarity,
                dehaze = src.dehaze,
                vibrance = src.vibrance,
                saturation = src.saturation
            )
        }
        if (toneCurve) {
            out = out.copy(
                parametricHighlights = src.parametricHighlights,
                parametricLights = src.parametricLights,
                parametricDarks = src.parametricDarks,
                parametricShadows = src.parametricShadows,
                toneCurveMaster = src.toneCurveMaster.toList(),
                toneCurveRed = src.toneCurveRed.toList(),
                toneCurveGreen = src.toneCurveGreen.toList(),
                toneCurveBlue = src.toneCurveBlue.toList()
            )
        }
        if (colorMixer) {
            out = out.copy(
                hueAdjustment = src.hueAdjustment.copyOf(),
                saturationAdjustment = src.saturationAdjustment.copyOf(),
                luminanceAdjustment = src.luminanceAdjustment.copyOf(),
                grayMixer = src.grayMixer.copyOf()
            )
        }
        if (colorGrading) {
            out = out.copy(
                cgShadowHue = src.cgShadowHue,
                cgShadowSat = src.cgShadowSat,
                cgShadowLum = src.cgShadowLum,
                cgMidHue = src.cgMidHue,
                cgMidSat = src.cgMidSat,
                cgMidLum = src.cgMidLum,
                cgHighHue = src.cgHighHue,
                cgHighSat = src.cgHighSat,
                cgHighLum = src.cgHighLum,
                cgGlobalHue = src.cgGlobalHue,
                cgGlobalSat = src.cgGlobalSat,
                cgGlobalLum = src.cgGlobalLum,
                cgBlending = src.cgBlending,
                cgBalance = src.cgBalance
            )
        }
        if (detail) {
            out = out.copy(
                sharpness = src.sharpness,
                sharpenRadius = src.sharpenRadius,
                luminanceNr = src.luminanceNr,
                colorNr = src.colorNr
            )
        }
        if (effects) {
            out = out.copy(
                vignetteAmount = src.vignetteAmount,
                vignetteMidpoint = src.vignetteMidpoint,
                vignetteRoundness = src.vignetteRoundness,
                vignetteFeather = src.vignetteFeather,
                grainAmount = src.grainAmount,
                grainSize = src.grainSize,
                grainRoughness = src.grainRoughness
            )
        }
        if (optics) {
            out = out.copy(
                lensDistortion = src.lensDistortion,
                lensVignetting = src.lensVignetting,
                lensCa = src.lensCa,
                lensBlurAmount = src.lensBlurAmount,
                lensBlurFocalCenter = src.lensBlurFocalCenter
            )
        }
        if (cropAndTransform) {
            out = out.copy(
                cropLeft = src.cropLeft,
                cropTop = src.cropTop,
                cropRight = src.cropRight,
                cropBottom = src.cropBottom,
                cropAngle = src.cropAngle,
                cropAspect = src.cropAspect,
                rotationDegrees = src.rotationDegrees,
                flipHorizontal = src.flipHorizontal,
                flipVertical = src.flipVertical,
                perspectiveVertical = src.perspectiveVertical,
                perspectiveHorizontal = src.perspectiveHorizontal,
                perspectiveAspect = src.perspectiveAspect,
                geometryScale = src.geometryScale,
                geometryOffsetX = src.geometryOffsetX,
                geometryOffsetY = src.geometryOffsetY
            )
        }
        if (masksAndHeal) {
            out = out.copy(
                masks = src.masks.map { it.copy(brushPoints = it.brushPoints.toList()) },
                retouchSpots = src.retouchSpots.toList()
            )
        }
        return out
    }
}

data class DevelopPreset(
    val id: String,
    val category: String,
    val name: String,
    val isUser: Boolean,
    val isFavorite: Boolean,
    val params: DevelopParams,
    val xmpPath: String? = null
)

data class PhotoAsset(
    val idLocal: Long = 0L,
    val idGlobal: String = UUID.randomUUID().toString(),
    val uriString: String,
    val displayName: String,
    val width: Int = 0,
    val height: Int = 0,
    val orientation: Int = 1,
    val captureTime: String = "",
    val isRaw: Boolean = false,
    val rating: Int = 0,
    val pickFlag: PickFlag = PickFlag.UNFLAGGED,
    val colorLabel: ColorLabel = ColorLabel.NONE,
    val albumName: String = "All Photos",
    val params: DevelopParams = DevelopParams()
)

data class HistoryEntry(
    val id: Long,
    val name: String,
    val valueSummary: String,
    val timestamp: String,
    val params: DevelopParams
)

data class PhotoVersion(
    val id: Long,
    val name: String,
    val createdAt: String,
    val params: DevelopParams
)

data class AlbumInfo(
    val name: String,
    val photoCount: Int,
    val coverUri: String?,
    val isQuickCollection: Boolean = false
)

enum class WatermarkStyle(val label: String) {
    TEXT("Text"),
    GRAPHIC("Graphic")
}

enum class WatermarkSizeMode(val label: String) {
    PROPORTIONAL("Proportional"),
    FIT("Fit"),
    FILL("Fill")
}

enum class WatermarkAnchor(val label: String) {
    TOP_LEFT("TL"), TOP_CENTER("TC"), TOP_RIGHT("TR"),
    CENTER_LEFT("CL"), CENTER("C"), CENTER_RIGHT("CR"),
    BOTTOM_LEFT("BL"), BOTTOM_CENTER("BC"), BOTTOM_RIGHT("BR")
}

data class WatermarkSettings(
    val enabled: Boolean = false,
    val style: WatermarkStyle = WatermarkStyle.TEXT,
    val text: String = "© Moestret Photography",
    val graphicUriString: String? = null,
    val graphicDisplayName: String? = null,
    val anchor: WatermarkAnchor = WatermarkAnchor.BOTTOM_RIGHT,
    val sizeMode: WatermarkSizeMode = WatermarkSizeMode.PROPORTIONAL,
    val sizePercent: Float = 8f,
    val opacityPercent: Float = 82f,
    val insetPercent: Float = 4f,
    val shadowEnabled: Boolean = true,
    val rotationDegrees: Int = 0
)

enum class ExportFormat(val label: String, val mime: String, val ext: String) {
    JPEG("JPEG (.jpg)", "image/jpeg", "jpg"),
    PNG("PNG Lossless (.png)", "image/png", "png"),
    WEBP("WebP (.webp)", "image/webp", "webp")
}

enum class ExportResize(val label: String, val maxEdgePx: Int?) {
    ORIGINAL("Original Full Res", null),
    LONG_3840("4K Long Edge (3840px)", 3840),
    LONG_2048("Web / Instagram (2048px)", 2048),
    LONG_1080("Fast Share (1080px)", 1080)
}

data class ExportConfig(
    val format: ExportFormat = ExportFormat.JPEG,
    val quality: Int = 94,
    val resize: ExportResize = ExportResize.ORIGINAL,
    val watermark: WatermarkSettings = WatermarkSettings()
)
