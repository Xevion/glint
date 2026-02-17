package com.xevion.glint.mixin;

import com.xevion.glint.scene.ActiveScene;
import com.xevion.glint.scene.SceneChunkProvider;
import com.xevion.glint.scene.SceneChunkProviderKt;
import com.xevion.glint.scene.SceneInjection;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.chunk.storage.ChunkStorage;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Intercepts chunk reads to serve scene data from memory when a scene is active.
 *
 * <p>When {@link SceneInjection} has an active provider, chunk reads within scene bounds are
 * redirected to the scene's region files. Reads outside scene bounds fall through to the original
 * {@link ChunkStorage#read} so the real world data loads normally.
 *
 * <p>Returned chunk NBT is relocated: position fields (xPos/zPos, block entities, ticks) are
 * patched to match the world coordinates so Minecraft's position validation passes.
 */
@Mixin(ChunkStorage.class)
public class ChunkStorageMixin {

    @Inject(method = "read", at = @At("HEAD"), cancellable = true)
    private void glint$interceptChunkRead(
            ChunkPos pos, CallbackInfoReturnable<CompletableFuture<Optional<CompoundTag>>> cir) {
        ActiveScene scene = SceneInjection.INSTANCE.getActiveScene();
        if (scene == null) return;

        SceneChunkProvider provider = scene.getProvider();
        int offsetX = scene.getChunkOffsetX();
        int offsetZ = scene.getChunkOffsetZ();
        int sceneX = pos.x - offsetX;
        int sceneZ = pos.z - offsetZ;

        if (!provider.isInBounds(sceneX, sceneZ)) return;

        CompoundTag chunk = provider.getChunk(new ChunkPos(sceneX, sceneZ));
        if (chunk != null) {
            SceneChunkProviderKt.relocateChunkNbt(chunk, pos.x, pos.z, offsetX * 16, offsetZ * 16);
        }
        cir.setReturnValue(CompletableFuture.completedFuture(Optional.ofNullable(chunk)));
    }
}
