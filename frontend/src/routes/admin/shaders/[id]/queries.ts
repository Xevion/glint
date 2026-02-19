import { graphql, type ResultOf } from '$lib/graphql';

export const AdminShaderQuery = graphql(`
	query AdminShader($id: String!) {
		adminShader(id: $id) {
			id
			name
			slug
			description
			modrinthId
			curseforgeId
			websiteUrl
			iconUrl
			sourceUrl
			licenseId
			upstreamDownloads
			upstreamUpdatedAt
			lastSyncedAt
			createdAt
			updatedAt
			viewCount
			preferredVersionId
			captureEnabled
			versions {
				version {
					id
					shaderId
					version
					modrinthVersionId
					curseforgeFileId
					downloadUrl
					fileHash
					fileSize
					gameVersions
					releaseChannel
					upstreamPublishedAt
					createdAt
					captureFailureCount
					lastCaptureError
					extractionStatus
					extractionError
					extractedAt
				}
				captureCount
			}
			authors {
				id
				shaderId
				name
				url
				platform
			}
			captures {
				id
				sceneName
				sceneId
				shaderVersion
				profileDisplayName
				imagePath
				thumbhash
				freshness
			}
			profiles {
				id
				shaderVersionId
				name
				label
				displayName
				description
				options
				sortOrder
				createdAt
			}
			metadata {
				shaderVersionId
				hasCustomTextures
				extractedAt
				pipelineFeatures
				irisFeaturesRequired
				irisFeaturesOptional
				settingsScreen
				filePaths
				dimensionSupport
			}
		}
	}
`);

export type AdminShaderData = NonNullable<ResultOf<typeof AdminShaderQuery>['adminShader']>;
export type AdminShaderVersion = AdminShaderData['versions'][number];
export type AdminCapture = AdminShaderData['captures'][number];
export type AdminProfile = AdminShaderData['profiles'][number];
export type AdminMetadata = NonNullable<AdminShaderData['metadata']>;
