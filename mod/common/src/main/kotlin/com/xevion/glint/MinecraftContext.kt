package com.xevion.glint

import net.minecraft.client.Minecraft
import net.minecraft.client.multiplayer.ClientLevel
import net.minecraft.client.player.LocalPlayer
import net.minecraft.client.server.IntegratedServer

/** Provides typed access to Minecraft singletons that are only available in-world. */
data class MinecraftContext(
    val mc: Minecraft,
    val player: LocalPlayer,
    val level: ClientLevel,
) {
    /** Non-null only when running an integrated server (singleplayer/LAN). */
    val server: IntegratedServer? get() = mc.singleplayerServer

    /** Like [server] but throws if not available. */
    fun requireServer(): IntegratedServer = mc.singleplayerServer ?: error("No integrated server")
}

/**
 * Runs [block] with a [MinecraftContext] if player and level are available.
 * Returns null if the context cannot be constructed (not in-world).
 */
inline fun <T> withMinecraftContext(block: MinecraftContext.() -> T): T? {
    val mc = Minecraft.getInstance()
    val player = mc.player ?: return null
    val level = mc.level ?: return null
    return block(MinecraftContext(mc, player, level))
}
