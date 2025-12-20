package com.xevion.glint

import org.slf4j.Logger
import org.slf4j.LoggerFactory

object Glint {
    const val MOD_ID = "glint"
    val LOGGER: Logger = LoggerFactory.getLogger(MOD_ID)

    fun init() {
        LOGGER.info("Initializing Glint mod")
    }
}
