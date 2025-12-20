package com.xevion.glint.mixin;

import com.mojang.blaze3d.pipeline.RenderTarget;
import com.xevion.glint.screenshot.ScreenshotHandler;
import net.minecraft.client.Screenshot;
import net.minecraft.network.chat.Component;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.io.File;
import java.util.function.Consumer;

/**
 * Mixin to intercept screenshot captures and collect metadata.
 * Injects at HEAD to capture game state before vanilla processes the screenshot.
 */
@Mixin(Screenshot.class)
public class ScreenshotCaptureMixin {

    @Inject(
            method = "grab(Ljava/io/File;Ljava/lang/String;Lcom/mojang/blaze3d/pipeline/RenderTarget;Ljava/util/function/Consumer;)V",
            at = @At("HEAD")
    )
    private static void glint$captureScreenshot(
            File gameDir,
            String filename,
            RenderTarget renderTarget,
            Consumer<Component> consumer,
            CallbackInfo ci
    ) {
        ScreenshotHandler.INSTANCE.onScreenshotCaptured(renderTarget);
    }
}
