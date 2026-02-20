package com.xevion.glint.ui.base

import io.wispforest.owo.ui.component.ButtonComponent
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.component.LabelComponent
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.CursorStyle
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.Insets
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.Surface
import io.wispforest.owo.ui.core.VerticalAlignment
import net.minecraft.network.chat.CommonComponents
import net.minecraft.network.chat.Component as McComponent

/**
 * Reusable component builders for consistent Glint UI styling.
 */
object GlintComponents {
    /**
     * Creates a standard button with consistent sizing.
     * Note: The button sizing is applied via owo-lib's component mixin system.
     */
    fun button(
        text: McComponent,
        onClick: (ButtonComponent) -> Unit,
    ): ButtonComponent {
        val btn = Components.button(text, onClick)
        // Cast to Component to access the mixin-injected sizing method
        (btn as Component).horizontalSizing(GlintTheme.buttonSizing())
        return btn
    }

    /**
     * Creates a wide button with consistent sizing.
     */
    fun wideButton(
        text: McComponent,
        onClick: (ButtonComponent) -> Unit,
    ): ButtonComponent {
        val btn = Components.button(text, onClick)
        (btn as Component).horizontalSizing(GlintTheme.buttonWideSizing())
        return btn
    }

    /**
     * Creates a cancel button with localized text.
     */
    fun cancelButton(onClick: (ButtonComponent) -> Unit): ButtonComponent = button(CommonComponents.GUI_CANCEL, onClick)

    /**
     * Creates a horizontal row of two buttons with consistent spacing.
     */
    fun buttonRow(
        leftButton: ButtonComponent,
        rightButton: ButtonComponent,
    ): FlowLayout =
        Containers
            .horizontalFlow(Sizing.content(), Sizing.content())
            .child(leftButton as Component)
            .child(rightButton as Component)
            .gap(GlintTheme.GAP_MD)

    /**
     * Creates a title label with primary color.
     */
    fun title(text: McComponent): LabelComponent =
        Components
            .label(text)
            .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY))

    /**
     * Creates a subtitle/description label with muted color.
     */
    fun subtitle(text: McComponent): LabelComponent =
        Components
            .label(text)
            .color(Color.ofRgb(GlintTheme.TEXT_MUTED))

    /**
     * Creates a centered label.
     */
    fun centeredLabel(
        text: McComponent,
        color: Int = GlintTheme.TEXT_PRIMARY,
    ): LabelComponent =
        Components
            .label(text)
            .color(Color.ofRgb(color))
            .horizontalTextAlignment(HorizontalAlignment.CENTER)

    /**
     * Creates a vertically stacked set of labels (useful for multi-line text).
     */
    fun textBlock(vararg lines: Pair<McComponent, Int>): FlowLayout {
        val container = Containers.verticalFlow(Sizing.content(), Sizing.content())
        container.horizontalAlignment(HorizontalAlignment.CENTER)
        container.gap(GlintTheme.GAP_SM)

        for ((text, color) in lines) {
            container.child(centeredLabel(text, color))
        }

        return container
    }

    /**
     * Creates a confirmation dialog layout with title, description, and buttons.
     */
    fun confirmationContent(
        title: McComponent,
        description: McComponent,
        confirmText: McComponent,
        onConfirm: (ButtonComponent) -> Unit,
        onCancel: (ButtonComponent) -> Unit,
    ): FlowLayout {
        val content = Containers.verticalFlow(Sizing.content(), Sizing.content())
        content.horizontalAlignment(HorizontalAlignment.CENTER)
        content.gap(GlintTheme.GAP_MD)

        content.child(title(title))
        content.child(subtitle(description).margins(Insets.bottom(GlintTheme.GAP_MD)))
        content.child(
            buttonRow(
                button(confirmText, onConfirm),
                cancelButton(onCancel),
            ),
        )

        return content
    }

    /**
     * Creates a small icon button (20px square) for list item actions.
     */
    fun iconButton(
        icon: String,
        tooltip: McComponent? = null,
        onClick: (ButtonComponent) -> Unit,
    ): ButtonComponent {
        val btn = Components.button(McComponent.literal(icon), onClick)
        (btn as Component).sizing(Sizing.fixed(20), Sizing.fixed(20))
        tooltip?.let { (btn as Component).tooltip(it) }
        return btn
    }

    /**
     * Creates a small text button for list item actions.
     */
    fun smallButton(
        text: McComponent,
        width: Int = 40,
        tooltip: McComponent? = null,
        onClick: (ButtonComponent) -> Unit,
    ): ButtonComponent {
        val btn = Components.button(text, onClick)
        (btn as Component).sizing(Sizing.fixed(width), Sizing.fixed(20))
        tooltip?.let { (btn as Component).tooltip(it) }
        return btn
    }

    /**
     * Creates a tab button for the tab bar.
     */
    fun tabButton(
        text: McComponent,
        isActive: Boolean,
        onClick: () -> Unit,
    ): FlowLayout {
        val tab = Containers.horizontalFlow(Sizing.content(), Sizing.fixed(24))
        tab.padding(Insets.of(GlintTheme.PADDING_SM))
        tab.verticalAlignment(VerticalAlignment.CENTER)
        tab.horizontalAlignment(HorizontalAlignment.CENTER)
        val label =
            Components
                .label(text)
                .color(Color.ofRgb(if (isActive) GlintTheme.TEXT_PRIMARY else GlintTheme.TEXT_SECONDARY))
                .cursorStyle(CursorStyle.HAND)
        tab.child(label)

        if (isActive) {
            tab.surface(Surface.flat(GlintTheme.SELECTED_BG))
        }

        tab.withClick(onClick)
        if (!isActive) {
            tab.withHover()
        }

        return tab
    }

    /**
     * Vertical label-above-field container for form inputs.
     * Use the [builder] lambda to add child components to the container.
     */
    fun labeledField(
        label: String,
        builder: FlowLayout.() -> Unit,
    ): FlowLayout {
        val container = Containers.verticalFlow(Sizing.content(), Sizing.content())
        container.horizontalAlignment(HorizontalAlignment.LEFT)
        container.gap(GlintTheme.GAP_SM)
        container.child(
            Components
                .label(McComponent.literal(label))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)),
        )
        container.builder()
        return container
    }

    /** Wire up a left-click handler with hand cursor. */
    private fun FlowLayout.withClick(onClick: () -> Unit) {
        cursorStyle(CursorStyle.HAND)
        mouseDown().subscribe { _, _, button ->
            if (button == 0) {
                onClick()
                true
            } else {
                false
            }
        }
    }

    /** Wire up standard hover surface toggle. */
    private fun FlowLayout.withHover(
        hoverSurface: Surface = Surface.flat(GlintTheme.HIGHLIGHT_BG),
        restSurface: Surface = Surface.BLANK,
    ) {
        mouseEnter().subscribe {
            surface(hoverSurface)
            true
        }
        mouseLeave().subscribe {
            surface(restSurface)
            true
        }
    }
}
