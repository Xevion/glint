package com.xevion.glint.mixin;

import com.xevion.glint.capture.HighResCapture;
import net.minecraft.client.Minecraft;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Hooks into the render loop to advance high-resolution capture tasks.
 *
 * <p>Injects immediately after GameRenderer.render() completes in the main tick loop. At this
 * point, the frame has been fully rendered to the main render target and is ready for pixel
 * readback.
 */
@Mixin(Minecraft.class)
public class GameRendererMixin {

    @Inject(
            method = "runTick",
            at =
                    @At(
                            value = "INVOKE",
                            target =
                                    "Lnet/minecraft/client/renderer/GameRenderer;render(Lnet/minecraft/client/DeltaTracker;Z)V",
                            shift = At.Shift.AFTER))
    private void onPostRender(CallbackInfo ci) {
        HighResCapture.INSTANCE.onPostRender();
    }
}
