package com.xevion.glint.fabric

import com.xevion.glint.Glint
import net.fabricmc.api.ClientModInitializer
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents

class GlintFabric : ClientModInitializer {
    override fun onInitializeClient() {
        Glint.init()
        
        // Register client tick handler for keybind polling
        ClientTickEvents.END_CLIENT_TICK.register { Glint.onClientTick() }
    }
}
