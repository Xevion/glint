import { graphql } from '$lib/graphql';

export const BrowseAuthorsQuery = graphql(`
	query BrowseAuthors($first: Int!, $after: String, $search: String, $sort: String) {
		authors(first: $first, after: $after, search: $search, sort: $sort) {
			edges {
				node {
					name
					slug
					shaderCount
					totalViews
					imagePath
					thumbhash
					topShaderName
					topShaderSlug
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
