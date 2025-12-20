package com.xevion.glint

import com.xevion.glint.input.KeybindHandler
import org.slf4j.Logger
import org.slf4j.LoggerFactory

object Glint {
    const val MOD_ID = "glint"
    val LOGGER: Logger = LoggerFactory.getLogger(MOD_ID)

    fun init() {
        LOGGER.info("Initializing Glint mod")
    }

    /**
     * Called every client tick. Register this with the platform's tick event.
     */
    fun onClientTick() {
        KeybindHandler.onTick()
    }
}
