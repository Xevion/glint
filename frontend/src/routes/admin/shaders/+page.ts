import { createGraphQLClient, graphql, query, type ResultOf } from '$lib/graphql';
import type { PageLoad } from './$types';

const AdminShadersQuery = graphql(`
	query AdminShaders {
		adminShaders(pageSize: 250) {
			items {
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
				versionCount
				extractionSummary {
					completed
					failed
					pending
					skipped
					total
				}
			}
			total
			page
			pageSize
		}
	}
`);

export type AdminShader = ResultOf<typeof AdminShadersQuery>['adminShaders']['items'][number];

export const load: PageLoad = async ({ fetch }) => {
	const client = createGraphQLClient(fetch);
	const result = await query(client, AdminShadersQuery, {});

	return result.match({
		Ok: (data) => ({ shaders: data.adminShaders.items, error: null as string | null }),
		Err: (err) => ({ shaders: [] as AdminShader[], error: err.message })
	});
};
