package com.xevion.glint.ui.base

import com.xevion.glint.scene.SceneState
import io.wispforest.owo.ui.component.ButtonComponent
import io.wispforest.owo.ui.component.Components
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
import net.minecraft.network.chat.Component as McComponent

/**
 * Reusable list-oriented component builders (rows, cards, collapsible sections).
 */
object GlintListComponents {
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
            row.withClick(onClick)
            row.withHover()
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
            child(leftContent)

            // Right side: buttons
            if (buttons.isNotEmpty()) {
                val buttonRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
                buttonRow.verticalAlignment(VerticalAlignment.CENTER)
                buttonRow.gap(GlintTheme.GAP_SM)
                for (btn in buttons) {
                    buttonRow.child(btn as Component)
                }
                child(buttonRow)
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
        if (isSelected) {
            row.surface(Surface.flat(GlintTheme.SELECTED_BG))
        }

        row.withClick(onSelect)
        if (!isSelected) {
            row.withHover()
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
    ): io.wispforest.owo.ui.component.LabelComponent =
        Components
            .label(McComponent.literal(text))
            .color(Color.ofRgb(color))

    /**
     * Creates a label for list item secondary text (counts, descriptions).
     */
    fun itemDetail(text: String): io.wispforest.owo.ui.component.LabelComponent =
        Components
            .label(McComponent.literal(text))
            .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY))

    /**
     * Overload for itemDetail with custom color.
     */
    fun itemDetail(
        text: String,
        color: Int,
    ): io.wispforest.owo.ui.component.LabelComponent =
        Components
            .label(McComponent.literal(text))
            .color(Color.ofRgb(color))

    /**
     * Creates an expand/collapse toggle button.
     */
    fun expandToggle(
        isExpanded: Boolean,
        onToggle: () -> Unit,
    ): ButtonComponent {
        val icon = if (isExpanded) "v" else ">"
        return GlintComponents.iconButton(icon, McComponent.literal(if (isExpanded) "Collapse" else "Expand")) {
            onToggle()
        }
    }

    /**
     * Creates a collapsible section with a clickable header that toggles visibility of its content.
     */
    fun collapsibleSection(
        title: String,
        defaultExpanded: Boolean = true,
        builder: FlowLayout.() -> Unit,
    ): FlowLayout {
        val outer = Containers.verticalFlow(Sizing.fill(100), Sizing.content())
        outer.gap(GlintTheme.GAP_SM)

        val content = Containers.verticalFlow(Sizing.fill(100), Sizing.content())
        content.gap(GlintTheme.GAP_SM)
        content.padding(Insets.left(GlintTheme.GAP_MD))

        var expanded = defaultExpanded

        val header = Containers.horizontalFlow(Sizing.fill(100), Sizing.content())
        header.gap(GlintTheme.GAP_SM)
        header.verticalAlignment(VerticalAlignment.CENTER)
        header.padding(Insets.vertical(2))
        header.cursorStyle(CursorStyle.HAND)
        header.withHover()

        val indicator = Components.label(McComponent.literal(if (expanded) "v" else ">"))
        indicator.color(Color.ofRgb(GlintTheme.TEXT_SECONDARY))

        val titleLabel = Components.label(McComponent.literal(title))
        titleLabel.color(Color.ofRgb(GlintTheme.TEXT_PRIMARY))

        header.child(indicator)
        header.child(titleLabel)

        header.mouseDown().subscribe { _, _, button ->
            if (button == 0) {
                expanded = !expanded
                indicator.text(McComponent.literal(if (expanded) "v" else ">"))
                if (expanded) {
                    if (content.parent() == null) outer.child(content)
                } else {
                    outer.removeChild(content)
                }
                true
            } else {
                false
            }
        }

        outer.child(header)

        content.builder()
        if (expanded) {
            outer.child(content)
        }

        return outer
    }

    /**
     * Creates a world card for the master grid.
     * Horizontal layout: thumbnail placeholder left, text right.
     */
    fun worldCard(
        name: String,
        sceneCount: Int,
        status: String,
        isSelected: Boolean,
        onClick: () -> Unit,
    ): FlowLayout {
        val card = Containers.horizontalFlow(Sizing.fill(100), Sizing.fixed(GlintTheme.CARD_HEIGHT))
        card.padding(GlintTheme.paddingSm())
        card.gap(GlintTheme.GAP_MD)
        card.verticalAlignment(VerticalAlignment.CENTER)
        if (isSelected) {
            card.surface(Surface.flat(GlintTheme.SELECTED_BG))
        } else {
            card.surface(Surface.flat(GlintTheme.CARD_BG))
        }

        // Thumbnail: first letter of world name, colored by status
        val thumbnail = Containers.verticalFlow(Sizing.fixed(GlintTheme.CARD_THUMBNAIL_SIZE), Sizing.fixed(GlintTheme.CARD_THUMBNAIL_SIZE))
        thumbnail.surface(Surface.flat(GlintTheme.THUMBNAIL_BG))
        thumbnail.horizontalAlignment(HorizontalAlignment.CENTER)
        thumbnail.verticalAlignment(VerticalAlignment.CENTER)
        val letter = name.firstOrNull()?.uppercase() ?: "?"
        thumbnail.child(Components.label(McComponent.literal(letter)).color(Color.ofRgb(statusColor(status))))
        card.child(thumbnail)

        // Text content (right)
        val textContainer = Containers.verticalFlow(Sizing.expand(), Sizing.content())
        textContainer.gap(GlintTheme.GAP_SM)
        textContainer.child(itemLabel(name))
        textContainer.child(itemDetail("$sceneCount scenes"))
        textContainer.child(itemDetail(status, statusColor(status)))
        card.child(textContainer)

        card.withClick(onClick)
        if (!isSelected) {
            card.withHover(restSurface = Surface.flat(GlintTheme.CARD_BG))
        }

        return card
    }

    /**
     * Creates a scene card for the master list.
     * Horizontal layout: status thumbnail left, text right.
     */
    fun sceneCard(
        name: String,
        dimension: String,
        state: SceneState,
        presetCount: Int,
        isSelected: Boolean,
        isLoaded: Boolean,
        needsPush: Boolean,
        onClick: () -> Unit,
    ): FlowLayout {
        val card = Containers.horizontalFlow(Sizing.fill(100), Sizing.fixed(GlintTheme.CARD_HEIGHT))
        card.padding(GlintTheme.paddingSm())
        card.gap(GlintTheme.GAP_MD)
        card.verticalAlignment(VerticalAlignment.CENTER)
        if (isSelected) {
            card.surface(Surface.flat(GlintTheme.SELECTED_BG))
        } else if (isLoaded) {
            card.surface(Surface.flat(GlintTheme.CARD_BG_LOADED))
        } else {
            card.surface(Surface.flat(GlintTheme.CARD_BG))
        }

        val statusIcon =
            when {
                needsPush -> "^"
                state == SceneState.SYNCED -> "="
                else -> "*"
            }
        val statusColor =
            when {
                needsPush -> GlintTheme.TEXT_WARNING
                state == SceneState.SYNCED -> GlintTheme.TEXT_SUCCESS
                else -> GlintTheme.TEXT_INFO
            }
        val thumbnail = Containers.verticalFlow(Sizing.fixed(GlintTheme.CARD_THUMBNAIL_SIZE), Sizing.fixed(GlintTheme.CARD_THUMBNAIL_SIZE))
        thumbnail.surface(Surface.flat(GlintTheme.THUMBNAIL_BG))
        thumbnail.horizontalAlignment(HorizontalAlignment.CENTER)
        thumbnail.verticalAlignment(VerticalAlignment.CENTER)
        thumbnail.child(Components.label(McComponent.literal(statusIcon)).color(Color.ofRgb(statusColor)))
        card.child(thumbnail)

        val textContainer = Containers.verticalFlow(Sizing.expand(), Sizing.content())
        textContainer.gap(GlintTheme.GAP_SM)
        textContainer.child(itemLabel(name))
        val dimensionShort =
            dimension
                .substringAfter(":")
                .replaceFirstChar { it.uppercase() }
        val detailText = if (presetCount > 0) "$dimensionShort • $presetCount presets" else dimensionShort
        textContainer.child(itemDetail(detailText))
        card.child(textContainer)

        card.withClick(onClick)
        if (!isSelected && !isLoaded) {
            card.withHover(restSurface = Surface.flat(GlintTheme.CARD_BG))
        }

        return card
    }

    /**
     * Helper to get status color.
     */
    private fun statusColor(status: String): Int =
        when (status) {
            "synced" -> GlintTheme.TEXT_SUCCESS
            "local" -> GlintTheme.TEXT_INFO
            "stale" -> GlintTheme.TEXT_WARNING
            "remote" -> GlintTheme.TEXT_SECONDARY
            else -> GlintTheme.TEXT_MUTED
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
