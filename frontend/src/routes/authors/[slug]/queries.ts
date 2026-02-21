import { ShaderCardFragment } from '$lib/components/ShaderCard.svelte';
import { graphql } from '$lib/graphql';

export const AuthorDetailQuery = graphql(
	`
		query AuthorDetail($slug: String!) {
			author(slug: $slug) {
				name
				slug
				shaderCount
				totalViews
				totalCaptures
				imagePath
				thumbhash
				topShaderName
				topShaderSlug
				platforms {
					platform
					url
				}
				shaders(first: 24) {
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
		}
	`,
	[ShaderCardFragment]
);

export const AuthorShadersQuery = graphql(
	`
		query AuthorShaders($slug: String!, $first: Int!, $after: String) {
			author(slug: $slug) {
				shaders(first: $first, after: $after) {
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
		}
	`,
	[ShaderCardFragment]
);
