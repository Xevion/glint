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
import net.minecraft.client.gui.components.Tooltip
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
            ) as Component,
        )

        return content
    }

    // ============================================
    // List Item Components
    // ============================================

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
        tooltip?.let { btn.setTooltip(Tooltip.create(it)) }
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
        tooltip?.let { btn.setTooltip(Tooltip.create(it)) }
        return btn
    }

    /**
     * Creates a clickable list item row.
     * @param indent Horizontal indentation in pixels
     * @param onClick Handler for click events (null for non-clickable rows)
     * @param builder Lambda to build row contents
     */
    fun listItemRow(
        indent: Int = 0,
        height: Int = GlintTheme.ITEM_HEIGHT,
        onClick: (() -> Unit)? = null,
        builder: FlowLayout.() -> Unit,
    ): FlowLayout {
        val row = Containers.horizontalFlow(Sizing.fill(100), Sizing.fixed(height))
        row.verticalAlignment(VerticalAlignment.CENTER)
        row.padding(Insets.left(indent))
        row.gap(GlintTheme.GAP_SM)

        if (onClick != null) {
            row.cursorStyle(CursorStyle.HAND)
            row.mouseDown().subscribe { _, _, button ->
                if (button == 0) {
                    onClick()
                    true
                } else {
                    false
                }
            }
            // Hover effect
            row.mouseEnter().subscribe {
                row.surface(Surface.flat(GlintTheme.HIGHLIGHT_BG))
                true
            }
            row.mouseLeave().subscribe {
                row.surface(Surface.BLANK)
                true
            }
        }

        row.builder()
        return row
    }

    /**
     * Creates a row with left content and right-aligned buttons.
     */
    fun listItemWithButtons(
        indent: Int = 0,
        height: Int = GlintTheme.ITEM_HEIGHT,
        onClick: (() -> Unit)? = null,
        contentBuilder: FlowLayout.() -> Unit,
        vararg buttons: ButtonComponent,
    ): FlowLayout =
        listItemRow(indent, height, onClick) {
            // Left side: content
            val leftContent = Containers.horizontalFlow(Sizing.expand(), Sizing.content())
            leftContent.verticalAlignment(VerticalAlignment.CENTER)
            leftContent.gap(GlintTheme.GAP_SM)
            leftContent.contentBuilder()
            child(leftContent as Component)

            // Right side: buttons
            if (buttons.isNotEmpty()) {
                val buttonRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
                buttonRow.verticalAlignment(VerticalAlignment.CENTER)
                buttonRow.gap(GlintTheme.GAP_SM)
                for (btn in buttons) {
                    buttonRow.child(btn as Component)
                }
                child(buttonRow as Component)
            }
        }

    /**
     * Creates a selectable list item with selection highlighting.
     */
    fun selectableRow(
        indent: Int = 0,
        height: Int = GlintTheme.ITEM_HEIGHT,
        isSelected: Boolean = false,
        onSelect: () -> Unit,
        builder: FlowLayout.() -> Unit,
    ): FlowLayout {
        val row = Containers.horizontalFlow(Sizing.fill(100), Sizing.fixed(height))
        row.verticalAlignment(VerticalAlignment.CENTER)
        row.padding(Insets.left(indent).add(GlintTheme.PADDING_SM, 0, GlintTheme.PADDING_SM, 0))
        row.gap(GlintTheme.GAP_SM)
        row.cursorStyle(CursorStyle.HAND)

        if (isSelected) {
            row.surface(Surface.flat(GlintTheme.SELECTED_BG))
        }

        row.mouseDown().subscribe { _, _, button ->
            if (button == 0) {
                onSelect()
                true
            } else {
                false
            }
        }

        // Hover effect (only if not selected)
        if (!isSelected) {
            row.mouseEnter().subscribe {
                row.surface(Surface.flat(GlintTheme.HIGHLIGHT_BG))
                true
            }
            row.mouseLeave().subscribe {
                row.surface(Surface.BLANK)
                true
            }
        }

        row.builder()
        return row
    }

    /**
     * Creates a label for list item names.
     */
    fun itemLabel(
        text: String,
        color: Int = GlintTheme.TEXT_PRIMARY,
    ): LabelComponent =
        Components
            .label(McComponent.literal(text))
            .color(Color.ofRgb(color))

    /**
     * Creates a label for list item secondary text (counts, descriptions).
     */
    fun itemDetail(text: String): LabelComponent =
        Components
            .label(McComponent.literal(text))
            .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY))

    /**
     * Creates an expand/collapse toggle button.
     */
    fun expandToggle(
        isExpanded: Boolean,
        onToggle: () -> Unit,
    ): ButtonComponent {
        val icon = if (isExpanded) "v" else ">"
        return iconButton(icon, McComponent.literal(if (isExpanded) "Collapse" else "Expand")) {
            onToggle()
        }
    }
}
