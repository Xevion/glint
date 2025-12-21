package com.xevion.glint.mixin;

import com.xevion.glint.capture.CaptureState;
import net.minecraft.client.MouseHandler;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Prevents camera rotation during screenshot capture sessions.
 *
 * Blocks the turnPlayer() method that applies mouse input to player rotation,
 * ensuring camera remains stable during multi-shader captures.
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
        if (CaptureState.INSTANCE.isActive()) {
            accumulatedDX = 0.0;
            accumulatedDY = 0.0;
            ci.cancel();
        }
    }
}
