package com.xevion.glint.mixin;

import com.xevion.glint.scene.SceneInjection;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.Entity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Suppresses entity ticking when scene injection has entity freeze active.
 *
 * <p>Both {@code tickNonPassenger} and {@code tickPassenger} are cancelled at HEAD, preventing all
 * entity movement, AI, and physics while a scene is being captured. This keeps the scene visually
 * frozen for consistent screenshots.
 */
@Mixin(ServerLevel.class)
public class ServerLevelMixin {

    @Inject(method = "tickNonPassenger", at = @At("HEAD"), cancellable = true)
    private void glint$freezeEntityTick(Entity entity, CallbackInfo ci) {
        if (SceneInjection.INSTANCE.getEntityTickFrozen()) {
            ci.cancel();
        }
    }

    @Inject(method = "tickPassenger", at = @At("HEAD"), cancellable = true)
    private void glint$freezePassengerTick(Entity entity, Entity passenger, CallbackInfo ci) {
        if (SceneInjection.INSTANCE.getEntityTickFrozen()) {
            ci.cancel();
        }
    }
}
