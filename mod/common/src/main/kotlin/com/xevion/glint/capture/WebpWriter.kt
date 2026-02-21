package com.xevion.glint.capture

import com.luciad.imageio.webp.WebPImageWriterSpi
import com.luciad.imageio.webp.WebPWriteParam
import com.xevion.glint.Loggers
import java.awt.image.BufferedImage
import java.awt.image.ColorModel
import java.awt.image.DataBufferInt
import java.awt.image.Raster
import java.nio.file.Files
import java.nio.file.Path
import javax.imageio.IIOImage
import javax.imageio.stream.MemoryCacheImageOutputStream

/**
 * Writes ARGB pixel data to disk as WebP using the webp-imageio library.
 *
 * Accepts a pre-converted ARGB pixel array shared with the analysis pipeline. The array is
 * wrapped directly in a BufferedImage (zero-copy) to avoid a second 32MB heap allocation
 * at 4K resolution.
 */
object WebpWriter {
    private val log = Loggers.Capture.get()

    /**
     * Encodes ARGB pixel data to WebP and writes it to the given path.
     *
     * @param argbPixels pre-converted ARGB pixel data (row-major, width × height)
     * @param width image width in pixels
     * @param height image height in pixels
     * @param file the output file path
     * @param quality compression quality (0-100, higher = better). 93 gives excellent quality.
     */
    fun write(
        argbPixels: IntArray,
        width: Int,
        height: Int,
        file: Path,
        quality: Int = 93,
    ) {
        // Wrap the existing pixel array directly in a BufferedImage — no copy, no new heap allocation.
        // DataBufferInt + createPackedRaster uses argbPixels as the raster's backing store.
        val dataBuffer = DataBufferInt(argbPixels, argbPixels.size)
        val raster =
            Raster.createPackedRaster(
                dataBuffer,
                width,
                height,
                width,
                intArrayOf(0x00FF0000, 0x0000FF00, 0x000000FF, 0xFF000000.toInt()),
                null,
            )
        val bufferedImage = BufferedImage(ColorModel.getRGBdefault(), raster, false, null)

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
