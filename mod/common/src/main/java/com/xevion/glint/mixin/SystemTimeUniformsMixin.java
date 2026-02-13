package com.xevion.glint.mixin;

import com.xevion.glint.capture.CaptureTimeOverride;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Pseudo;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Overrides Iris's shader time uniforms during capture with deterministic values.
 *
 * <p>Iris's {@code Timer.frameTimeCounter} is <b>cumulative</b> — it sums all frame deltas since
 * game start. Overriding the input to {@code beginFrame(long)} doesn't help because the accumulated
 * base is already non-deterministic by the time capture starts. Instead, we intercept the getter
 * methods that feed uniform values to shaders.
 *
 * <p>{@code @Pseudo} allows this mixin to load even when Iris is not installed — it simply has no
 * effect. The {@code require = 0} on each injection prevents errors when the target class is
 * absent.
 *
 * @see CaptureTimeOverride
 */
@Pseudo
@Mixin(targets = "net.irisshaders.iris.uniforms.SystemTimeUniforms$Timer", remap = false)
public class SystemTimeUniformsMixin {

    /**
     * Override {@code frameTimeCounter} (cumulative seconds) with a deterministic value starting
     * from zero at capture activation.
     */
    @Inject(method = "getFrameTimeCounter", at = @At("HEAD"), cancellable = true, require = 0)
    private void glint$overrideFrameTimeCounter(CallbackInfoReturnable<Float> cir) {
        Float override = CaptureTimeOverride.INSTANCE.getOverrideFrameTimeCounter();
        if (override != null) {
            cir.setReturnValue(override);
        }
    }

    /** Override {@code frameTime} (per-frame delta in seconds) with a constant 1/60s. */
    @Inject(method = "getLastFrameTime", at = @At("HEAD"), cancellable = true, require = 0)
    private void glint$overrideLastFrameTime(CallbackInfoReturnable<Float> cir) {
        Float override = CaptureTimeOverride.INSTANCE.getOverrideLastFrameTime();
        if (override != null) {
            cir.setReturnValue(override);
        }
    }
}
