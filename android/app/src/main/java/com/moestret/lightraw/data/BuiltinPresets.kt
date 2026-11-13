package com.moestret.lightraw.data

import com.moestret.lightraw.model.CameraProfile
import com.moestret.lightraw.model.CurvePoint
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.DevelopPreset

object BuiltinPresets {
    fun all(): List<DevelopPreset> = listOf(
        DevelopPreset(
            id = "builtin_natural",
            category = "Basic",
            name = "Clean Natural",
            isUser = false,
            isFavorite = true,
            params = DevelopParams(
                contrast = 12f,
                highlights = -14f,
                shadows = 16f,
                clarity = 6f,
                vibrance = 10f,
                sharpness = 28f
            )
        ),
        DevelopPreset(
            id = "builtin_golden_hour",
            category = "Portrait",
            name = "Warm Golden",
            isUser = false,
            isFavorite = true,
            params = DevelopParams(
                cameraProfile = CameraProfile.PORTRAIT,
                temperature = 6150f,
                tint = 6f,
                exposure = 0.15f,
                contrast = 10f,
                highlights = -22f,
                shadows = 24f,
                whites = 8f,
                clarity = -4f,
                vibrance = 14f,
                cgShadowHue = 32f,
                cgShadowSat = 12f,
                cgHighHue = 44f,
                cgHighSat = 16f,
                vignetteAmount = -12f
            )
        ),
        DevelopPreset(
            id = "builtin_flattering_portrait",
            category = "Portrait",
            name = "Soft Skin Glow",
            isUser = false,
            isFavorite = false,
            params = DevelopParams(
                cameraProfile = CameraProfile.PORTRAIT,
                highlights = -18f,
                shadows = 22f,
                texture = -14f,
                clarity = -10f,
                vibrance = 8f,
                saturationAdjustment = floatArrayOf(0f, -6f, -8f, 0f, 0f, 0f, 0f, 0f),
                luminanceAdjustment = floatArrayOf(0f, 10f, 6f, 0f, 0f, 0f, 0f, 0f),
                vignetteAmount = -10f
            )
        ),
        DevelopPreset(
            id = "builtin_teal_orange",
            category = "Cinematic",
            name = "Teal & Orange",
            isUser = false,
            isFavorite = true,
            params = DevelopParams(
                contrast = 24f,
                highlights = -28f,
                shadows = 18f,
                blacks = -14f,
                clarity = 12f,
                dehaze = 10f,
                hueAdjustment = floatArrayOf(6f, -8f, -12f, 22f, -16f, -18f, 0f, 0f),
                saturationAdjustment = floatArrayOf(8f, 18f, -15f, -25f, 12f, 22f, -20f, -20f),
                cgShadowHue = 198f,
                cgShadowSat = 26f,
                cgHighHue = 34f,
                cgHighSat = 22f,
                vignetteAmount = -24f,
                grainAmount = 14f
            )
        ),
        DevelopPreset(
            id = "builtin_portra_film",
            category = "Film",
            name = "Warm Analog 400",
            isUser = false,
            isFavorite = false,
            params = DevelopParams(
                cameraProfile = CameraProfile.FILM,
                temperature = 5850f,
                tint = 4f,
                contrast = 14f,
                highlights = -26f,
                shadows = 20f,
                blacks = 12f,
                vibrance = 10f,
                saturation = -8f,
                toneCurveMaster = listOf(
                    CurvePoint(0f, 0.04f),
                    CurvePoint(0.25f, 0.23f),
                    CurvePoint(0.75f, 0.78f),
                    CurvePoint(1f, 0.96f)
                ),
                cgShadowHue = 115f,
                cgShadowSat = 8f,
                cgMidHue = 38f,
                cgMidSat = 10f,
                grainAmount = 22f,
                grainSize = 30f
            )
        ),
        DevelopPreset(
            id = "builtin_crisp_landscape",
            category = "Landscape",
            name = "Crisp Alpine",
            isUser = false,
            isFavorite = false,
            params = DevelopParams(
                cameraProfile = CameraProfile.LANDSCAPE,
                contrast = 18f,
                highlights = -32f,
                shadows = 28f,
                whites = 14f,
                texture = 16f,
                clarity = 22f,
                dehaze = 18f,
                vibrance = 20f,
                sharpness = 45f
            )
        ),
        DevelopPreset(
            id = "builtin_bw_contrast",
            category = "B&W",
            name = "Noir Tri-X",
            isUser = false,
            isFavorite = false,
            params = DevelopParams(
                convertToGrayscale = true,
                cameraProfile = CameraProfile.MONOCHROME,
                contrast = 38f,
                highlights = -18f,
                shadows = 12f,
                whites = 18f,
                blacks = -22f,
                clarity = 24f,
                grayMixer = floatArrayOf(-8f, 18f, 14f, -10f, -14f, -22f, -12f, -4f),
                grainAmount = 32f,
                grainSize = 34f,
                vignetteAmount = -28f
            )
        ),
        DevelopPreset(
            id = "builtin_bw_silver",
            category = "B&W",
            name = "Soft Silver",
            isUser = false,
            isFavorite = false,
            params = DevelopParams(
                convertToGrayscale = true,
                cameraProfile = CameraProfile.MONOCHROME,
                contrast = 16f,
                highlights = -24f,
                shadows = 26f,
                blacks = 8f,
                clarity = 8f,
                grainAmount = 16f
            )
        ),
        DevelopPreset(
            id = "builtin_moody_forest",
            category = "Moody",
            name = "Nordic Pine",
            isUser = false,
            isFavorite = false,
            params = DevelopParams(
                temperature = 5150f,
                exposure = -0.15f,
                contrast = 20f,
                highlights = -25f,
                shadows = 14f,
                blacks = -16f,
                clarity = 16f,
                vibrance = -8f,
                saturation = -14f,
                hueAdjustment = floatArrayOf(0f, -6f, -22f, -26f, 0f, -10f, 0f, 0f),
                saturationAdjustment = floatArrayOf(-10f, -12f, -28f, -32f, -15f, -20f, -30f, -30f),
                luminanceAdjustment = floatArrayOf(0f, 6f, -12f, -20f, -10f, -16f, 0f, 0f),
                cgShadowHue = 175f,
                cgShadowSat = 18f,
                vignetteAmount = -26f
            )
        ),
        DevelopPreset(
            id = "builtin_vivid_street",
            category = "Cinematic",
            name = "Tokyo Neon",
            isUser = false,
            isFavorite = false,
            params = DevelopParams(
                cameraProfile = CameraProfile.VIVID,
                temperature = 4900f,
                tint = 12f,
                contrast = 26f,
                highlights = -20f,
                shadows = 22f,
                clarity = 18f,
                dehaze = 14f,
                vibrance = 26f,
                cgShadowHue = 225f,
                cgShadowSat = 28f,
                cgHighHue = 325f,
                cgHighSat = 18f
            )
        )
    )
}
