package com.xevion.glint.mixin;

import com.mojang.blaze3d.platform.Window;
import com.xevion.glint.capture.HighResCapture;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Suppresses GLFW framebuffer resize events during high-res capture sessions.
 *
 * <p>When a capture session is active, {@link HighResCapture} has spoofed {@code
 * window.framebufferWidth/Height} to 3840×2160. GLFW fires {@code onFramebufferResize} whenever the
 * physical GLFW window reports a size change (world loads, shader reloads, WM events, etc.), which
 * overwrites our spoofed values with the real physical dimensions and calls {@code
 * resizeDisplay()}, collapsing {@code mainRenderTarget} back to the physical size (854×495 in dev).
 *
 * <p>Cancelling this callback during capture keeps the spoofed 3840×2160 dimensions intact so every
 * {@code Iris.reload()} reads {@code window.getWidth() == 3840} and builds its pipeline at the
 * correct capture resolution.
 */
@Mixin(Window.class)
public class WindowMixin {

    @Inject(method = "onFramebufferResize", at = @At("HEAD"), cancellable = true)
    private void glint$suppressResizeDuringCapture(
            long window, int width, int height, CallbackInfo ci) {
        if (HighResCapture.INSTANCE.isSessionActive()) {
            ci.cancel();
        }
    }
}
