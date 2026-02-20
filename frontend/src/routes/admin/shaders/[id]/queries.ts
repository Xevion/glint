import { graphql, type ResultOf } from '$lib/graphql';

export const ShaderDetailQuery = graphql(`
	query ShaderDetail($id: String!) {
		shader(id: $id) {
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

export type ShaderDetailData = NonNullable<ResultOf<typeof ShaderDetailQuery>['shader']>;
export type ShaderVersionDetail = ShaderDetailData['versions'][number];
export type ShaderCapture = ShaderDetailData['captures'][number];
export type ShaderProfile = ShaderDetailData['profiles'][number];
export type ShaderMetadata = NonNullable<ShaderDetailData['metadata']>;
