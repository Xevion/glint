import { createGraphQLClient, query } from '$lib/graphql';
import type { PageLoad } from './$types';
import { AdminShadersQuery, type AdminShader } from './queries';

export const load: PageLoad = async ({ fetch, depends }) => {
	depends('glint:shaders');
	const client = createGraphQLClient(fetch);

	// Extract into closure so TypeScript can infer result type without a circular reference
	// caused by the while loop reassigning `after` from `result.value`.
	const fetchPage = (cursor?: string) =>
		query(client, AdminShadersQuery, { first: 100, after: cursor });

	const allShaders: AdminShader[] = [];
	let hasNextPage = true;
	let cursor: string | undefined;

	while (hasNextPage) {
		// eslint-disable-next-line no-await-in-loop
		const result = await fetchPage(cursor);
		if (!result.isOk) {
			return { shaders: [] as AdminShader[], error: result.error.message };
		}
		const conn = result.value.shaders;
		allShaders.push(...conn.edges.map((e) => e.node));
		hasNextPage = conn.pageInfo.hasNextPage;
		cursor = conn.pageInfo.endCursor ?? undefined;
	}

	return { shaders: allShaders, error: null as string | null };
};
