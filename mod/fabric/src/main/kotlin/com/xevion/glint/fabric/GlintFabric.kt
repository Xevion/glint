package com.xevion.glint.fabric

import com.xevion.glint.Glint
import com.xevion.glint.command.GlintCommands
import net.fabricmc.api.ClientModInitializer
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents
import net.fabricmc.fabric.api.command.v2.CommandRegistrationCallback

class GlintFabric : ClientModInitializer {
    override fun onInitializeClient() {
        Glint.init()

        // Register client tick handler for keybind polling
        ClientTickEvents.END_CLIENT_TICK.register { Glint.onClientTick() }

        // Register /glint commands
        CommandRegistrationCallback.EVENT.register { dispatcher, _, _ ->
            GlintCommands.register(dispatcher)
        }
    }
}
