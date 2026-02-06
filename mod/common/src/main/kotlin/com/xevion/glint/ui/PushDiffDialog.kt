package com.xevion.glint.ui

import com.xevion.glint.api.SceneDiff
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintDialogScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.Insets
import io.wispforest.owo.ui.core.Sizing
import net.minecraft.network.chat.Component as McComponent

/**
 * Diff preview dialog for pushing scenes to the API.
 * Shows what will be created, updated, and removed before executing.
 */
class PushDiffDialog(
    private val parent: GlintMainScreen,
    private val worldName: String,
    private val diff: SceneDiff,
    private val onConfirm: () -> Unit,
) : GlintDialogScreen(McComponent.literal("Push Scenes")) {
    override fun buildDialog(dialog: FlowLayout) {
        dialog.child(GlintComponents.title(McComponent.literal("Push Scenes: $worldName")) as Component)

        dialog.child(
            Components
                .label(McComponent.literal("${diff.changeCount} change(s) to push"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )

        // Scrollable diff list
        val diffList = Containers.verticalFlow(Sizing.fixed(250), Sizing.content())
        diffList.gap(GlintTheme.GAP_SM)
        diffList.padding(Insets.vertical(GlintTheme.GAP_SM))

        for (scene in diff.toCreate) {
            diffList.child(diffLine("+ ${scene.name}", GlintTheme.TEXT_SUCCESS))
        }
        for (update in diff.toUpdate) {
            diffList.child(diffLine("~ ${update.local.name}", GlintTheme.TEXT_WARNING))
        }
        for (scene in diff.toRemove) {
            diffList.child(diffLine("- ${scene.name}", GlintTheme.TEXT_ERROR))
        }
        for (slug in diff.unchanged) {
            diffList.child(diffLine("  $slug", GlintTheme.TEXT_MUTED))
        }

        val scroll = Containers.verticalScroll(Sizing.fixed(260), Sizing.fixed(150), diffList)
        dialog.child(scroll as Component)

        dialog.child(
            GlintComponents.buttonRow(
                GlintComponents.wideButton(McComponent.literal("Push ${diff.changeCount} changes")) {
                    onConfirm()
                    minecraft?.setScreen(parent)
                },
                GlintComponents.cancelButton { minecraft?.setScreen(parent) },
            ) as Component,
        )
    }

    private fun diffLine(
        text: String,
        color: Int,
    ): Component =
        Components
            .label(McComponent.literal(text))
            .color(Color.ofRgb(color)) as Component

    override fun onClose() {
        minecraft?.setScreen(parent)
    }
}
