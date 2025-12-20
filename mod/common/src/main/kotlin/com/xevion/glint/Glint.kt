package com.xevion.glint

import com.xevion.glint.input.KeybindHandler
import org.slf4j.Logger
import org.slf4j.LoggerFactory

object Glint {
    const val MOD_ID = "glint"
    val LOGGER: Logger = LoggerFactory.getLogger(MOD_ID)

    fun init() {
        LogConfig.setupDebugLogging(MOD_ID)
        LOGGER.info("Initializing Glint mod")
    }

    fun onClientTick() {
        KeybindHandler.onTick()
    }
}
