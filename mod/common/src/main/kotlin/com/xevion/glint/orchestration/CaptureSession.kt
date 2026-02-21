package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import com.xevion.glint.api.GlintJsonFile
import com.xevion.glint.capture.CaptureSessionData
import com.xevion.glint.io.SessionDirectoryManager
import net.minecraft.client.Minecraft
import java.io.File
import java.io.IOException
import java.time.Instant

/**
 * Owns session filesystem I/O and capture metadata tracking.
 *
 * Constructed fresh per orchestration run in [LinearOrchestrator.start].
 */
class CaptureSession(
    private val runId: String?,
    private val outputDir: String?,
) {
    private val log = Loggers.Orchestration.get()

    var sessionDir: File? = null
        private set
    var sessionId: String = ""
        private set
    var startedAt: Instant? = null
        private set

    private val captureEntries = mutableListOf<CaptureSessionData>()

    val entries: List<CaptureSessionData> get() = captureEntries

    fun createSessionDirectory(): Boolean {
        val mc = Minecraft.getInstance()
        startedAt = Instant.now()

        return try {
            if (outputDir != null) {
                val dir = File(mc.gameDirectory, outputDir)
                if (!dir.exists() && !dir.mkdirs()) {
                    log.error("Failed to create output directory") { "path" to dir.absolutePath }
                    return false
                }
                sessionId = runId?.let { "run_$it" } ?: dir.name
                sessionDir = dir
            } else {
                val capturesDir = File(mc.gameDirectory, "glint/captures")
                val (dir, id) = SessionDirectoryManager.createSessionDirectory(capturesDir, startedAt!!)
                sessionId = id
                sessionDir = dir
            }
            log.info("Session directory created") { "path" to sessionDir!!.absolutePath }
            true
        } catch (e: IOException) {
            log.error(e, "Failed to create session directory")
            false
        } catch (e: SecurityException) {
            log.error(e, "Failed to create session directory")
            false
        }
    }

    fun writeManifest(partial: Boolean) {
        val currentSessionDir = sessionDir ?: return
        val manifestName = if (partial) "manifest_partial.json" else "manifest.json"
        val manifestFile = File(currentSessionDir, manifestName)

        val manifest =
            OrchestrationManifest.create(
                captureEntries,
                sessionId,
                startedAt ?: Instant.now(),
                runId = runId,
            )

        try {
            manifestFile.writeText(GlintJsonFile.encodeToString(OrchestrationManifest.serializer(), manifest))
            log.info("Manifest written") {
                "partial" to partial
                "path" to manifestFile.absolutePath
            }
        } catch (e: IOException) {
            log.error(e, "Failed to write manifest")
        } catch (e: kotlinx.serialization.SerializationException) {
            log.error(e, "Failed to serialize manifest")
        }
    }

    fun recordCapture(data: CaptureSessionData) {
        captureEntries.add(data)
    }

    fun buildCaptureFilename(
        item: com.xevion.glint.api.WorkItem,
        shader: ShaderSpec,
    ): String {
        val scenePrefix = sanitize(item.scene.slug)
        val presetSuffix = item.preset?.slug?.let { "_${sanitize(it)}" } ?: ""

        if (shader.filename == null) {
            return "${scenePrefix}${presetSuffix}_vanilla.webp"
        }

        val info = parseShaderPackName(shader.filename)
        val profileSuffix = shader.profile?.let { "_${sanitize(it)}" } ?: ""
        return "${scenePrefix}${presetSuffix}_${info.id}_${info.version}$profileSuffix.webp"
    }

    fun buildShaderMetadata(shader: ShaderSpec): com.xevion.glint.capture.ShaderMetadata? {
        if (shader.filename == null) return null
        val info = parseShaderPackName(shader.filename)
        return com.xevion.glint.capture.ShaderMetadata(
            filename = shader.filename,
            id = info.id,
            version = info.version,
            profile = shader.profile,
            profileId = shader.profileId,
        )
    }

    fun close() {
        captureEntries.clear()
        sessionDir = null
        sessionId = ""
        startedAt = null
    }
}

private fun parseShaderPackName(filename: String): ShaderPackInfo {
    val baseName = filename.removeSuffix(".zip").removeSuffix(".ZIP")
    val parts = baseName.split("-")

    return if (parts.size >= 3) {
        ShaderPackInfo(
            id = sanitize(parts.dropLast(2).joinToString("-")),
            version = sanitize(parts[parts.size - 2]),
        )
    } else if (parts.size >= 2) {
        ShaderPackInfo(
            id = sanitize(parts.dropLast(1).joinToString("-")),
            version = sanitize(parts.last()),
        )
    } else {
        ShaderPackInfo(id = sanitize(baseName), version = "unknown")
    }
}

private data class ShaderPackInfo(
    val id: String,
    val version: String,
)

private fun sanitize(input: String): String = input.lowercase().replace(Regex("[^a-z0-9._-]"), "-")
