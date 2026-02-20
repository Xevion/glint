import { SceneCardFragment } from '$lib/components/SceneCard.svelte';
import { createGraphQLClient, graphql, query } from '$lib/graphql';
import type { ResultOf } from '$lib/graphql/tada';
import type { PageLoad } from './$types';

const ScenesQuery = graphql(
	`
		query ScenesBrowse($first: Int!, $after: String) {
			scenes(first: $first, after: $after) {
				edges {
					node {
						...SceneCardFields
					}
				}
			}
		}
	`,
	[SceneCardFragment]
);

export type BrowseScene = NonNullable<
	ResultOf<typeof ScenesQuery>['scenes']['edges'][number]['node']
>;

export const load: PageLoad = async ({ fetch, depends }) => {
	depends('glint:scenes');
	const client = createGraphQLClient(fetch);
	const result = await query(client, ScenesQuery, { first: 100 });

	return result.match<{ scenes: BrowseScene[]; error?: string }>({
		Ok: (data) => ({
			scenes: data.scenes.edges.map((edge) => edge.node)
		}),
		Err: (error) => ({
			scenes: [],
			error: error.message
		})
	});
};
