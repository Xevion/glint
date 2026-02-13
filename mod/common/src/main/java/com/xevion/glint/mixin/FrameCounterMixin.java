package com.xevion.glint.mixin;

import com.xevion.glint.capture.CaptureTimeOverride;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Pseudo;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Overrides Iris's {@code frameCounter} shader uniform during capture.
 *
 * <p>The frame counter is an integer (0–720719) that increments every rendered frame. Shaders use
 * it for TAA jitter patterns, dithering, and some animation offsets. Without this override,
 * different shader profiles accumulate different total frame counts before capture, producing
 * non-deterministic uniform values.
 *
 * <p>{@code @Pseudo} allows this mixin to load even when Iris is not installed.
 *
 * @see CaptureTimeOverride
 */
@Pseudo
@Mixin(targets = "net.irisshaders.iris.uniforms.SystemTimeUniforms$FrameCounter", remap = false)
public class FrameCounterMixin {

    /**
     * Override {@code frameCounter} with a deterministic value starting from zero at capture
     * activation.
     */
    @Inject(method = "getAsInt", at = @At("HEAD"), cancellable = true, require = 0)
    private void glint$overrideFrameCounter(CallbackInfoReturnable<Integer> cir) {
        Integer override = CaptureTimeOverride.INSTANCE.getOverrideFrameCounter();
        if (override != null) {
            cir.setReturnValue(override);
        }
    }
}
