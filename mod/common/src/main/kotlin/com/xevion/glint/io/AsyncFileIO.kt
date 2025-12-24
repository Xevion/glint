package com.xevion.glint.io

import com.xevion.glint.Glint
import kotlinx.serialization.KSerializer
import kotlinx.serialization.json.Json
import java.io.File
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Executors

/**
 * Provides async file I/O operations to avoid blocking the game thread.
 * All operations return CompletableFuture for async/await patterns.
 */
object AsyncFileIO {
    private val executor =
        Executors.newFixedThreadPool(2) { r ->
            Thread(r, "Glint-FileIO").apply {
                isDaemon = true
                priority = Thread.NORM_PRIORITY - 1
            }
        }

    private val json =
        Json {
            prettyPrint = true
            ignoreUnknownKeys = true
        }

    /**
     * Reads and deserializes a JSON file asynchronously.
     */
    fun <T> readJson(
        file: File,
        serializer: KSerializer<T>,
    ): CompletableFuture<T> =
        CompletableFuture.supplyAsync({
            try {
                json.decodeFromString(serializer, file.readText())
            } catch (e: Exception) {
                Glint.LOGGER.error("Failed to read JSON file: ${file.absolutePath}", e)
                throw e
            }
        }, executor)

    /**
     * Serializes and writes a JSON file asynchronously.
     */
    fun <T> writeJson(
        file: File,
        data: T,
        serializer: KSerializer<T>,
    ): CompletableFuture<Unit> =
        CompletableFuture
            .runAsync(
                {
                    try {
                        file.writeText(json.encodeToString(serializer, data))
                    } catch (e: Exception) {
                        Glint.LOGGER.error("Failed to write JSON file: ${file.absolutePath}", e)
                        throw e
                    }
                },
                executor,
            ).thenApply { }

    /**
     * Lists files in a directory asynchronously.
     */
    fun listFiles(
        dir: File,
        filter: (File) -> Boolean = { true },
    ): CompletableFuture<List<File>> =
        CompletableFuture.supplyAsync({
            dir.listFiles()?.filter(filter)?.toList() ?: emptyList()
        }, executor)

    /**
     * Shuts down the executor pool. Call on mod shutdown.
     */
    fun shutdown() {
        executor.shutdown()
    }
}
