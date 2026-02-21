import { type ResultOf, createGraphQLClient, graphql, query } from '$lib/graphql';
import type { PageLoad } from './$types';

const ShadersQuery = graphql(`
	query AdminShadersList {
		shaders(first: 250, visibility: INCLUDE) {
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
					versionCount
					extractionSummary {
						completed
						failed
						pending
						skipped
						total
					}
				}
			}
			totalCount
		}
	}
`);

export type AdminShader = ResultOf<typeof ShadersQuery>['shaders']['edges'][number]['node'];

export const load: PageLoad = async ({ fetch, depends }) => {
	depends('glint:shaders');
	const client = createGraphQLClient(fetch);
	const result = await query(client, ShadersQuery, {}, { requestPolicy: 'cache-and-network' });

	return result.match({
		Ok: (data) => ({
			shaders: data.shaders.edges.map((e) => e.node),
			error: null as string | null
		}),
		Err: (err) => ({ shaders: [] as AdminShader[], error: err.message })
	});
};
