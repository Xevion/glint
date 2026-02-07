package com.xevion.glint.mixin;

import com.mojang.blaze3d.platform.NativeImage;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

/**
 * Accessor for NativeImage's private pixel memory pointer.
 *
 * <p>Required by FramebufferWriter to pass raw pixel data directly to STB image encoding functions
 * without an intermediate copy.
 */
@Mixin(NativeImage.class)
public interface NativeImageAccessor {

    @Accessor("pixels")
    long glint$getPixels();
}
