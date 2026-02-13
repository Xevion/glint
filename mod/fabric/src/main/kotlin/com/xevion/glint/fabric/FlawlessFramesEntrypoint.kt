package com.xevion.glint.fabric

import com.xevion.glint.capture.FlawlessFrames
import java.util.function.Consumer
import java.util.function.Function

/**
 * Fabric entrypoint for the FREX Flawless Frames API.
 *
 * Registered as `frex_flawless_frames` in fabric.mod.json. Sodium discovers this at startup
 * and calls [accept] with its provider function, which we delegate to [FlawlessFrames].
 */
class FlawlessFramesEntrypoint : Consumer<Function<String, Consumer<Boolean>>> {
    override fun accept(provider: Function<String, Consumer<Boolean>>) {
        FlawlessFrames.registerToggle(provider)
    }
}
