import { ShaderCardFragment, type ShaderCardShader } from '$lib/components/ShaderCard.svelte';
import type { HeroPair } from '$lib/components/hero/types';
import { createGraphQLClient, graphql, query } from '$lib/graphql';
import type { PageLoad } from './$types';

const HomePageQuery = graphql(
	`
		query HomePage($shaderCount: Int!, $featuredCount: Int!) {
			stats {
				shaderCount
				sceneCount
				captureCount
			}
			shaders(first: $shaderCount) {
				edges {
					node {
						...ShaderCardFields
					}
				}
			}
			featured(count: $featuredCount) {
				left {
					imagePath
					thumbhash
					shaderName
					shaderSlug
					shaderAuthor
					shaderVersion
					sceneName
				}
				right {
					imagePath
					thumbhash
					shaderName
					shaderSlug
					shaderAuthor
					shaderVersion
					sceneName
				}
			}
		}
	`,
	[ShaderCardFragment]
);

export const load: PageLoad = async ({ fetch, depends }) => {
	depends('glint:stats');
	depends('glint:shaders');
	const client = createGraphQLClient(fetch);

	const result = await query(
		client,
		HomePageQuery,
		{ shaderCount: 6, featuredCount: 6 },
		{ requestPolicy: 'cache-and-network' }
	);

	return result.match({
		Ok: (data) => ({
			stats: data.stats,
			shaders: data.shaders.edges.map((edge) => edge.node),
			featuredPairs: data.featured,
			errors: [] as string[]
		}),
		Err: (e) => ({
			stats: { shaderCount: 0, sceneCount: 0, captureCount: 0 },
			shaders: [] as ShaderCardShader[],
			featuredPairs: [] as HeroPair[],
			errors: [`Home: ${e.message}`]
		})
	});
};
