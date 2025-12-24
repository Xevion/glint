package com.xevion.glint.mixin;

import com.xevion.glint.capture.CaptureStateManager;
import net.minecraft.client.KeyboardHandler;
import org.lwjgl.glfw.GLFW;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Blocks keyboard input during capture sessions.
 *
 * During capture:
 * - ESC cancels the session (first press only)
 * - All other keys are blocked (T for chat, F1-F5, F2 screenshot, F3 debug, etc.)
 */
@Mixin(KeyboardHandler.class)
public class KeyboardHandlerMixin {

    /**
     * Intercepts keyPress() to block keyboard input during capture.
     * ESC key cancels the session; all other keys are blocked.
     *
     * @param window The window handle
     * @param key The GLFW key code
     * @param scancode The platform-specific scancode
     * @param action The key action (press/release/repeat)
     * @param mods The modifier keys
     * @param ci Callback info for cancelling the method
     */
    @Inject(method = "keyPress", at = @At("HEAD"), cancellable = true)
    private void onKeyPress(long window, int key, int scancode, int action, int mods, CallbackInfo ci) {
        if (!CaptureStateManager.INSTANCE.isActive()) {
            return;
        }

        // ESC key cancels the session
        if (key == GLFW.GLFW_KEY_ESCAPE && action == GLFW.GLFW_PRESS) {
            CaptureStateManager.INSTANCE.requestCancel();
            ci.cancel();
            return;
        }

        // Block all other keys during capture
        ci.cancel();
    }

    /**
     * Intercepts charTyped() to block character input during capture.
     *
     * @param window The window handle
     * @param codePoint The Unicode code point
     * @param mods The modifier keys
     * @param ci Callback info for cancelling the method
     */
    @Inject(method = "charTyped", at = @At("HEAD"), cancellable = true)
    private void onCharTyped(long window, int codePoint, int mods, CallbackInfo ci) {
        if (CaptureStateManager.INSTANCE.isActive()) {
            ci.cancel();
        }
    }
}
