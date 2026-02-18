package com.xevion.glint.orchestration

import com.xevion.glint.capture.CaptureEntry

/** Event fired when a single capture is taken during orchestration. */
class CaptureTakenEvent(
    val entry: CaptureEntry,
    val fileBytes: ByteArray,
    val sceneId: String,
    val presetId: String? = null,
)
