package com.xevion.glint.ui

import com.xevion.glint.Loggers
import com.xevion.glint.scene.LocalSceneMetadata
import com.xevion.glint.scene.LocalSceneStore
import com.xevion.glint.scene.SceneExporter
import com.xevion.glint.scene.ScenePackageMeta
import com.xevion.glint.scene.SceneState
import com.xevion.glint.scene.scenePackageJson
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintDialogScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.ButtonComponent
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.component.LabelComponent
import io.wispforest.owo.ui.component.TextBoxComponent
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.Insets
import io.wispforest.owo.ui.core.Sizing
import net.minecraft.client.Minecraft
import net.minecraft.client.gui.screens.Screen
import java.nio.file.Path
import java.security.MessageDigest
import java.time.Instant
import java.util.concurrent.CompletableFuture
import java.util.zip.ZipFile
import net.minecraft.network.chat.Component as McComponent

class ExportSceneDialog(
    private val parentScreen: Screen,
    private val onExportComplete: () -> Unit,
) : GlintDialogScreen(McComponent.literal("Export Scene")) {
    companion object {
        private val log = Loggers.Ui.get()
        private val RENDER_DISTANCES = listOf(8, 12, 16, 24, 32)
        private val SLUG_PATTERN = Regex("[a-z0-9-]+")
        private val NON_SLUG_CHARS = Regex("[^a-z0-9\\s-]")
        private val WHITESPACE_RUNS = Regex("\\s+")
        private val HYPHEN_RUNS = Regex("-+")
    }

    private lateinit var nameInput: TextBoxComponent
    private lateinit var slugInput: TextBoxComponent
    private lateinit var descriptionInput: TextBoxComponent
    private lateinit var exportButton: ButtonComponent
    private lateinit var errorLabel: LabelComponent
    private var slugManuallyEdited = false
    private var renderDistanceIndex: Int = 0

    private var isExporting = false
    private lateinit var progressLabel: LabelComponent

    override fun buildDialog(dialog: FlowLayout) {
        if (isExporting) buildProgressView(dialog) else buildFormView(dialog)
    }

    @Suppress("LongMethod")
    private fun buildFormView(dialog: FlowLayout) {
        dialog.child(GlintComponents.title(title) as Component)

        val mc = Minecraft.getInstance()

        // Initialize render distance index from game settings
        val currentRd = mc.options.renderDistance().get()
        renderDistanceIndex =
            RENDER_DISTANCES.indexOfFirst { it >= currentRd }.let { if (it < 0) RENDER_DISTANCES.lastIndex else it }

        // Name
        val nameContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
        nameContainer.horizontalAlignment(HorizontalAlignment.LEFT)
        nameContainer.gap(GlintTheme.GAP_SM)
        nameContainer.child(
            Components
                .label(McComponent.literal("Name:"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        nameInput = Components.textBox(Sizing.fixed(200))
        nameInput.setMaxLength(128)
        nameInput.onChanged().subscribe {
            if (!slugManuallyEdited) {
                slugInput.text(generateSlug(nameInput.value))
            }
            validateInput()
        }
        nameContainer.child(nameInput as Component)
        dialog.child(nameContainer as Component)

        // Slug
        val slugContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
        slugContainer.horizontalAlignment(HorizontalAlignment.LEFT)
        slugContainer.gap(GlintTheme.GAP_SM)
        slugContainer.child(
            Components
                .label(McComponent.literal("Slug (URL-safe):"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        slugInput = Components.textBox(Sizing.fixed(200))
        slugInput.setMaxLength(64)
        slugInput.onChanged().subscribe {
            slugManuallyEdited = slugInput.value != generateSlug(nameInput.value)
            validateInput()
        }
        slugContainer.child(slugInput as Component)
        dialog.child(slugContainer as Component)

        // Description
        val descContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
        descContainer.horizontalAlignment(HorizontalAlignment.LEFT)
        descContainer.gap(GlintTheme.GAP_SM)
        descContainer.child(
            Components
                .label(McComponent.literal("Description (optional):"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        descriptionInput = Components.textBox(Sizing.fixed(200))
        descriptionInput.setMaxLength(256)
        descriptionInput.setSuggestion("Brief description")
        descContainer.child(descriptionInput as Component)
        dialog.child(descContainer as Component)

        // Render Distance (cycle button)
        val rdContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
        rdContainer.horizontalAlignment(HorizontalAlignment.LEFT)
        rdContainer.gap(GlintTheme.GAP_SM)
        rdContainer.child(
            Components
                .label(McComponent.literal("Render Distance:"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        val rdRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        rdRow.gap(GlintTheme.GAP_SM)

        val rd = RENDER_DISTANCES[renderDistanceIndex]
        val chunks = (rd * 2 + 1) * (rd * 2 + 1)
        val rdChunkLabel = Components.label(McComponent.literal("~$chunks chunks"))
        rdChunkLabel.color(Color.ofRgb(GlintTheme.TEXT_MUTED))

        val rdButton =
            GlintComponents.smallButton(
                McComponent.literal("$rd"),
                width = 40,
                tooltip = McComponent.literal("Cycle render distance"),
            ) {
                renderDistanceIndex = (renderDistanceIndex + 1) % RENDER_DISTANCES.size
                val newRd = RENDER_DISTANCES[renderDistanceIndex]
                val newChunks = (newRd * 2 + 1) * (newRd * 2 + 1)
                it.setMessage(McComponent.literal("$newRd"))
                rdChunkLabel.text(McComponent.literal("~$newChunks chunks"))
            }
        rdRow.child(rdButton as Component)
        rdRow.child(rdChunkLabel as Component)
        rdContainer.child(rdRow as Component)
        dialog.child(rdContainer as Component)

        // Camera (read-only)
        val player = mc.player
        val cameraRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        cameraRow.gap(GlintTheme.GAP_SM)
        cameraRow.child(
            Components
                .label(McComponent.literal("Camera:"))
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
        val cameraText =
            if (player != null) {
                "X: %.1f  Y: %.1f  Z: %.1f  Yaw: %.1f  Pitch: %.1f".format(
                    player.x,
                    player.y,
                    player.z,
                    player.yRot,
                    player.xRot,
                )
            } else {
                "unknown"
            }
        cameraRow.child(
            Components
                .label(McComponent.literal(cameraText))
                .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component,
        )
        dialog.child(cameraRow as Component)

        // Environment (read-only)
        val level = mc.level
        val envRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        envRow.gap(GlintTheme.GAP_SM)
        envRow.child(
            Components
                .label(McComponent.literal("Environment:"))
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
        val envText =
            if (level != null) {
                val time = level.dayTime % 24000
                val timeName =
                    when {
                        time < 1000L -> "Night"
                        time < 6000L -> "Morning"
                        time < 9000L -> "Noon"
                        time < 13000L -> "Afternoon"
                        time < 18000L -> "Evening"
                        else -> "Night"
                    }
                val weather =
                    when {
                        level.isThundering -> "Thunder"
                        level.levelData.isRaining -> "Rain"
                        else -> "Clear"
                    }
                val moonPhase = level.getMoonPhase()
                "Time: $time ($timeName)  Weather: $weather  Moon: $moonPhase"
            } else {
                "unknown"
            }
        envRow.child(
            Components
                .label(McComponent.literal(envText))
                .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component,
        )
        dialog.child(envRow as Component)

        // Error label
        errorLabel = Components.label(McComponent.literal(""))
        errorLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
        errorLabel.margins(Insets.top(GlintTheme.GAP_SM))
        dialog.child(errorLabel as Component)

        // Buttons
        exportButton = GlintComponents.button(McComponent.literal("Export")) { doExport() }
        exportButton.active = false
        dialog.child(
            GlintComponents.buttonRow(
                exportButton,
                GlintComponents.cancelButton { minecraft?.setScreen(parentScreen) },
            ) as Component,
        )

        validateInput()

        uiAdapter.rootComponent.focusHandler()?.focus(nameInput as Component, null)
    }

    private fun buildProgressView(dialog: FlowLayout) {
        dialog.child(GlintComponents.title(McComponent.literal("Exporting Scene...")) as Component)

        progressLabel = Components.label(McComponent.literal("Starting export..."))
        progressLabel.color(Color.ofRgb(GlintTheme.TEXT_SECONDARY))
        dialog.child(progressLabel as Component)
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
            errorLabel.text(McComponent.literal("Slug must contain only lowercase letters, numbers, and hyphens"))
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
        val renderDistance = RENDER_DISTANCES[renderDistanceIndex]

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
                            handleExportSuccess(zipPath, name, slug, description, renderDistance)
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
