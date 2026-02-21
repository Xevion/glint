import { type ResultOf, createGraphQLClient, graphql, query } from '$lib/graphql';
import type { PageLoad } from './$types';

const ScenesQuery = graphql(`
	query AdminScenesList {
		scenes(first: 250, visibility: INCLUDE) {
			edges {
				node {
					id
					name
					slug
					description
					dimension
					active
					imagePath
					thumbhash
					captureCount
					createdAt
				}
			}
			totalCount
		}
	}
`);

export type AdminScene = ResultOf<typeof ScenesQuery>['scenes']['edges'][number]['node'];

export const load: PageLoad = async ({ fetch }) => {
	const client = createGraphQLClient(fetch);
	const result = await query(client, ScenesQuery, {}, { requestPolicy: 'cache-and-network' });

	return result.match({
		Ok: (data) => ({
			scenes: data.scenes.edges.map((e) => e.node),
			error: null as string | null
		}),
		Err: (err) => ({ scenes: [] as AdminScene[], error: err.message })
	});
};
