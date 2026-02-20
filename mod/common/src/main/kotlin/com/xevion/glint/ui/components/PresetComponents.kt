package com.xevion.glint.ui.components

import com.xevion.glint.scene.LocalPreset
import com.xevion.glint.scene.SceneFormatting
import com.xevion.glint.scene.Weather
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintListComponents
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.ButtonComponent
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.component.TextBoxComponent
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.Insets
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.Surface
import io.wispforest.owo.ui.core.VerticalAlignment
import net.minecraft.client.Minecraft
import net.minecraft.network.chat.Component as McComponent

/**
 * Component builders for preset rows and edit forms.
 */
object PresetComponents {
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
                GlintComponents.smallButton(
                    McComponent.literal("Apply"),
                    width = 40,
                    tooltip = McComponent.literal("Apply this preset's environment"),
                ) {
                    onApply()
                },
            )
        }
        buttons.add(
            GlintComponents.iconButton("E", tooltip = McComponent.literal("Edit preset")) { onEdit() },
        )
        if (!isDefault) {
            buttons.add(
                GlintComponents.iconButton("X", tooltip = McComponent.literal("Delete preset")) { onDelete() },
            )
        }

        return GlintListComponents.listItemWithButtons(
            contentBuilder = {
                child(GlintListComponents.itemLabel(preset.name))
                child(GlintListComponents.itemDetail(SceneFormatting.formatTime(preset.timeOfDayTicks)))
                child(
                    GlintListComponents.itemDetail(
                        preset.weather.replaceFirstChar { it.uppercase() },
                        when (preset.weather.lowercase()) {
                            "clear" -> GlintTheme.TEXT_SUCCESS
                            "rain" -> GlintTheme.TEXT_INFO
                            "thunder" -> GlintTheme.TEXT_WARNING
                            else -> GlintTheme.TEXT_SECONDARY
                        },
                    ),
                )
                if (preset.moonPhase != null) {
                    child(GlintListComponents.itemDetail(SceneFormatting.moonPhaseName(preset.moonPhase!!)))
                }
                if (isDefault) {
                    child(
                        GlintListComponents.itemDetail("Default", GlintTheme.TEXT_MUTED),
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
        form.surface(Surface.flat(GlintTheme.SURFACE_SUBTLE))

        val nameBox = Components.textBox(Sizing.fill(60), state.name)
        nameBox.setMaxLength(64)
        form.child(labeledRow("Name:", nameBox as Component))

        val timeBox = Components.textBox(Sizing.fixed(60), state.timeOfDay.toString())
        timeBox.setMaxLength(5)
        val timeRow = labeledRow("Time:", timeBox as Component)
        timeRow.child(GlintListComponents.itemDetail("(0-24000)"))
        form.child(timeRow)

        val weatherRow = Containers.horizontalFlow(Sizing.fill(100), Sizing.content())
        weatherRow.gap(GlintTheme.GAP_SM)
        weatherRow.verticalAlignment(VerticalAlignment.CENTER)
        weatherRow.child(GlintListComponents.itemDetail("Weather:"))

        val intensityBox = Components.textBox(Sizing.fixed(50), "%.1f".format(state.weatherIntensity))
        intensityBox.setMaxLength(4)
        val intensityRow = labeledRow("Intensity:", intensityBox as Component)
        intensityRow.child(GlintListComponents.itemDetail("(0.0-1.0)"))

        val syncIntensityVisibility = {
            insertChildAfter(form, intensityRow, weatherRow, state.weather != Weather.CLEAR)
        }

        val weatherBtn =
            GlintComponents.smallButton(McComponent.literal(state.weatherLabel()), width = 60) { btn ->
                state.cycleWeather()
                btn.message = McComponent.literal(state.weatherLabel())
                syncIntensityVisibility()
            }
        weatherRow.child(weatherBtn as Component)
        form.child(weatherRow)

        if (state.weather != Weather.CLEAR) form.child(intensityRow)

        val moonBtn =
            GlintComponents.smallButton(McComponent.literal(SceneFormatting.moonPhaseName(state.moonPhase)), width = 60) { btn ->
                state.moonPhase = (state.moonPhase + 1) % 8
                btn.message = McComponent.literal(SceneFormatting.moonPhaseName(state.moonPhase))
            }
        form.child(labeledRow("Moon:", moonBtn as Component))

        if (isSceneLoaded) {
            form.child(
                buildFromWorldButton(state, timeBox, intensityBox, moonBtn, weatherBtn, syncIntensityVisibility) as Component,
            )
        }

        val actionRow = Containers.horizontalFlow(Sizing.fill(100), Sizing.content())
        actionRow.gap(GlintTheme.GAP_SM)
        actionRow.child(
            GlintComponents.smallButton(McComponent.literal("Save"), width = 40) {
                buildPresetFromForm(initial, state, nameBox, timeBox, intensityBox)?.let(onSave)
            } as Component,
        )
        actionRow.child(
            GlintComponents.smallButton(McComponent.literal("Cancel"), width = 50) { onCancel() } as Component,
        )
        form.child(actionRow)

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

        fun weatherLabel(): String = weather.displayName

        fun cycleWeather() {
            weather = weather.next()
        }
    }

    private fun labeledRow(
        label: String,
        field: Component,
    ): FlowLayout {
        val row = Containers.horizontalFlow(Sizing.fill(100), Sizing.content())
        row.gap(GlintTheme.GAP_SM)
        row.verticalAlignment(VerticalAlignment.CENTER)
        row.child(GlintListComponents.itemDetail(label))
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
            parent.removeChild(child)
        } else if (child.parent() == null) {
            val children = parent.children().toList()
            val anchorIdx = children.indexOf(anchor)
            if (anchorIdx >= 0 && anchorIdx < children.size - 1) {
                val afterAnchor = children.subList(anchorIdx + 1, children.size).toList()
                for (c in afterAnchor) parent.removeChild(c)
                parent.child(child)
                for (c in afterAnchor) parent.child(c)
            } else {
                parent.child(child)
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
        GlintComponents.smallButton(
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
