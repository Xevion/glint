import { ShaderCardFragment, type ShaderCardShader } from '$lib/components/ShaderCard.svelte';
import { createGraphQLClient, graphql, query } from '$lib/graphql';
import type { PageLoad } from './$types';

export const _BrowseShadersQuery = graphql(
	`
		query BrowseShaders($first: Int!, $after: String, $search: String, $sort: String) {
			shaders(first: $first, after: $after, search: $search, sort: $sort) {
				edges {
					node {
						...ShaderCardFields
					}
				}
				pageInfo {
					hasNextPage
					endCursor
				}
				totalCount
			}
		}
	`,
	[ShaderCardFragment]
);

export const load: PageLoad = async ({ fetch, url }) => {
	const client = createGraphQLClient(fetch);

	const q = url.searchParams.get('q') ?? undefined;
	const sort = url.searchParams.get('sort') ?? undefined;

	const result = await query(client, _BrowseShadersQuery, { first: 24, search: q, sort });

	return result.match<{
		shaders: ShaderCardShader[];
		total: number;
		hasNextPage: boolean;
		endCursor: string | null;
		q: string;
		sort: string;
		error: string | null;
	}>({
		Ok: (data) => ({
			// eslint-disable-next-line @typescript-eslint/no-unsafe-return -- gql.tada fragment types unresolvable by eslint
			shaders: data.shaders.edges.map((e) => e.node as ShaderCardShader),
			total: data.shaders.totalCount,
			hasNextPage: data.shaders.pageInfo.hasNextPage,
			endCursor: data.shaders.pageInfo.endCursor ?? null,
			q: q ?? '',
			sort: sort ?? 'popular',
			error: null
		}),
		Err: (error) => ({
			shaders: [],
			total: 0,
			hasNextPage: false,
			endCursor: null,
			q: q ?? '',
			sort: sort ?? 'popular',
			error: error.message
		})
	});
};
