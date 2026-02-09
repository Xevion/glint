package com.xevion.glint.mixin;

import com.mojang.blaze3d.pipeline.RenderTarget;
import com.xevion.glint.capture.HighResCapture;
import net.minecraft.client.Minecraft;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

/**
 * Fixes the blit-to-screen step during capture sessions.
 *
 * <p>When the framebuffer is at 4K but the physical GLFW window is smaller, the default {@code
 * blitToScreen(window.getWidth(), window.getHeight())} uses the spoofed 4K dimensions as the
 * destination rectangle. This causes the GPU to blit a 4K source onto a 4K destination that's
 * clipped to the physical window, showing only the top-left corner (zoom artifact).
 *
 * <p>This mixin substitutes the real GLFW window dimensions so the 4K framebuffer is properly
 * downscaled to fit the physical display.
 */
@Mixin(Minecraft.class)
public class BlitScaleMixin {

    @Redirect(
            method = "runTick",
            at =
                    @At(
                            value = "INVOKE",
                            target = "Lcom/mojang/blaze3d/pipeline/RenderTarget;blitToScreen(II)V"))
    private void redirectBlitToScreen(RenderTarget renderTarget, int width, int height) {
        if (HighResCapture.INSTANCE.isSessionActive()) {
            renderTarget.blitToScreen(
                    HighResCapture.INSTANCE.getRealFramebufferWidth(),
                    HighResCapture.INSTANCE.getRealFramebufferHeight());
        } else {
            renderTarget.blitToScreen(width, height);
        }
    }
}
