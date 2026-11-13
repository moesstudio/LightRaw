package com.moestret.lightraw.ui

import com.moestret.lightraw.model.CopySettingsMask
import com.moestret.lightraw.model.DevelopParams
import com.moestret.lightraw.model.WatermarkSettings

/**
 * In-session state shared across Library Grid, Loupe Editor, and Export dialogs.
 */
object SessionClipboard {
    var copiedParams: DevelopParams? = null
    var copyMask: CopySettingsMask = CopySettingsMask()
    var lastWatermark: WatermarkSettings = WatermarkSettings()
}
