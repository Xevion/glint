package com.xevion.glint.ui

import com.xevion.glint.Loggers
import com.xevion.glint.scene.LocalSceneMetadata
import com.xevion.glint.scene.LocalSceneStore
import com.xevion.glint.scene.SceneExporter
import com.xevion.glint.scene.SceneFormatting
import com.xevion.glint.scene.SceneInjection
import com.xevion.glint.scene.ScenePackageMeta
import com.xevion.glint.scene.SceneState
import com.xevion.glint.scene.WEATHER_DURATION_TICKS
import com.xevion.glint.scene.Weather
import com.xevion.glint.scene.applyParameters
import com.xevion.glint.scene.levels
import com.xevion.glint.scene.scenePackageJson
import com.xevion.glint.scene.snapLevels
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintPanelScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.ButtonComponent
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.component.LabelComponent
import io.wispforest.owo.ui.component.SlimSliderComponent
import io.wispforest.owo.ui.component.TextBoxComponent
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.Insets
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.VerticalAlignment
import net.minecraft.client.Minecraft
import net.minecraft.client.gui.screens.Screen
import net.minecraft.world.level.GameRules
import java.nio.file.Path
import java.security.MessageDigest
import java.time.Instant
import java.util.concurrent.CompletableFuture
import java.util.zip.ZipFile
import net.minecraft.network.chat.Component as McComponent

