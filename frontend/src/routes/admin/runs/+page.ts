import {
	type RelayConnection,
	emptyConnection,
	extractFiltersFromUrl
} from '$lib/components/data-view';
import { createGraphQLClient, query } from '$lib/graphql';
import type { PageLoad } from './$types';
import { AdminRunsQuery, type RunNode, runFilterConfig } from './queries';

export const load: PageLoad = async ({ fetch, url }) => {
	const client = createGraphQLClient(fetch);
	const filters = extractFiltersFromUrl(url.searchParams, runFilterConfig, 'filters');

	const result = await query(client, AdminRunsQuery, {
		first: 25,
		...filters
	});

	return result.match<{ runs: RelayConnection<RunNode>; error: string | null }>({
		Ok: (data) => ({
			runs: data.adminCaptureRuns as RelayConnection<RunNode>,
			error: null
		}),
		Err: (error) => ({
			runs: emptyConnection<RunNode>(),
			error: error.message
		})
	});
};
