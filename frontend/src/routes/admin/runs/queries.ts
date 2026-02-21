import { filter, type FilterDescriptor } from '$lib/components/data-view';
import { type ResultOf, graphql } from '$lib/graphql';

export const AdminRunsQuery = graphql(`
	query AdminCaptureRuns($first: Int!, $after: String, $sort: String, $filters: AdminRunFiltersInput) {
		adminCaptureRuns(first: $first, after: $after, sort: $sort, filters: $filters) {
			edges {
				node {
					id
					agentId
					startedAt
					completedAt
					status
					totalItems
					completedItems
					failedItems
					skippedItems
					resolutionWidth
					resolutionHeight
					imageFormat
					durationSecs
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

export type RunNode = ResultOf<typeof AdminRunsQuery>['adminCaptureRuns']['edges'][number]['node'];

export interface RunFilters extends Record<string, FilterDescriptor<unknown>> {
	statuses: FilterDescriptor<string[]>;
	startedAfter: FilterDescriptor<string>;
	startedBefore: FilterDescriptor<string>;
	minDurationSecs: FilterDescriptor<number>;
	maxDurationSecs: FilterDescriptor<number>;
}

export const runFilterConfig: RunFilters = {
	statuses: filter<string[]>({ default: [], parse: (raw) => raw.split(',') }),
	startedAfter: filter<string>(),
	startedBefore: filter<string>(),
	minDurationSecs: filter<number>({ parse: Number }),
	maxDurationSecs: filter<number>({ parse: Number })
};
