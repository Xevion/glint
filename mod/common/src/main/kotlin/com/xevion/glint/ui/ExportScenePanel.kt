package com.xevion.glint.ui

import com.xevion.glint.Loggers
import com.xevion.glint.scene.LocalSceneMetadata
import com.xevion.glint.scene.LocalSceneStore
import com.xevion.glint.scene.SceneExporter
import com.xevion.glint.scene.ScenePackageMeta
import com.xevion.glint.scene.SceneSetupState
import com.xevion.glint.scene.scenePackageJson
import com.xevion.glint.ui.base.GlintComponents
import com.xevion.glint.ui.base.GlintPanelScreen
import com.xevion.glint.ui.base.GlintTheme
import io.wispforest.owo.ui.component.ButtonComponent
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.component.LabelComponent
import io.wispforest.owo.ui.component.TextBoxComponent
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
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

/**
 * Lightweight export dialog for saving a scene package.
 *
 * Environment settings are managed by [SceneSetupState] — this dialog
 * only handles the name, slug, and description inputs plus the actual export.
 * [SceneSetupState] must be [active][SceneSetupState.active] before opening.
 */
class ExportScenePanel(
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
    }

    private lateinit var nameInput: TextBoxComponent
    private lateinit var slugInput: TextBoxComponent
    private lateinit var descriptionInput: TextBoxComponent
    private lateinit var exportButton: ButtonComponent
    private lateinit var errorLabel: LabelComponent

    private var slugManuallyEdited = false
    private var isExporting = false
    private lateinit var progressLabel: LabelComponent

    override fun buildPanel(panel: FlowLayout) {
        if (!SceneSetupState.active) {
            SceneSetupState.activate()
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
            },
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
            },
        )

        panel.child(
            GlintComponents.labeledField("Description (optional):") {
                descriptionInput = Components.textBox(Sizing.fixed(INPUT_WIDTH))
                descriptionInput.setMaxLength(256)
                descriptionInput.setSuggestion("Brief description")
                child(descriptionInput as Component)
            },
        )

        errorLabel = Components.label(McComponent.literal(""))
        errorLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
        errorLabel.margins(Insets.top(GlintTheme.GAP_SM))
        panel.child(errorLabel)

        exportButton = GlintComponents.button(McComponent.literal("Export")) { doExport() }
        exportButton.active = false
        panel.child(
            GlintComponents.buttonRow(
                exportButton,
                GlintComponents.cancelButton { minecraft?.setScreen(parentScreen) },
            ),
        )

        validateInput()
        uiAdapter.rootComponent.focusHandler()?.focus(nameInput as Component, null)
    }

    private fun buildProgressView(panel: FlowLayout) {
        panel.child(GlintComponents.title(McComponent.literal("Exporting Scene...")))

        progressLabel = Components.label(McComponent.literal("Starting export..."))
        progressLabel.color(Color.ofRgb(GlintTheme.TEXT_SECONDARY))
        panel.child(progressLabel)
    }

    override fun onClose() {
        minecraft?.setScreen(parentScreen)
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
                            handleExportSuccess(zipPath, name, slug, description, SceneSetupState.renderDistance)
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

        val parent = progressLabel.parent() as? FlowLayout ?: return
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

        val parent = progressLabel.parent() as? FlowLayout ?: return
        parent.child(
            GlintComponents.buttonRow(
                GlintComponents.button(McComponent.literal("Retry")) {
                    isExporting = false
                    uiAdapter.rootComponent.clearChildren()
                    build(uiAdapter.rootComponent)
                },
                GlintComponents.cancelButton { minecraft?.setScreen(parentScreen) },
            ),
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
        val hex = digest.digest().joinToString("") { "%02x".format(it) }
        return "sha256:$hex"
    }
}
