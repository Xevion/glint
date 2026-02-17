package com.xevion.glint.neoforge

import com.xevion.glint.Glint
import com.xevion.glint.command.GlintCommands
import net.neoforged.fml.common.Mod
import net.neoforged.neoforge.client.event.ClientTickEvent
import net.neoforged.neoforge.common.NeoForge
import net.neoforged.neoforge.event.RegisterCommandsEvent

@Mod(Glint.MOD_ID)
class GlintNeoForge {
    init {
        Glint.init()

        // Register client tick handler for keybind polling and orchestration
        NeoForge.EVENT_BUS.addListener<ClientTickEvent.Post> { _ ->
            Glint.onClientTick()
        }

        // Register /glint commands
        NeoForge.EVENT_BUS.addListener<RegisterCommandsEvent> { event ->
            GlintCommands.register(event.dispatcher)
        }
    }
}
