import { ShaderCardFragment } from '$lib/components/ShaderCard.svelte';
import { graphql } from '$lib/graphql';

export const BrowseShadersQuery = graphql(
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
