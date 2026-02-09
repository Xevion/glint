package com.xevion.glint.capture

import com.luciad.imageio.webp.WebPImageWriterSpi
import com.luciad.imageio.webp.WebPWriteParam
import com.mojang.blaze3d.platform.NativeImage
import com.xevion.glint.Loggers
import com.xevion.glint.mixin.NativeImageAccessor
import org.lwjgl.system.MemoryUtil
import java.awt.image.BufferedImage
import java.awt.image.DataBufferInt
import java.nio.file.Files
import java.nio.file.Path
import javax.imageio.IIOImage
import javax.imageio.stream.MemoryCacheImageOutputStream

/**
 * Writes a NativeImage to disk as WebP using the webp-imageio library.
 *
 * Converts the NativeImage's ABGR pixel buffer to a BufferedImage, then encodes
 * via ImageIO with lossy compression at the specified quality.
 */
object WebpWriter {
    private val log = Loggers.Capture.get()

    /**
     * Encodes a NativeImage to WebP and writes it to the given path.
     *
     * @param image the NativeImage containing RGBA pixel data
     * @param file the output file path
     * @param quality compression quality (0-100, higher = better). 93 gives excellent quality for screenshots.
     */
    fun write(
        image: NativeImage,
        file: Path,
        quality: Int = 93,
    ) {
        val width = image.width
        val height = image.height

        // Read ABGR pixels from native memory and convert to ARGB for BufferedImage
        val pixelCount = width * height
        val pixelPtr = (image as NativeImageAccessor).`glint$getPixels`()
        val intBuffer = MemoryUtil.memIntBuffer(pixelPtr, pixelCount)
        val bufferedImage = BufferedImage(width, height, BufferedImage.TYPE_INT_ARGB)
        val destPixels = (bufferedImage.raster.dataBuffer as DataBufferInt).data

        for (i in 0 until pixelCount) {
            val abgr = intBuffer.get(i)
            // ABGR → ARGB: swap R and B channels
            val a = abgr and 0xFF000000.toInt()
            val b = (abgr shr 16) and 0xFF
            val g = abgr and 0x0000FF00.toInt()
            val r = (abgr and 0xFF) shl 16
            destPixels[i] = a or r or g or b
        }

        // Instantiate the WebP writer directly to bypass ImageIO SPI discovery,
        // which fails in Minecraft's modded classloader environment.
        val writer = WebPImageWriterSpi().createWriterInstance()
        try {
            val writeParam = WebPWriteParam(writer.locale)
            writeParam.compressionMode = WebPWriteParam.MODE_EXPLICIT
            writeParam.compressionType = writeParam.compressionTypes[WebPWriteParam.LOSSY_COMPRESSION]
            writeParam.compressionQuality = quality / 100f

            Files.newOutputStream(file).use { outputStream ->
                MemoryCacheImageOutputStream(outputStream).use { imageOutput ->
                    writer.output = imageOutput
                    writer.write(null, IIOImage(bufferedImage, null, null), writeParam)
                }
            }
        } finally {
            writer.dispose()
        }

        log.info("WebP screenshot saved") {
            "file" to file.fileName.toString()
            "size_kb" to (Files.size(file) / 1024)
        }
    }
}
