package com.xevion.glint.fabric

import com.xevion.glint.Glint
import net.fabricmc.api.ClientModInitializer

class GlintFabric : ClientModInitializer {
    override fun onInitializeClient() {
        Glint.init()
    }
}
