package com.xevion.glint.capture

import com.mojang.blaze3d.platform.NativeImage
import com.xevion.glint.Loggers
import com.xevion.glint.mixin.NativeImageAccessor
import org.lwjgl.stb.STBIWriteCallback
import org.lwjgl.stb.STBImageWrite
import java.io.IOException
import java.nio.channels.FileChannel
import java.nio.channels.WritableByteChannel
import java.nio.file.Path
import java.nio.file.StandardOpenOption

/**
 * Writes NativeImage screenshots to disk as PNG using LWJGL's STB image library.
 *
 * Uses streaming callback-based I/O to avoid allocating a full in-memory PNG buffer.
 * Accesses the raw pixel pointer from NativeImage via NativeImageAccessor mixin.
 */
object FramebufferWriter {
    private val log = Loggers.Capture.get()

    @Throws(IOException::class)
    fun write(
        image: NativeImage,
        file: Path,
    ) {
        FileChannel.open(file, StandardOpenOption.CREATE, StandardOpenOption.WRITE).use { channel ->
            WriteCallback(channel).use { callback ->
                val pixels = (image as NativeImageAccessor).`glint$getPixels`()

                STBImageWrite.nstbi_write_png_to_func(
                    callback.address(),
                    0L,
                    image.width,
                    image.height,
                    image.format().components(),
                    pixels,
                    0,
                )

                if (callback.exception != null) {
                    throw callback.exception!!
                }
            }
        }

        log.info("High-res screenshot saved") {
            "file" to file.fileName.toString()
        }
    }

    /**
     * STB write callback that streams PNG chunks directly to a file channel.
     * Captures any I/O exceptions for propagation after STB returns.
     */
    private class WriteCallback(
        private val channel: WritableByteChannel,
    ) : STBIWriteCallback() {
        var exception: IOException? = null

        override fun invoke(
            context: Long,
            data: Long,
            size: Int,
        ) {
            if (exception != null) return

            try {
                channel.write(getData(data, size))
            } catch (e: IOException) {
                exception = e
            }
        }

        override fun close() {
            free()
        }
    }
}
