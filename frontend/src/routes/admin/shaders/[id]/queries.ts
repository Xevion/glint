import { type ResultOf, graphql } from '$lib/graphql';

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
			deletedAt
			versions(first: 100) {
				edges {
					node {
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
				}
			}
			authors {
				id
				shaderId
				name
				url
				platform
			}
			captures(first: 100) {
				edges {
					node {
						id
						sceneName
						sceneId
						shaderVersion
						profileDisplayName
						imagePath
						thumbhash
						freshness
					}
				}
			}
			profiles(first: 100) {
				edges {
					node {
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
				}
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
export type ShaderVersionDetail = ShaderDetailData['versions']['edges'][number]['node'];
export type ShaderCapture = ShaderDetailData['captures']['edges'][number]['node'];
export type ShaderProfile = ShaderDetailData['profiles']['edges'][number]['node'];
export type ShaderMetadata = NonNullable<ShaderDetailData['metadata']>;
