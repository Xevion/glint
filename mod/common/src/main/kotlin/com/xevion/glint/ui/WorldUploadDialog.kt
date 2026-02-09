package com.xevion.glint.ui

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
import net.minecraft.client.gui.screens.Screen
import java.io.File
import net.minecraft.network.chat.Component as McComponent

/**
 * Dialog for entering world metadata before uploading.
 * Pre-fills name/slug from the world name, allows editing.
 */
class WorldUploadDialog(
    private val parentScreen: Screen,
    private val worldDir: File,
    private val defaultName: String,
    private val onUpload: (name: String, slug: String, description: String?) -> Unit,
) : GlintDialogScreen(McComponent.literal("Upload World")) {
    private lateinit var nameInput: TextBoxComponent
    private lateinit var slugInput: TextBoxComponent
    private lateinit var descriptionInput: TextBoxComponent
    private lateinit var uploadButton: ButtonComponent
    private lateinit var errorLabel: LabelComponent

    private var slugManuallyEdited = false

    override fun buildDialog(dialog: FlowLayout) {
        dialog.child(GlintComponents.title(title) as Component)

        // World Name
        val nameContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
        nameContainer.horizontalAlignment(HorizontalAlignment.LEFT)
        nameContainer.gap(GlintTheme.GAP_SM)
        nameContainer.child(
            Components
                .label(McComponent.literal("World Name:"))
                .color(Color.ofRgb(GlintTheme.TEXT_SECONDARY)) as Component,
        )
        nameInput = Components.textBox(Sizing.fixed(200))
        nameInput.setMaxLength(128)
        nameInput.text(defaultName)
        nameInput.onChanged().subscribe {
            if (!slugManuallyEdited) {
                slugInput.text(deriveSlug(nameInput.value))
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
        slugInput.text(deriveSlug(defaultName))
        slugInput.onChanged().subscribe {
            slugManuallyEdited = slugInput.value != deriveSlug(nameInput.value)
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

        // Minecraft version (read-only info)
        val versionRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        versionRow.gap(GlintTheme.GAP_SM)
        versionRow.child(
            Components
                .label(McComponent.literal("Minecraft:"))
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
        versionRow.child(
            Components
                .label(McComponent.literal("1.21.4"))
                .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component,
        )
        dialog.child(versionRow as Component)

        // World path (read-only info)
        val pathRow = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        pathRow.gap(GlintTheme.GAP_SM)
        pathRow.child(
            Components
                .label(McComponent.literal("Source:"))
                .color(Color.ofRgb(GlintTheme.TEXT_MUTED)) as Component,
        )
        pathRow.child(
            Components
                .label(McComponent.literal(worldDir.name))
                .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)) as Component,
        )
        dialog.child(pathRow as Component)

        // Error label
        errorLabel = Components.label(McComponent.literal(""))
        errorLabel.color(Color.ofRgb(GlintTheme.TEXT_ERROR))
        errorLabel.margins(Insets.top(GlintTheme.GAP_SM))
        dialog.child(errorLabel as Component)

        // Buttons
        uploadButton = GlintComponents.button(McComponent.literal("Upload")) { doUpload() }
        uploadButton.active = true
        dialog.child(
            GlintComponents.buttonRow(
                uploadButton,
                GlintComponents.cancelButton { minecraft?.setScreen(parentScreen) },
            ) as Component,
        )

        // Initial validation
        validateInput()

        uiAdapter.rootComponent.focusHandler()?.focus(nameInput as Component, null)
    }

    private fun validateInput() {
        val name = nameInput.value.trim()
        val slug = slugInput.value.trim()

        if (name.isEmpty()) {
            errorLabel.text(McComponent.literal("Name is required"))
            uploadButton.active = false
            return
        }

        if (slug.isEmpty()) {
            errorLabel.text(McComponent.literal("Slug is required"))
            uploadButton.active = false
            return
        }

        if (!slug.matches(Regex("[a-z0-9-]+"))) {
            errorLabel.text(McComponent.literal("Slug must contain only lowercase letters, numbers, and hyphens"))
            uploadButton.active = false
            return
        }

        errorLabel.text(McComponent.literal(""))
        uploadButton.active = true
    }

    private fun doUpload() {
        val name = nameInput.value.trim()
        val slug = slugInput.value.trim()
        val description = descriptionInput.value.trim().ifEmpty { null }
        onUpload(name, slug, description)
    }

    companion object {
        fun deriveSlug(name: String): String =
            name
                .lowercase()
                .replace(' ', '-')
                .replace(Regex("[^a-z0-9-]"), "")
                .replace(Regex("-+"), "-")
                .trim('-')
    }
}
