package com.xevion.glint.ui.tabs

import io.wispforest.owo.ui.container.FlowLayout

/** Contract for each tab in GlintMainScreen. */
interface MainScreenTab {
    fun buildMaster(master: FlowLayout)

    fun buildDetail(detail: FlowLayout)
}
