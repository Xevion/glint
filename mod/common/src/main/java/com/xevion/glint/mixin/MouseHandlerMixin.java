package com.xevion.glint.mixin;

import com.xevion.glint.capture.CaptureStateManager;
import net.minecraft.client.MouseHandler;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Controls mouse behavior during screenshot capture sessions.
 *
 * During capture:
 * - Blocks camera rotation (turnPlayer)
 * - Prevents mouse re-grabbing (grabMouse)
 */
@Mixin(MouseHandler.class)
public class MouseHandlerMixin {

    @Shadow private double accumulatedDX;

    @Shadow private double accumulatedDY;

    /**
     * Intercepts turnPlayer() to block camera rotation during capture.
     *
     * @param deltaTime The delta time since last frame
     * @param ci Callback info for cancelling the method
     */
    @Inject(method = "turnPlayer", at = @At("HEAD"), cancellable = true)
    private void onTurnPlayer(double deltaTime, CallbackInfo ci) {
        if (CaptureStateManager.INSTANCE.isActive()) {
            accumulatedDX = 0.0;
            accumulatedDY = 0.0;
            ci.cancel();
        }
    }

    /**
     * Intercepts grabMouse() to prevent mouse re-grabbing during capture.
     * This ensures clicking during a session doesn't refocus the game.
     *
     * @param ci Callback info for cancelling the method
     */
    @Inject(method = "grabMouse", at = @At("HEAD"), cancellable = true)
    private void onGrabMouse(CallbackInfo ci) {
        if (CaptureStateManager.INSTANCE.isActive()) {
            ci.cancel();
        }
    }
}
