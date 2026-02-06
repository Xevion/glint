package com.xevion.glint.api

import kotlinx.serialization.KSerializer
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import java.io.File
import kotlin.test.Test
import kotlin.test.fail

/**
 * Validates that Kotlin serializable types can deserialize JSON matching the exported backend JSON
 * schemas. Catches schema drift between the Rust backend and the Kotlin mod.
 *
 * The schemas directory is populated by `cargo test export_all_schemas` in the backend, and
 * contains the exact field names the API will produce.
 */
class SchemaCompatibilityTest {
    private val json = Json { ignoreUnknownKeys = true }
    private val schemasDir = findSchemasDir()

    /** Builds a minimal JSON object from a JSON Schema definition, using stub values. */
    private fun buildStubFromSchema(schema: JsonObject): JsonObject {
        val properties = schema["properties"]?.jsonObject ?: return JsonObject(emptyMap())
        val required = schema["required"]?.jsonArray?.map { it.toString().trim('"') }?.toSet() ?: emptySet()

        val fields = mutableMapOf<String, JsonElement>()
        for ((key, prop) in properties) {
            val propObj = prop.jsonObject
            val typeField = propObj["type"]
            val isRequired = key in required

            // Determine the primary type (handles both "type": "string" and "type": ["string", "null"])
            val primaryType =
                when (typeField) {
                    is JsonPrimitive -> typeField.content
                    is JsonArray -> typeField.map { it.toString().trim('"') }.firstOrNull { it != "null" } ?: "null"
                    else -> "string"
                }

            val value: JsonElement =
                when (primaryType) {
                    "string" -> JsonPrimitive("stub")
                    "integer" -> JsonPrimitive(0)
                    "number" -> JsonPrimitive(0.0)
                    "boolean" -> JsonPrimitive(false)
                    "array" -> JsonArray(emptyList())
                    "object" -> JsonObject(emptyMap())
                    "null" -> JsonNull
                    else -> JsonPrimitive("stub")
                }

            // Include required fields always; include optional fields too (with a value)
            // so we validate the field name mapping
            if (isRequired || !isRequired) {
                fields[key] = value
            }
        }
        return JsonObject(fields)
    }

    /**
     * Asserts that a Kotlin serializable type can deserialize a JSON object built from its
     * corresponding backend JSON schema. If the field names don't match (e.g., camelCase vs
     * snake_case), deserialization will fail.
     */
    private fun <T> assertSchemaCompatible(
        schemaName: String,
        serializer: KSerializer<T>,
    ) {
        val schemaFile = schemasDir.resolve("$schemaName.json")
        if (!schemaFile.exists()) {
            fail("Schema file not found: $schemaFile — run `cargo test export_all_schemas` in the backend")
        }

        val schema = json.parseToJsonElement(schemaFile.readText()).jsonObject
        val stub = buildStubFromSchema(schema)
        val stubJson = stub.toString()

        try {
            json.decodeFromString(serializer, stubJson)
        } catch (e: Exception) {
            fail(
                "Schema drift detected for $schemaName!\n" +
                    "Kotlin type cannot deserialize JSON matching the backend schema.\n" +
                    "Schema fields: ${stub.keys.sorted()}\n" +
                    "Stub JSON: $stubJson\n" +
                    "Error: ${e.message}",
            )
        }
    }

    @Test
    fun `WorkItem schema matches backend`() {
        assertSchemaCompatible("WorkItem", WorkItem.serializer())
    }

    companion object {
        /** Finds the schemas directory relative to the mod project root. */
        private fun findSchemasDir(): File {
            // When running from Gradle, the working directory is the subproject directory
            // (mod/common). The schemas are at the repo root.
            val candidates =
                listOf(
                    File("../../schemas"), // mod/common -> repo root
                    File("../schemas"), // mod -> repo root
                    File("schemas"), // repo root
                )
            return candidates.firstOrNull { it.isDirectory }
                ?: error(
                    "Cannot find schemas directory. " +
                        "Run `cargo test export_all_schemas` in the backend first. " +
                        "Searched: ${candidates.map { it.absolutePath }}",
                )
        }
    }
}
