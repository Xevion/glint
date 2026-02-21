import { ApiErrorType, pageError } from '$lib/api/errors';
import type { ShaderCardShader } from '$lib/components/ShaderCard.svelte';
import type { RelayConnection } from '$lib/components/data-view';
import { createGraphQLClient, query, type ResultOf } from '$lib/graphql';
import type { PageLoad } from './$types';
import { AuthorDetailQuery } from './queries';

type GqlAuthor = NonNullable<ResultOf<typeof AuthorDetailQuery>['author']>;
type GqlPlatform = GqlAuthor['platforms'][number];

export interface AuthorDetail {
	name: string;
	slug: string;
	shaderCount: number;
	totalViews: number;
	totalCaptures: number;
	imagePath?: string | null;
	thumbhash?: string | null;
	topShaderName?: string | null;
	topShaderSlug?: string | null;
	platforms: GqlPlatform[];
}

export const load: PageLoad = async ({ params, fetch, depends }) => {
	depends(`glint:author:${params.slug}`);
	const client = createGraphQLClient(fetch);

	const result = await query(
		client,
		AuthorDetailQuery,
		{ slug: params.slug },
		{ requestPolicy: 'cache-and-network' }
	);

	return result.match({
		Ok: (data) => {
			if (!data.author) {
				pageError(404, `Author "${params.slug}" not found`);
			}
			const author: AuthorDetail = {
				name: data.author.name,
				slug: data.author.slug,
				shaderCount: data.author.shaderCount,
				totalViews: data.author.totalViews,
				totalCaptures: data.author.totalCaptures,
				imagePath: data.author.imagePath,
				thumbhash: data.author.thumbhash,
				topShaderName: data.author.topShaderName,
				topShaderSlug: data.author.topShaderSlug,
				platforms: data.author.platforms
			};
			return {
				author,
				shaders: data.author.shaders as RelayConnection<ShaderCardShader>
			};
		},
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) {
				pageError(404, `Author "${params.slug}" not found`);
			}
			return err.throw();
		}
	});
};
