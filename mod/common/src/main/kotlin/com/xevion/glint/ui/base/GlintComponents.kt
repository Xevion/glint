package com.xevion.glint.ui.base

import com.xevion.glint.scene.LocalPreset
import com.xevion.glint.scene.SceneFormatting
import com.xevion.glint.scene.SceneState
import com.xevion.glint.scene.Weather
import io.wispforest.owo.ui.component.ButtonComponent
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.component.LabelComponent
import io.wispforest.owo.ui.component.TextBoxComponent
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
import net.minecraft.client.Minecraft
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
        tab.cursorStyle(CursorStyle.HAND)

        val label =
            Components
                .label(text)
                .color(Color.ofRgb(if (isActive) GlintTheme.TEXT_PRIMARY else GlintTheme.TEXT_SECONDARY))
                .cursorStyle(CursorStyle.HAND)
        tab.child(label as Component)

        if (isActive) {
            tab.surface(Surface.flat(GlintTheme.SELECTED_BG))
        }

        tab.mouseDown().subscribe { _, _, button ->
            if (button == 0) {
                onClick()
                true
            } else {
                false
            }
        }

        if (!isActive) {
            tab.mouseEnter().subscribe {
                tab.surface(Surface.flat(GlintTheme.HIGHLIGHT_BG))
                true
            }
            tab.mouseLeave().subscribe {
                tab.surface(Surface.BLANK)
                true
            }
        }

        return tab
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
        card.cursorStyle(CursorStyle.HAND)

        if (isSelected) {
            card.surface(Surface.flat(GlintTheme.SELECTED_BG))
        } else {
            card.surface(Surface.flat(0x22FFFFFF)) // Subtle card background
        }

        // Thumbnail: first letter of world name, colored by status
        val thumbnail = Containers.verticalFlow(Sizing.fixed(GlintTheme.CARD_THUMBNAIL_SIZE), Sizing.fixed(GlintTheme.CARD_THUMBNAIL_SIZE))
        thumbnail.surface(Surface.flat(0x44888888))
        thumbnail.horizontalAlignment(HorizontalAlignment.CENTER)
        thumbnail.verticalAlignment(VerticalAlignment.CENTER)
        val letter = name.firstOrNull()?.uppercase() ?: "?"
        thumbnail.child(Components.label(McComponent.literal(letter)).color(Color.ofRgb(statusColor(status))) as Component)
        card.child(thumbnail as Component)

        // Text content (right)
        val textContainer = Containers.verticalFlow(Sizing.expand(), Sizing.content())
        textContainer.gap(GlintTheme.GAP_SM)
        textContainer.child(itemLabel(name) as Component)
        textContainer.child(itemDetail("$sceneCount scenes") as Component)
        textContainer.child(itemDetail(status, statusColor(status)) as Component)
        card.child(textContainer as Component)

        // Click handler
        card.mouseDown().subscribe { _, _, button ->
            if (button == 0) {
                onClick()
                true
            } else {
                false
            }
        }

        // Hover effect (only if not selected)
        if (!isSelected) {
            card.mouseEnter().subscribe {
                card.surface(Surface.flat(GlintTheme.HIGHLIGHT_BG))
                true
            }
            card.mouseLeave().subscribe {
                card.surface(Surface.flat(0x22FFFFFF))
                true
            }
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
        card.cursorStyle(CursorStyle.HAND)

        if (isSelected) {
            card.surface(Surface.flat(GlintTheme.SELECTED_BG))
        } else if (isLoaded) {
            card.surface(Surface.flat(0x33AAFFAA))
        } else {
            card.surface(Surface.flat(0x22FFFFFF))
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
        thumbnail.surface(Surface.flat(0x44888888))
        thumbnail.horizontalAlignment(HorizontalAlignment.CENTER)
        thumbnail.verticalAlignment(VerticalAlignment.CENTER)
        thumbnail.child(Components.label(McComponent.literal(statusIcon)).color(Color.ofRgb(statusColor)) as Component)
        card.child(thumbnail as Component)

        val textContainer = Containers.verticalFlow(Sizing.expand(), Sizing.content())
        textContainer.gap(GlintTheme.GAP_SM)
        textContainer.child(itemLabel(name) as Component)
        val dimensionShort =
            dimension
                .substringAfter(":")
                .replaceFirstChar { it.uppercase() }
        val detailText = if (presetCount > 0) "$dimensionShort • $presetCount presets" else dimensionShort
        textContainer.child(itemDetail(detailText) as Component)
        card.child(textContainer as Component)

        card.mouseDown().subscribe { _, _, button ->
            if (button == 0) {
                onClick()
                true
            } else {
                false
            }
        }

        if (!isSelected && !isLoaded) {
            card.mouseEnter().subscribe {
                card.surface(Surface.flat(GlintTheme.HIGHLIGHT_BG))
                true
            }
            card.mouseLeave().subscribe {
                card.surface(Surface.flat(0x22FFFFFF))
                true
            }
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
        header.cursorStyle(CursorStyle.HAND)
        header.padding(Insets.vertical(2))

        val indicator = Components.label(McComponent.literal(if (expanded) "v" else ">"))
        indicator.color(Color.ofRgb(GlintTheme.TEXT_SECONDARY))

        val titleLabel = Components.label(McComponent.literal(title))
        titleLabel.color(Color.ofRgb(GlintTheme.TEXT_PRIMARY))

        header.child(indicator as Component)
        header.child(titleLabel as Component)

        header.mouseDown().subscribe { _, _, button ->
            if (button == 0) {
                expanded = !expanded
                indicator.text(McComponent.literal(if (expanded) "v" else ">"))
                if (expanded) {
                    if (content.parent() == null) outer.child(content as Component)
                } else {
                    outer.removeChild(content as Component)
                }
                true
            } else {
                false
            }
        }

        outer.child(header as Component)

        content.builder()
        if (expanded) {
            outer.child(content as Component)
        }

        return outer
    }

    /**
     * Overload for itemDetail with custom color.
     */
    fun itemDetail(
        text: String,
        color: Int,
    ): LabelComponent =
        Components
            .label(McComponent.literal(text))
            .color(Color.ofRgb(color))

    /**
     * Creates a row displaying a preset with action buttons.
     */
    fun presetRow(
        preset: LocalPreset,
        isDefault: Boolean,
        isSceneLoaded: Boolean,
        onEdit: () -> Unit,
        onDelete: () -> Unit,
        onApply: () -> Unit,
    ): FlowLayout {
        val buttons = mutableListOf<ButtonComponent>()

        if (isSceneLoaded) {
            buttons.add(
                smallButton(McComponent.literal("Apply"), width = 40, tooltip = McComponent.literal("Apply this preset's environment")) {
                    onApply()
                },
            )
        }
        buttons.add(
            iconButton("E", tooltip = McComponent.literal("Edit preset")) { onEdit() },
        )
        if (!isDefault) {
            buttons.add(
                iconButton("X", tooltip = McComponent.literal("Delete preset")) { onDelete() },
            )
        }

        return listItemWithButtons(
            contentBuilder = {
                child(itemLabel(preset.name) as Component)
                child(itemDetail(SceneFormatting.formatTime(preset.timeOfDayTicks)) as Component)
                child(
                    itemDetail(
                        preset.weather.replaceFirstChar { it.uppercase() },
                        when (preset.weather.lowercase()) {
                            "clear" -> GlintTheme.TEXT_SUCCESS
                            "rain" -> GlintTheme.TEXT_INFO
                            "thunder" -> GlintTheme.TEXT_WARNING
                            else -> GlintTheme.TEXT_SECONDARY
                        },
                    ) as Component,
                )
                if (preset.moonPhase != null) {
                    child(itemDetail(SceneFormatting.moonPhaseName(preset.moonPhase!!)) as Component)
                }
                if (isDefault) {
                    child(
                        itemDetail("Default", GlintTheme.TEXT_MUTED) as Component,
                    )
                }
            },
            buttons = buttons.toTypedArray(),
        )
    }

    /**
     * Creates an inline form for editing or creating a preset.
     */
    fun presetEditForm(
        initial: LocalPreset?,
        isSceneLoaded: Boolean,
        onSave: (LocalPreset) -> Unit,
        onCancel: () -> Unit,
    ): FlowLayout {
        val state = PresetFormState(initial)
        val form = Containers.verticalFlow(Sizing.fill(100), Sizing.content())
        form.gap(GlintTheme.GAP_SM)
        form.padding(Insets.of(GlintTheme.PADDING_SM))
        form.surface(Surface.flat(0x33FFFFFF))

        val nameBox = Components.textBox(Sizing.fill(60), state.name)
        nameBox.setMaxLength(64)
        form.child(labeledRow("Name:", nameBox as Component) as Component)

        val timeBox = Components.textBox(Sizing.fixed(60), state.timeOfDay.toString())
        timeBox.setMaxLength(5)
        val timeRow = labeledRow("Time:", timeBox as Component)
        timeRow.child(itemDetail("(0-24000)") as Component)
        form.child(timeRow as Component)

        val weatherRow = Containers.horizontalFlow(Sizing.fill(100), Sizing.content())
        weatherRow.gap(GlintTheme.GAP_SM)
        weatherRow.verticalAlignment(VerticalAlignment.CENTER)
        weatherRow.child(itemDetail("Weather:") as Component)

        val intensityBox = Components.textBox(Sizing.fixed(50), "%.1f".format(state.weatherIntensity))
        intensityBox.setMaxLength(4)
        val intensityRow = labeledRow("Intensity:", intensityBox as Component)
        intensityRow.child(itemDetail("(0.0-1.0)") as Component)

        val syncIntensityVisibility = {
            insertChildAfter(form, intensityRow, weatherRow, state.weather != Weather.CLEAR)
        }

        val weatherBtn =
            smallButton(McComponent.literal(state.weatherLabel()), width = 60) { btn ->
                state.cycleWeather()
                btn.message = McComponent.literal(state.weatherLabel())
                syncIntensityVisibility()
            }
        weatherRow.child(weatherBtn as Component)
        form.child(weatherRow as Component)

        if (state.weather != Weather.CLEAR) form.child(intensityRow as Component)

        val moonBtn =
            smallButton(McComponent.literal(SceneFormatting.moonPhaseName(state.moonPhase)), width = 60) { btn ->
                state.moonPhase = (state.moonPhase + 1) % 8
                btn.message = McComponent.literal(SceneFormatting.moonPhaseName(state.moonPhase))
            }
        form.child(labeledRow("Moon:", moonBtn as Component) as Component)

        if (isSceneLoaded) {
            form.child(
                buildFromWorldButton(state, timeBox, intensityBox, moonBtn, weatherBtn, syncIntensityVisibility) as Component,
            )
        }

        val actionRow = Containers.horizontalFlow(Sizing.fill(100), Sizing.content())
        actionRow.gap(GlintTheme.GAP_SM)
        actionRow.child(
            smallButton(McComponent.literal("Save"), width = 40) {
                buildPresetFromForm(initial, state, nameBox, timeBox, intensityBox)?.let(onSave)
            } as Component,
        )
        actionRow.child(
            smallButton(McComponent.literal("Cancel"), width = 50) { onCancel() } as Component,
        )
        form.child(actionRow as Component)

        return form
    }

    /** Mutable form state for [presetEditForm], extracted to reduce function length. */
    private class PresetFormState(
        initial: LocalPreset?,
    ) {
        var name = initial?.name ?: ""
        var timeOfDay = initial?.timeOfDayTicks ?: 6000
        var weather = initial?.weather?.let { Weather.fromString(it) } ?: Weather.CLEAR
        var weatherIntensity = initial?.weatherIntensity ?: 0.0
        var moonPhase = initial?.moonPhase ?: 0

        fun weatherLabel(): String = weather.name.lowercase().replaceFirstChar { it.uppercase() }

        fun cycleWeather() {
            weather =
                when (weather) {
                    Weather.CLEAR -> Weather.RAIN
                    Weather.RAIN -> Weather.THUNDER
                    Weather.THUNDER -> Weather.CLEAR
                }
        }
    }

    private fun labeledRow(
        label: String,
        field: Component,
    ): FlowLayout {
        val row = Containers.horizontalFlow(Sizing.fill(100), Sizing.content())
        row.gap(GlintTheme.GAP_SM)
        row.verticalAlignment(VerticalAlignment.CENTER)
        row.child(itemDetail(label) as Component)
        row.child(field)
        return row
    }

    /**
     * Inserts or removes [child] from [parent] immediately after [anchor].
     * FlowLayout lacks index-based insertion so we reorder children manually.
     */
    private fun insertChildAfter(
        parent: FlowLayout,
        child: FlowLayout,
        anchor: FlowLayout,
        visible: Boolean,
    ) {
        if (!visible) {
            parent.removeChild(child as Component)
        } else if (child.parent() == null) {
            val children = parent.children().toList()
            val anchorIdx = children.indexOf(anchor as Component)
            if (anchorIdx >= 0 && anchorIdx < children.size - 1) {
                val afterAnchor = children.subList(anchorIdx + 1, children.size).toList()
                for (c in afterAnchor) parent.removeChild(c)
                parent.child(child as Component)
                for (c in afterAnchor) parent.child(c)
            } else {
                parent.child(child as Component)
            }
        }
    }

    private fun buildFromWorldButton(
        state: PresetFormState,
        timeBox: TextBoxComponent,
        intensityBox: TextBoxComponent,
        moonBtn: ButtonComponent,
        weatherBtn: ButtonComponent,
        syncIntensityVisibility: () -> Unit,
    ): ButtonComponent =
        smallButton(
            McComponent.literal("From World"),
            width = 75,
            tooltip = McComponent.literal("Pull current world environment into form"),
        ) {
            val mc = Minecraft.getInstance()
            val server = mc.singleplayerServer ?: return@smallButton
            val overworld = server.overworld()
            val dayTime = overworld.dayTime

            state.timeOfDay = (dayTime % 24000L).toInt()
            timeBox.text(state.timeOfDay.toString())

            state.moonPhase = ((dayTime / 24000L) % 8).toInt()
            moonBtn.message = McComponent.literal("Phase ${state.moonPhase}")

            state.weather =
                when {
                    overworld.isThundering -> Weather.THUNDER
                    overworld.isRaining -> Weather.RAIN
                    else -> Weather.CLEAR
                }
            weatherBtn.message = McComponent.literal(state.weatherLabel())

            state.weatherIntensity = overworld.getRainLevel(1f).toDouble()
            intensityBox.text("%.1f".format(state.weatherIntensity))

            syncIntensityVisibility()
        }

    private fun buildPresetFromForm(
        initial: LocalPreset?,
        state: PresetFormState,
        nameBox: TextBoxComponent,
        timeBox: TextBoxComponent,
        intensityBox: TextBoxComponent,
    ): LocalPreset? {
        val finalName = nameBox.value.trim()
        if (finalName.isEmpty()) return null

        val finalTime = timeBox.value.toIntOrNull()?.coerceIn(0, 24000) ?: state.timeOfDay
        val finalIntensity =
            if (state.weather != Weather.CLEAR) {
                intensityBox.value.toDoubleOrNull()?.coerceIn(0.0, 1.0) ?: state.weatherIntensity
            } else {
                0.0
            }

        val slug =
            initial?.slug
                ?: finalName
                    .lowercase()
                    .replace(Regex("[^a-z0-9]+"), "-")
                    .trimEnd('-')
                    .trimStart('-')

        return LocalPreset(
            name = finalName,
            slug = slug,
            timeOfDayTicks = finalTime,
            weather = state.weather.toMinecraftString(),
            weatherIntensity = finalIntensity,
            moonPhase = state.moonPhase,
            backendPresetId = initial?.backendPresetId,
        )
    }
}