class ExportSceneDialog(
    private val parentScreen: Screen,
    private val onExportComplete: () -> Unit,
) : GlintPanelScreen(McComponent.literal("Export Scene")) {
    companion object {
        private val log = Loggers.Ui.get()
        private val SLUG_PATTERN = Regex("[a-z0-9-]+")
        private val NON_SLUG_CHARS = Regex("[^a-z0-9\\s-]")
        private val WHITESPACE_RUNS = Regex("\\s+")
        private val HYPHEN_RUNS = Regex("-+")

        private const val INPUT_WIDTH = 180
        private const val SLIDER_WIDTH = 180
        private const val RD_MIN = 2.0
        private const val RD_MAX = 32.0
        private const val FOV_MIN = 30.0
        private const val FOV_MAX = 110.0
        private const val TIME_MIN = 0.0
        private const val TIME_MAX = 23999.0
        private const val MOON_MIN = 0.0

        /** Minecraft has 8 moon phases (0–7). */
        private const val MOON_MAX = 7.0
    }

    /**
     * Snapshot of environment state captured when the dialog opens.
     * Used to restore the world to its original state on close.
     */
    private data class EnvironmentSnapshot(
        val dayTime: Long,
        val isRaining: Boolean,
        val isThundering: Boolean,
        val rainLevel: Float,
        val thunderLevel: Float,
        val renderDistance: Int,
        val simulationDistance: Int,
        val fov: Int,
        val doDaylightCycle: Boolean,
        val doWeatherCycle: Boolean,
    ) {
        companion object {
            fun capture(): EnvironmentSnapshot? {
                val mc = Minecraft.getInstance()
                val server = mc.singleplayerServer ?: return null
                val overworld = server.overworld()
                val options = mc.options

                return EnvironmentSnapshot(
                    dayTime = overworld.dayTime,
                    isRaining = overworld.levelData.isRaining,
                    isThundering = overworld.isThundering,
                    rainLevel = overworld.getRainLevel(1f),
                    thunderLevel = overworld.getThunderLevel(1f),
                    renderDistance = options.renderDistance().get(),
                    simulationDistance = options.simulationDistance().get(),
                    fov = options.fov().get(),
                    doDaylightCycle =
                        overworld.gameRules
                            .getRule(GameRules.RULE_DAYLIGHT)
                            .get(),
                    doWeatherCycle =
                        overworld.gameRules
                            .getRule(GameRules.RULE_WEATHER_CYCLE)
                            .get(),
                )
            }
        }

        fun restore() {
            val mc = Minecraft.getInstance()
            val server = mc.singleplayerServer ?: return
            val options = mc.options

            server.execute {
                val overworld = server.overworld()
                overworld.dayTime = dayTime

                if (isThundering) {
                    overworld.setWeatherParameters(0, WEATHER_DURATION_TICKS, true, true)
                } else if (isRaining) {
                    overworld.setWeatherParameters(0, WEATHER_DURATION_TICKS, true, false)
                } else {
                    overworld.setWeatherParameters(WEATHER_DURATION_TICKS, 0, false, false)
                }

                overworld.setRainLevel(rainLevel)
                overworld.setThunderLevel(thunderLevel)

                overworld.gameRules.getRule(GameRules.RULE_DAYLIGHT).set(doDaylightCycle, server)
                overworld.gameRules.getRule(GameRules.RULE_WEATHER_CYCLE).set(doWeatherCycle, server)
            }

            mc.level?.let { clientLevel ->
                clientLevel.setRainLevel(rainLevel)
                clientLevel.setThunderLevel(thunderLevel)
            }

            options.fov().set(fov)

            val currentRd = options.renderDistance().get()
            if (currentRd != renderDistance) {
                options.renderDistance().set(renderDistance)
                options.simulationDistance().set(simulationDistance)
                mc.levelRenderer.allChanged()
            }
        }
    }

    private lateinit var nameInput: TextBoxComponent
    private lateinit var slugInput: TextBoxComponent
    private lateinit var descriptionInput: TextBoxComponent
    private lateinit var exportButton: ButtonComponent
    private lateinit var errorLabel: LabelComponent

    private lateinit var rdLabel: LabelComponent
    private lateinit var fovLabel: LabelComponent
    private lateinit var timeLabel: LabelComponent
    private lateinit var moonLabel: LabelComponent
    private lateinit var fovSlider: SlimSliderComponent

    private var slugManuallyEdited = false
    private var snapshot: EnvironmentSnapshot? = null

    private var currentRenderDistance: Int = 16
    private var currentFov: Int = 70
    private var currentTime: Int = 6000
    private var currentWeather: Weather = Weather.CLEAR
    private var currentMoonPhase: Int = 0

    private var entityTicksFrozen = false
    private var isExporting = false
    private lateinit var progressLabel: LabelComponent

    override fun buildPanel(panel: FlowLayout) {
        val mc = Minecraft.getInstance()

        if (snapshot == null) {
            snapshot = EnvironmentSnapshot.capture()

            val options = mc.options
            currentRenderDistance = options.renderDistance().get()
            currentFov = options.fov().get()
            mc.level?.let { level ->
                currentTime = (level.dayTime % 24000).toInt()
                currentWeather =
                    when {
                        level.isThundering -> Weather.THUNDER
                        level.levelData.isRaining -> Weather.RAIN
                        else -> Weather.CLEAR
                    }
                currentMoonPhase = level.getMoonPhase()
            }

            freezeWorldState()
        }

        if (isExporting) {
            buildProgressView(panel)
        } else {
            buildFormView(panel)
        }
    }

    private fun buildFormView(panel: FlowLayout) {
        panel.child(
            GlintComponents.labeledField("Name:") {
                nameInput = Components.textBox(Sizing.fixed(INPUT_WIDTH))
                nameInput.setMaxLength(128)
                nameInput.onChanged().subscribe {
                    if (!slugManuallyEdited) {
                        slugInput.text(generateSlug(nameInput.value))
                    }
                    validateInput()
                }
                child(nameInput as Component)
            } as Component,
        )

        panel.child(
            GlintComponents.labeledField("Slug (URL-safe):") {
                slugInput = Components.textBox(Sizing.fixed(INPUT_WIDTH))
                slugInput.setMaxLength(64)
                slugInput.onChanged().subscribe {
                    slugManuallyEdited = slugInput.value != generateSlug(nameInput.value)
                    validateInput()
                }
                child(slugInput as Component)
            } as Component,
        )

        panel.child(
            GlintComponents.labeledField("Description (optional):") {
                descriptionInput = Components.textBox(Sizing.fixed(INPUT_WIDTH))
                descriptionInput.setMaxLength(256)
                descriptionInput.setSuggestion("Brief description")
                child(descriptionInput as Component)
            } as Component,
        )

        rdLabel = Components.label(renderDistanceText())
        addSlider(
            panel,
            rdLabel,
            RD_MIN,
            RD_MAX,
            currentRenderDistance.toDouble(),
            onChange = { value ->
                currentRenderDistance = value
                rdLabel.text(renderDistanceText())
            },
            onRelease = { applyRenderDistance() },
        )

        fovLabel = Components.label(fovText())
        fovSlider =
            addSlider(
                panel,
                fovLabel,
                FOV_MIN,
                FOV_MAX,
                currentFov.toDouble(),
                onChange = { value ->
                    currentFov = value
                    fovLabel.text(fovText())
                    applyFov()
                },
            )

        timeLabel = Components.label(timeText())
        addSlider(
            panel,
            timeLabel,
            TIME_MIN,
            TIME_MAX,
            currentTime.toDouble(),
            onChange = { value ->
                currentTime = value
                timeLabel.text(timeText())
                applyTimeAndMoon()
            },
        )

        panel.child(buildWeatherRow() as Component)

        moonLabel = Components.label(moonText())
        addSlider(
            panel,
            moonLabel,
            MOON_MIN,
            MOON_MAX,
            currentMoonPhase.toDouble(),
            onChange = { value ->
                currentMoonPhase = value
                moonLabel.text(moonText())
                applyTimeAndMoon()
            },
        )

        panel.child(buildFreezeRow() as Component)

        errorLabel = Components.label(McComponent.literal(""))
        errorLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
        errorLabel.margins(Insets.top(GlintTheme.GAP_SM))
        panel.child(errorLabel as Component)

        exportButton = GlintComponents.button(McComponent.literal("Export")) { doExport() }
        exportButton.active = false
        panel.child(
            GlintComponents.buttonRow(
                exportButton,
                GlintComponents.cancelButton { minecraft?.setScreen(parentScreen) },
            ) as Component,
        )

        validateInput()
        uiAdapter.rootComponent.focusHandler()?.focus(nameInput as Component, null)
    }

    private fun buildWeatherRow(): FlowLayout {
        val row = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        row.gap(GlintTheme.GAP_SM)
        row.verticalAlignment(VerticalAlignment.CENTER)
        row.child(
            Components
                .label(McComponent.literal("Weather:"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        val button =
            GlintComponents.smallButton(
                McComponent.literal(currentWeather.displayName),
                width = 60,
                tooltip = McComponent.literal("Click to cycle weather"),
            ) { btn ->
                currentWeather = currentWeather.next()
                btn.setMessage(McComponent.literal(currentWeather.displayName))
                applyWeather()
            }
        row.child(button as Component)
        return row
    }

    private fun buildFreezeRow(): FlowLayout {
        val row = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        row.gap(GlintTheme.GAP_SM)
        row.verticalAlignment(VerticalAlignment.CENTER)
        row.child(
            Components
                .label(McComponent.literal("Freeze Entities:"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        val button =
            GlintComponents.smallButton(
                McComponent.literal(if (entityTicksFrozen) "On" else "Off"),
                width = 40,
                tooltip = McComponent.literal("Freeze entity movement and animations"),
            ) { btn ->
                entityTicksFrozen = !entityTicksFrozen
                if (entityTicksFrozen) {
                    SceneInjection.freezeEntityTick()
                } else {
                    SceneInjection.unfreezeEntityTick()
                }
                btn.setMessage(McComponent.literal(if (entityTicksFrozen) "On" else "Off"))
            }
        row.child(button as Component)
        return row
    }

    private fun buildProgressView(panel: FlowLayout) {
        panel.child(GlintComponents.title(McComponent.literal("Exporting Scene...")) as Component)

        progressLabel = Components.label(McComponent.literal("Starting export..."))
        progressLabel.color(Color.ofRgb(GlintTheme.TEXT_SECONDARY))
        panel.child(progressLabel as Component)
    }

    private fun addSlider(
        panel: FlowLayout,
        label: LabelComponent,
        min: Double,
        max: Double,
        initial: Double,
        onChange: (Int) -> Unit,
        onRelease: (() -> Unit)? = null,
    ): SlimSliderComponent {
        label.color(Color.ofRgb(GlintTheme.TEXT_SECONDARY))
        panel.child(label as Component)
        val slider =
            Components
                .slimSlider(SlimSliderComponent.Axis.HORIZONTAL)
                .min(min)
                .max(max)
                .stepSize(1.0)
                .value(initial)
        (slider as Component).horizontalSizing(Sizing.fixed(SLIDER_WIDTH))
        slider.onChanged().subscribe { value -> onChange(value.toInt()) }
        onRelease?.let { callback -> slider.onSlideEnd().subscribe { callback() } }
        panel.child(slider as Component)
        return slider
    }

    private fun renderDistanceText(): McComponent = McComponent.literal("Render Distance ($currentRenderDistance chunks)")

    private fun fovText(): McComponent = McComponent.literal("FOV ($currentFov)")

    private fun timeText(): McComponent = McComponent.literal("Time: ${SceneFormatting.formatTime(currentTime)}")

    private fun moonText(): McComponent = McComponent.literal("Moon: ${SceneFormatting.moonPhaseName(currentMoonPhase)}")

    private fun applyRenderDistance() {
        val mc = Minecraft.getInstance()
        val options = mc.options
        if (options.renderDistance().get() != currentRenderDistance) {
            options.renderDistance().set(currentRenderDistance)
            // Coupled intentionally: scene exports use the same render/sim distance
            // so entities tick at the full visible range.
            options.simulationDistance().set(currentRenderDistance)
            mc.levelRenderer.allChanged()
        }
    }

    private fun applyFov() {
        Minecraft
            .getInstance()
            .options
            .fov()
            .set(currentFov)
    }

    private fun applyTimeAndMoon() {
        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer ?: return
        server.execute {
            val overworld = server.overworld()
            overworld.dayTime = (currentMoonPhase.toLong() * 24000L) + currentTime.toLong()
        }
    }

    private fun applyWeather() {
        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer ?: return
        server.execute {
            val overworld = server.overworld()
            currentWeather.applyParameters(overworld)
            currentWeather.snapLevels(overworld)
        }
        // Snap client-side immediately for instant visual feedback
        val (rain, thunder) = currentWeather.levels()
        mc.level?.let { clientLevel ->
            clientLevel.setRainLevel(rain)
            clientLevel.setThunderLevel(thunder)
        }
    }

    private fun freezeWorldState() {
        val mc = Minecraft.getInstance()
        val server = mc.singleplayerServer ?: return
        server.execute {
            val overworld = server.overworld()
            overworld.gameRules.getRule(GameRules.RULE_DAYLIGHT).set(false, server)
            overworld.gameRules.getRule(GameRules.RULE_WEATHER_CYCLE).set(false, server)
        }
    }

    override fun onBackgroundScroll(amount: Double): Boolean {
        val newFov = (currentFov - amount.toInt()).coerceIn(FOV_MIN.toInt(), FOV_MAX.toInt())
        if (newFov == currentFov) return false
        currentFov = newFov
        fovLabel.text(fovText())
        fovSlider.value(currentFov.toDouble())
        applyFov()
        return true
    }

    override fun onClose() {
        minecraft?.setScreen(parentScreen)
    }

    /** Always restores the original environment on close — including after successful export. */
    override fun onPanelClosed() {
        if (entityTicksFrozen) {
            SceneInjection.unfreezeEntityTick()
            entityTicksFrozen = false
        }
        snapshot?.restore()
    }

    private fun validateInput() {
        val name = nameInput.value.trim()
        val slug = slugInput.value.trim()

        if (name.isEmpty()) {
            errorLabel.text(McComponent.literal("Name is required"))
            exportButton.active = false
            return
        }

        if (slug.isEmpty()) {
            errorLabel.text(McComponent.literal("Slug is required"))
            exportButton.active = false
            return
        }

        if (!slug.matches(SLUG_PATTERN)) {
            errorLabel.text(McComponent.literal("Slug: lowercase, numbers, hyphens only"))
            exportButton.active = false
            return
        }

        if (LocalSceneStore.allSlugs().contains(slug)) {
            errorLabel.text(McComponent.literal("Slug '$slug' already exists"))
            exportButton.active = false
            return
        }

        errorLabel.text(McComponent.literal(""))
        exportButton.active = true
    }

    private fun doExport() {
        val name = nameInput.value.trim()
        val slug = slugInput.value.trim()
        val description = descriptionInput.value.trim().ifEmpty { null }

        isExporting = true
        uiAdapter.rootComponent.clearChildren()
        build(uiAdapter.rootComponent)

        CompletableFuture
            .supplyAsync {
                SceneExporter.export(slug)
            }.thenAcceptAsync(
                { result ->
                    result.fold(
                        onSuccess = { zipPath ->
                            handleExportSuccess(zipPath, name, slug, description, currentRenderDistance)
                        },
                        onFailure = { error ->
                            showExportError(error.message ?: "Unknown error")
                        },
                    )
                },
                Minecraft.getInstance(),
            )
    }

    private fun handleExportSuccess(
        zipPath: Path,
        name: String,
        slug: String,
        description: String?,
        renderDistance: Int,
    ) {
        runCatching {
            val meta = parseMetaFromZip(zipPath)
            val packageFile = zipPath.toFile()
            val hash = computeFileHash(packageFile)
            val sizeBytes = packageFile.length()

            val targetPath = LocalSceneStore.packagePath(slug)
            targetPath.parentFile.mkdirs()
            packageFile.copyTo(targetPath, overwrite = true)
            packageFile.delete()

            LocalSceneStore.registerExport(
                slug,
                LocalSceneMetadata(
                    slug = slug,
                    name = name,
                    description = description,
                    dimension = meta.dimension,
                    minecraftVersion = meta.minecraftVersion,
                    state = SceneState.LOCAL,
                    exportedAt = Instant.now().toString(),
                    camera = meta.camera,
                    fov = meta.fov,
                    renderDistance = renderDistance,
                    environment = meta.environment,
                    chunkBounds = meta.chunkBounds,
                    entityCount = 0,
                    packageHash = hash,
                    packageSizeBytes = sizeBytes,
                ),
            )

            log.info("Exported scene") {
                "slug" to slug
                "size" to sizeBytes
            }
        }.fold(
            onSuccess = { showExportComplete() },
            onFailure = { error ->
                showExportError(error.message ?: "Failed to finalize export")
            },
        )
    }

    private fun showExportComplete() {
        progressLabel.text(McComponent.literal("Export complete"))
        progressLabel.color(Color.ofRgb(GlintTheme.TEXT_SUCCESS))

        val parent = (progressLabel as Component).parent() as? FlowLayout ?: return
        parent.child(
            GlintComponents.button(McComponent.literal("Close")) {
                onExportComplete()
                minecraft?.setScreen(parentScreen)
            } as Component,
        )
    }

    private fun showExportError(message: String) {
        log.error("Scene export failed") { "error" to message }

        progressLabel.text(McComponent.literal("Export failed: $message"))
        progressLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))

        val parent = (progressLabel as Component).parent() as? FlowLayout ?: return
        parent.child(
            GlintComponents.buttonRow(
                GlintComponents.button(McComponent.literal("Retry")) {
                    isExporting = false
                    uiAdapter.rootComponent.clearChildren()
                    build(uiAdapter.rootComponent)
                },
                GlintComponents.cancelButton { minecraft?.setScreen(parentScreen) },
            ) as Component,
        )
    }

    private fun generateSlug(name: String): String =
        name
            .lowercase()
            .replace(NON_SLUG_CHARS, "")
            .trim()
            .replace(WHITESPACE_RUNS, "-")
            .replace(HYPHEN_RUNS, "-")
            .trim('-')
            .take(64)

    private fun parseMetaFromZip(zipPath: Path): ScenePackageMeta {
        ZipFile(zipPath.toFile()).use { zip ->
            val entry = zip.getEntry("meta.json") ?: error("No meta.json in package")
            val json = zip.getInputStream(entry).use { it.bufferedReader().readText() }
            return scenePackageJson.decodeFromString(ScenePackageMeta.serializer(), json)
        }
    }

    private fun computeFileHash(file: java.io.File): String {
        val digest = MessageDigest.getInstance("SHA-256")
        file.inputStream().buffered().use { input ->
            val buffer = ByteArray(8192)
            var bytesRead: Int
            while (input.read(buffer).also { bytesRead = it } != -1) {
                digest.update(buffer, 0, bytesRead)
            }
        }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }
}
