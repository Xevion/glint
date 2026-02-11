package com.xevion.glint.api

import com.xevion.glint.api.HttpClient.Method
import kotlinx.serialization.Serializable

/**
 * World listing and upload operations against the Glint backend API.
 */
object WorldClient {
    fun listWorlds(client: HttpClient): Result<List<WorldInfo>> = client.request { path = "/api/worlds" }

    fun createWorldUpload(
        client: HttpClient,
        request: CreateWorldUploadRequest,
    ): Result<CreateWorldUploadResponse> =
        client.request {
            method = Method.POST
            path = "/api/worlds"
            jsonBody(CreateWorldUploadRequest.serializer(), request)
            expectedStatus = setOf(201)
            onStatus(409) { ApiError.HttpError(409, "World slug already exists") }
        }

    fun completeWorldUpload(
        client: HttpClient,
        worldSlug: String,
        request: CompleteWorldUploadRequest,
    ): Result<WorldInfo> =
        client.request {
            method = Method.POST
            path = "/api/worlds/$worldSlug/complete"
            jsonBody(CompleteWorldUploadRequest.serializer(), request)
            expectedStatus = setOf(201)
        }

    fun createWorldVersionUpload(
        client: HttpClient,
        worldId: String,
        request: CreateWorldVersionUploadRequest,
    ): Result<CreateWorldVersionUploadResponse> =
        client.request {
            method = Method.POST
            path = "/api/worlds/$worldId/versions"
            jsonBody(CreateWorldVersionUploadRequest.serializer(), request)
            expectedStatus = setOf(201)
        }

    fun completeWorldVersionUpload(
        client: HttpClient,
        worldId: String,
        request: CompleteWorldVersionUploadRequest,
    ): Result<Unit> =
        client.requestUnit {
            method = Method.POST
            path = "/api/worlds/$worldId/versions/complete"
            jsonBody(CompleteWorldVersionUploadRequest.serializer(), request)
            expectedStatus = setOf(201)
        }
}

@Serializable
data class WorldVersionInfo(
    val id: String,
    val worldId: String,
    val fileUrl: String?,
    val fileHash: String?,
    val sizeBytes: Long?,
    val createdAt: String,
)

@Serializable
data class WorldInfo(
    val id: String,
    val slug: String,
    val name: String,
    val description: String?,
    val minecraftVersion: String,
    val createdAt: String,
    val updatedAt: String,
    val latestVersion: WorldVersionInfo?,
)

@Serializable
data class CreateWorldUploadRequest(
    val name: String,
    val slug: String,
    val description: String? = null,
    val minecraftVersion: String,
    val fileHash: String,
    val fileSizeBytes: Long,
)

@Serializable
data class CreateWorldUploadResponse(
    val uploadId: String,
    val presignedUrl: String,
    val expiresAt: String,
)

@Serializable
data class CompleteWorldUploadRequest(
    val uploadId: String,
)

@Serializable
data class CreateWorldVersionUploadRequest(
    val fileHash: String,
    val fileSizeBytes: Long,
)

@Serializable
data class CreateWorldVersionUploadResponse(
    val uploadId: String,
    val presignedUrl: String,
    val expiresAt: String,
)

@Serializable
data class CompleteWorldVersionUploadRequest(
    val uploadId: String,
)
