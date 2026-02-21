import { emptyConnection, type RelayConnection } from '$lib/components/data-view';
import { createGraphQLClient, query, type ResultOf } from '$lib/graphql';
import type { PageLoad } from './$types';
import { BrowseAuthorsQuery } from './queries';

export type AuthorCardData = ResultOf<
	typeof BrowseAuthorsQuery
>['authors']['edges'][number]['node'];

export const load: PageLoad = async ({ fetch, url, depends }) => {
	depends('glint:authors');
	const client = createGraphQLClient(fetch);

	const q = url.searchParams.get('q') ?? undefined;
	const sort = url.searchParams.get('sort') ?? undefined;

	const result = await query(
		client,
		BrowseAuthorsQuery,
		{ first: 24, search: q, sort },
		{ requestPolicy: 'cache-and-network' }
	);

	return result.match<{
		authors: RelayConnection<AuthorCardData>;
		error: string | null;
	}>({
		Ok: (data) => ({
			authors: data.authors,
			error: null
		}),
		Err: (error) => ({
			authors: emptyConnection<AuthorCardData>(),
			error: error.message
		})
	});
};
