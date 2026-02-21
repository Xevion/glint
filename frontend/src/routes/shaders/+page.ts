import type { ShaderCardShader } from '$lib/components/ShaderCard.svelte';
import { type RelayConnection, emptyConnection } from '$lib/components/data-view';
import { createGraphQLClient, query } from '$lib/graphql';
import type { PageLoad } from './$types';
import { BrowseShadersQuery } from './queries';

export const load: PageLoad = async ({ fetch, url, depends }) => {
	depends('glint:shaders');
	const client = createGraphQLClient(fetch);

	const q = url.searchParams.get('q') ?? undefined;
	const sort = url.searchParams.get('sort') ?? undefined;

	const result = await query(
		client,
		BrowseShadersQuery,
		{ first: 24, search: q, sort },
		{ requestPolicy: 'cache-and-network' }
	);

	return result.match<{
		shaders: RelayConnection<ShaderCardShader>;
		error: string | null;
	}>({
		Ok: (data) => ({
			shaders: data.shaders,
			error: null
		}),
		Err: (error) => ({
			shaders: emptyConnection<ShaderCardShader>(),
			error: error.message
		})
	});
};
