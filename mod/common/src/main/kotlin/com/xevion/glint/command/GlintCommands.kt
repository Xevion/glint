package com.xevion.glint.command

import com.mojang.brigadier.CommandDispatcher
import com.mojang.brigadier.arguments.StringArgumentType
import com.xevion.glint.Loggers
import com.xevion.glint.scene.CameraPosition
import com.xevion.glint.scene.InjectionManager
import com.xevion.glint.scene.SceneExporter
import net.minecraft.client.Minecraft
import net.minecraft.commands.CommandSourceStack
import net.minecraft.commands.Commands
import net.minecraft.network.chat.Component
import net.minecraft.server.level.ServerLevel
import net.minecraft.server.level.ServerPlayer
import net.minecraft.util.Mth

/**
 * Registers all `/glint` subcommands.
 * Called from platform-specific command registration callbacks.
 */
object GlintCommands {
    private val log = Loggers.Scene.get()

    fun register(dispatcher: CommandDispatcher<CommandSourceStack>) {
        dispatcher.register(
            Commands
                .literal("glint")
                .then(exportCommand())
                .then(injectCommand())
                .then(deactivateCommand()),
        )
    }

    private fun exportCommand() =
        Commands
            .literal("export")
            .then(
                Commands
                    .argument("name", StringArgumentType.word())
                    .executes { ctx ->
                        val name = StringArgumentType.getString(ctx, "name")
                        SceneExporter.export(name).fold(
                            onSuccess = { path ->
                                ctx.source.sendSuccess(
                                    { Component.literal("Exported scene package: $path") },
                                    false,
                                )
                                1
                            },
                            onFailure = { error ->
                                ctx.source.sendFailure(
                                    Component.literal("Export failed: ${error.message}"),
                                )
                                0
                            },
                        )
                    },
            )

    private fun injectCommand() =
        Commands
            .literal("inject")
            .then(
                Commands
                    .argument("name", StringArgumentType.word())
                    .executes { ctx ->
                        try {
                            val name = StringArgumentType.getString(ctx, "name")
                            val mc = Minecraft.getInstance()
                            val exportsDir =
                                mc.gameDirectory
                                    .toPath()
                                    .resolve("glint")
                                    .resolve("exports")
                            val zipPath = exportsDir.resolve("$name.zip")

                            if (!zipPath.toFile().exists()) {
                                ctx.source.sendFailure(
                                    Component.literal("Scene package not found: $zipPath"),
                                )
                                return@executes 0
                            }

                            val level = ctx.source.level as? ServerLevel
                            if (level == null) {
                                ctx.source.sendFailure(
                                    Component.literal("Must be in a world to inject a scene"),
                                )
                                return@executes 0
                            }

                            val scene =
                                runCatching { InjectionManager.load(zipPath) }.getOrElse { error ->
                                    ctx.source.sendFailure(
                                        Component.literal("Failed to load scene: ${error.message}"),
                                    )
                                    return@executes 0
                                }

                            // Compute chunk offset: anchor scene camera on player's current chunk
                            val player = ctx.source.entity as? ServerPlayer
                            val camera = scene.meta.camera
                            val chunkOffsetX: Int
                            val chunkOffsetZ: Int
                            if (player != null) {
                                val playerChunkX = Mth.floor(player.x) shr 4
                                val playerChunkZ = Mth.floor(player.z) shr 4
                                val cameraChunkX = Mth.floor(camera.x) shr 4
                                val cameraChunkZ = Mth.floor(camera.z) shr 4
                                chunkOffsetX = playerChunkX - cameraChunkX
                                chunkOffsetZ = playerChunkZ - cameraChunkZ
                            } else {
                                chunkOffsetX = 0
                                chunkOffsetZ = 0
                            }

                            // Compute offset camera position for teleport after injection
                            val blockOffsetX = chunkOffsetX * 16
                            val blockOffsetZ = chunkOffsetZ * 16
                            val cameraTarget =
                                CameraPosition(
                                    x = camera.x + blockOffsetX,
                                    y = camera.y,
                                    z = camera.z + blockOffsetZ,
                                    yaw = camera.yaw,
                                    pitch = camera.pitch,
                                )

                            InjectionManager.startInjection(scene, level, chunkOffsetX, chunkOffsetZ, cameraTarget)

                            ctx.source.sendSuccess(
                                {
                                    Component.literal(
                                        "Injecting scene '$name' at player position " +
                                            "(offset: $chunkOffsetX, $chunkOffsetZ chunks)...",
                                    )
                                },
                                false,
                            )
                            1
                        } catch (e: Exception) {
                            log.error(e, "Inject command failed")
                            ctx.source.sendFailure(
                                Component.literal("Inject failed: ${e.message}"),
                            )
                            0
                        }
                    },
            )

    private fun deactivateCommand() =
        Commands
            .literal("deactivate")
            .executes { ctx ->
                try {
                    if (!InjectionManager.isActive) {
                        ctx.source.sendFailure(Component.literal("No active scene to deactivate"))
                        return@executes 0
                    }

                    InjectionManager.forceDeactivate(ctx.source.level as? ServerLevel)

                    ctx.source.sendSuccess(
                        { Component.literal("Scene deactivated") },
                        false,
                    )
                    1
                } catch (e: Exception) {
                    log.error(e, "Deactivate command failed")
                    ctx.source.sendFailure(
                        Component.literal("Deactivate failed: ${e.message}"),
                    )
                    0
                }
            }
}
