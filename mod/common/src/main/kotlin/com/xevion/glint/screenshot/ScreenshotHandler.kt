package com.xevion.glint.screenshot

import com.mojang.blaze3d.pipeline.RenderTarget
import com.mojang.blaze3d.platform.NativeImage
import com.xevion.glint.Glint
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import net.minecraft.SharedConstants
import net.minecraft.Util
import net.minecraft.client.Minecraft
import java.io.File

/**
 * Handles screenshot capture events and writes accompanying metadata files.
 */
object ScreenshotHandler {
    private val json = Json { prettyPrint = true }

    /**
     * Called by the mixin when a screenshot is captured.
     * Finds the most recently created screenshot file and collects metadata.
     */
    fun onScreenshotCaptured(renderTarget: RenderTarget) {
        val mc = Minecraft.getInstance()
        val screenshotsDir = File(mc.gameDirectory, "screenshots")

        val screenshotFile =
            screenshotsDir
                .listFiles()
                ?.filter { it.extension == "png" }
                ?.maxByOrNull { it.lastModified() }

        if (screenshotFile == null) {
            Glint.LOGGER.warn("Screenshot file not found for metadata collection")
            return
        }

        val metadata = collectMetadata(renderTarget, screenshotFile)

        Util.ioPool().execute {
            writeMetadata(metadata, screenshotFile)
        }
    }

    private fun collectMetadata(
        renderTarget: RenderTarget,
        file: File,
    ): ScreenshotMetadata {
        val mc = Minecraft.getInstance()
        val player = mc.player
        val level = mc.level

        if (player == null) {
            Glint.LOGGER.warn("Player is null during screenshot capture")
        }
        if (level == null) {
            Glint.LOGGER.warn("Level is null during screenshot capture")
        }

        return ScreenshotMetadata(
            timestamp =
                java.time.Instant
                    .now()
                    .toString(),
            screenshot =
                ScreenshotInfo(
                    file = file.name,
                    width = renderTarget.width,
                    height = renderTarget.height,
                ),
            minecraft =
                MinecraftInfo(
                    version = SharedConstants.getCurrentVersion().name,
                    dimension = level?.dimension()?.location()?.toString(),
                    position = player?.let { Position(it.x, it.y, it.z) },
                    camera = player?.let { Camera(it.yRot, it.xRot) },
                ),
            shader =
                com.xevion.glint.capture.IrisIntegration
                    .getShaderInfo(),
        )
    }

    private fun writeMetadata(
        metadata: ScreenshotMetadata,
        screenshotFile: File,
    ) {
        try {
            val jsonFile =
                File(
                    screenshotFile.parentFile,
                    screenshotFile.nameWithoutExtension + ".json",
                )

            jsonFile.writeText(json.encodeToString(metadata))

            Glint.LOGGER.debug("Saved screenshot metadata: ${jsonFile.name}")
        } catch (e: Exception) {
            Glint.LOGGER.error("Failed to write screenshot metadata", e)
        }
    }
}
