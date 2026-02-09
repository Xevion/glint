package com.xevion.glint.ui

import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintDialogScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.Surface
import net.minecraft.client.gui.screens.Screen
import java.io.File
import net.minecraft.network.chat.Component as McComponent

/**
 * Dialog for selecting a local world directory to upload.
 * Shows all worlds from saves/ and glint/worlds/ with level.dat.
 */
class WorldPickerDialog(
    private val parentScreen: Screen,
    private val worlds: List<Pair<File, String>>,
    private val onSelect: (worldDir: File, worldName: String) -> Unit,
) : GlintDialogScreen(McComponent.literal("Select World")) {
    override fun buildDialog(dialog: FlowLayout) {
        dialog.child(GlintComponents.title(title) as Component)

        dialog.child(
            Components
                .label(McComponent.literal("Choose a world to upload:"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )

        // Scrollable list of worlds
        val listContainer = Containers.verticalFlow(Sizing.fixed(250), Sizing.content())
        listContainer.gap(GlintTheme.GAP_SM)

        val scroll =
            Containers.verticalScroll(
                Sizing.fixed(250),
                Sizing.fixed(150.coerceAtMost(worlds.size * 28 + 4)),
                listContainer,
            )

        for ((worldDir, worldName) in worlds) {
            val row = Containers.horizontalFlow(Sizing.fill(100), Sizing.fixed(24))
            row.padding(GlintTheme.paddingSm())
            row.gap(GlintTheme.GAP_SM)
            row.surface(Surface.flat(0x22FFFFFF))

            row.child(
                Components
                    .label(McComponent.literal(worldName))
                    .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component,
            )

            // Show directory source as hint
            val source = if (worldDir.parentFile?.name == "saves") "saves" else "glint"
            row.child(
                Components
                    .label(McComponent.literal("($source)"))
                    .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
            )

            row.cursorStyle(io.wispforest.owo.ui.core.CursorStyle.HAND)
            row.mouseDown().subscribe { _, _, button ->
                if (button == 0) {
                    onSelect(worldDir, worldName)
                    true
                } else {
                    false
                }
            }
            row.mouseEnter().subscribe {
                row.surface(Surface.flat(GlintTheme.HIGHLIGHT_BG))
                true
            }
            row.mouseLeave().subscribe {
                row.surface(Surface.flat(0x22FFFFFF))
                true
            }

            listContainer.child(row as Component)
        }

        dialog.child(scroll as Component)

        // Cancel button
        dialog.child(
            GlintComponents.cancelButton { minecraft?.setScreen(parentScreen) } as Component,
        )
    }
}
