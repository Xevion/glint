package com.xevion.glint.mixin;

import com.xevion.glint.capture.CaptureState;
import net.minecraft.client.Minecraft;
import net.minecraft.client.MouseHandler;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Prevents automatic pausing and enables mouse interaction during capture sessions.
 *
 * During capture:
 * - Blocks pause screen from opening on window focus loss
 * - Releases mouse cursor for multi-tasking
 * - Keeps game rendering while window is unfocused
 */
@Mixin(Minecraft.class)
public class MinecraftMixin {

    @Shadow private MouseHandler mouseHandler;
    private boolean mouseReleasedForCapture = false;

    /**
     * Intercepts pauseGame() to prevent pause screen during capture.
     *
     * @param pauseOnly Whether to only pause without opening screen
     * @param ci Callback info for cancelling the method
     */
    @Inject(method = "pauseGame", at = @At("HEAD"), cancellable = true)
    private void onPauseGame(boolean pauseOnly, CallbackInfo ci) {
        if (CaptureState.INSTANCE.isActive()) {
            ci.cancel();
        }
    }

    /**
     * Releases mouse cursor and closes pause screen when capture session starts.
     */
    @Inject(method = "tick", at = @At("HEAD"))
    private void onTick(CallbackInfo ci) {
        boolean isCapturing = CaptureState.INSTANCE.isActive();
        Minecraft mc = (Minecraft) (Object) this;
        
        if (isCapturing && !mouseReleasedForCapture) {
            // Close pause screen if it's currently open
            if (mc.screen != null && mc.screen.isPauseScreen()) {
                mc.setScreen(null);
            }
            
            // Release mouse to allow user to interact with other windows
            if (mouseHandler.isMouseGrabbed()) {
                mouseHandler.releaseMouse();
            }
            mouseReleasedForCapture = true;
        } else if (!isCapturing && mouseReleasedForCapture) {
            // Reset flag when capture ends
            mouseReleasedForCapture = false;
        }
    }
}
