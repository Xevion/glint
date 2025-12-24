package com.xevion.glint

import com.xevion.glint.input.KeybindHandler
import com.xevion.glint.session.SessionRegistry
import org.slf4j.LoggerFactory

object Glint {
    const val MOD_ID = "glint"
    val LOGGER = LoggerFactory.getLogger(MOD_ID)

    fun init() {
        LOGGER.info("Initializing Glint mod")
        LogConfig.setupDebugLogging(MOD_ID)
    }

    fun onClientTick() {
        SessionRegistry.tick()
        KeybindHandler.onTick()
    }
}
