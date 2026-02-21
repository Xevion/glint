import { type ResultOf, graphql } from '$lib/graphql';

export const AdminShadersQuery = graphql(`
	query AdminShadersList($first: Int!, $after: String) {
		shaders(first: $first, after: $after, visibility: INCLUDE) {
			edges {
				node {
					id
					name
					slug
					description
					modrinthId
					curseforgeId
					websiteUrl
					iconUrl
					lastSyncedAt
					createdAt
					captureEnabled
					deletedAt
					versionCount
					extractionSummary {
						completed
						failed
						pending
						skipped
						total
					}
					captureHealth {
						total
						completed
					}
				}
			}
			pageInfo {
				hasNextPage
				endCursor
			}
			totalCount
		}
	}
`);

export type AdminShader = ResultOf<typeof AdminShadersQuery>['shaders']['edges'][number]['node'];
