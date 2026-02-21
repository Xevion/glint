import { createGraphQLClient, query } from '$lib/graphql';
import type { TypedDocumentNode } from '@graphql-typed-document-node/core';
import { untrack } from 'svelte';
import { type RelayConnection, appendDeduplicatedItems, unwrapConnection } from './connection';
import { type FilterDescriptor, isFilterDefault } from './filter';
import type { ListState, SortOption, ViewMode } from './types';
import { type FilterState, setupUrlSync } from './url-sync.svelte';

interface SearchConfig {
	/** Debounce delay in ms for URL sync (default: 300). */
	debounce?: number;
}

interface SortConfig {
	options: SortOption[];
	default: string;
}

export interface CursorListConfig<T, F extends Record<string, FilterDescriptor<unknown>>> {
	/** Unique identity extractor for deduplication and DOM keying. */
	key: (item: T) => string | number;
	/** Reactive getter for SSR data (first page, already unwrapped from Result). */
	initial: () => RelayConnection<T>;
	/** gql-tada typed document for fetching subsequent pages. */
	// eslint-disable-next-line @typescript-eslint/no-explicit-any -- variables are built dynamically at runtime
	query: TypedDocumentNode<any, any>;
	/** Extract the connection object from a raw GraphQL query response. */
	// eslint-disable-next-line @typescript-eslint/no-explicit-any -- data type is inferred from query at call site
	extract: (data: any) => RelayConnection<T>;
	/** Number of items per page (default: 24). */
	pageSize?: number;
	/** Pre-configured GraphQL client. If omitted, one is created with global `fetch`. */
	client?: ReturnType<typeof createGraphQLClient>;
	/** Search configuration. */
	search?: SearchConfig;
	/** Sort configuration. */
	sort?: SortConfig;
	/** Filter descriptors. */
	filters?: F;
	/** Whether to sync state to URL search params. */
	syncUrl?: boolean;
	/** Initial view mode. */
	viewMode?: ViewMode;
	/** When set, filter values are nested under this GraphQL variable name. */
	filterVariable?: string;
	/**
	 * Filter values always included in every query, not user-modifiable.
	 * Merged into the filter object after user-controlled filters.
	 * Useful for detail pages that force a parent entity (e.g., shaderSlug on a shader page).
	 */
	fixedFilters?: Record<string, unknown>;
}

/**
 * Creates a reactive list state with GraphQL cursor pagination.
 *
 * Manages accumulated items, automatic re-fetching when filters/search/sort
 * change, and load-more pagination.
 */
export function createCursorList<
	T,
	F extends Record<string, FilterDescriptor<unknown>> = Record<string, never>
>(config: CursorListConfig<T, F>): ListState<T, F> {
	const searchDebounce = config.search?.debounce ?? 300;
	const sortDefault = config.sort?.default ?? '';
	const sortOptions = config.sort?.options ?? [];
	const pageSize = config.pageSize ?? 24;

	const filterConfig = (config.filters ?? {}) as F;
	const filterDefaults = {} as FilterState<F>;
	for (const key of Object.keys(filterConfig)) {
		const k = key as keyof F;
		(filterDefaults as Record<string, unknown>)[key] = filterConfig[k].default;
	}

	let searchValue = $state('');
	let sortValue = $state(sortDefault);
	let filterValues = $state<FilterState<F>>({ ...filterDefaults });
	let viewModeValue = $state<ViewMode>(config.viewMode ?? 'grid');
	let loading = $state(false);
	let error = $state<string | null>(null);

	// Initialize from SSR data synchronously so the first render has items
	const initialConnection = config.initial();
	const initialData = unwrapConnection(initialConnection);
	let accumulatedItems = $state<T[]>(initialData.items);
	let endCursor = $state<string | null>(initialData.endCursor);
	let hasNextPage = $state(initialData.hasNextPage);
	let totalCount = $state<number | null>(initialData.totalCount);

	const defaultViewMode = config.viewMode ?? 'grid';

	// Bidirectional URL ↔ state sync (reads URL on mount + popstate, writes on changes)
	if (config.syncUrl) {
		setupUrlSync({
			sort: {
				get: () => sortValue,
				set: (v) => {
					sortValue = v;
				}
			},
			search: {
				get: () => searchValue,
				set: (v) => {
					searchValue = v;
				}
			},
			filters: {
				get: () => filterValues,
				set: (v) => {
					filterValues = v;
				}
			},
			viewMode: {
				get: () => viewModeValue,
				set: (v) => {
					viewModeValue = v;
				}
			},
			defaults: { viewMode: defaultViewMode, sort: sortDefault },
			filterConfig,
			filterDefaults,
			searchDebounce
		});
	}

	const gqlClient = config.client ?? createGraphQLClient(fetch);

	/** Build GraphQL variables from current filter/search/sort state. */
	function buildQueryVariables(cursor?: string | null): Record<string, unknown> {
		const variables: Record<string, unknown> = {
			first: pageSize,
			search: searchValue || undefined,
			sort: sortValue || undefined
		};
		if (cursor) variables.after = cursor;

		const filterVars: Record<string, unknown> = {};
		for (const key of Object.keys(filterConfig)) {
			const val = (filterValues as Record<string, unknown>)[key];
			const def = (filterDefaults as Record<string, unknown>)[key];
			if (!isFilterDefault(val, def)) {
				filterVars[key] = val;
			}
		}
		// Merge fixed filters (always included, not user-modifiable)
		if (config.fixedFilters) {
			Object.assign(filterVars, config.fixedFilters);
		}
		if (config.filterVariable) {
			if (Object.keys(filterVars).length > 0) {
				variables[config.filterVariable] = filterVars;
			}
		} else {
			Object.assign(variables, filterVars);
		}
		return variables;
	}

	// Generation counter for discarding stale fetch results
	let fetchGeneration = 0;

	// React to SSR data changes (actual navigation, not filter-driven URL changes)
	$effect(() => {
		const connection = config.initial();
		const data = unwrapConnection(connection);
		accumulatedItems = data.items;
		endCursor = data.endCursor;
		hasNextPage = data.hasNextPage;
		totalCount = data.totalCount;
		error = null;
	});

	// Re-fetch when filters/search/sort change (skip first run — SSR data is fresh)
	let refetchFirstRun = true;
	$effect(() => {
		// Read reactive dependencies
		const _filterSnapshot = JSON.stringify(filterValues);
		void searchValue;
		void sortValue;

		if (refetchFirstRun) {
			refetchFirstRun = false;
			return;
		}

		const generation = ++fetchGeneration;
		loading = true;

		const variables = untrack(() => buildQueryVariables());

		// eslint-disable-next-line @typescript-eslint/no-unsafe-argument -- query typed as any for dynamic variables
		void query(gqlClient, config.query, variables).then((result) => {
			if (generation !== fetchGeneration) return;

			result.match({
				Ok: (data) => {
					const connection = config.extract(data);
					const unwrapped = unwrapConnection(connection);
					accumulatedItems = unwrapped.items;
					endCursor = unwrapped.endCursor;
					hasNextPage = unwrapped.hasNextPage;
					totalCount = unwrapped.totalCount;
					error = null;
				},
				Err: (err) => {
					error = err.message;
				}
			});
			loading = false;
		});
	});

	async function loadMore() {
		if (loading || !hasNextPage || !endCursor) return;

		loading = true;
		const variables = buildQueryVariables(endCursor);

		// eslint-disable-next-line @typescript-eslint/no-unsafe-argument -- query typed as any for dynamic variables
		const result = await query(gqlClient, config.query, variables);
		result.match({
			Ok: (data) => {
				const connection = config.extract(data);
				const unwrapped = unwrapConnection(connection);
				accumulatedItems = appendDeduplicatedItems(accumulatedItems, unwrapped.items, config.key);
				endCursor = unwrapped.endCursor;
				hasNextPage = unwrapped.hasNextPage;
				if (unwrapped.totalCount != null) {
					totalCount = unwrapped.totalCount;
				}
			},
			Err: (err) => {
				error = err.message;
			}
		});
		loading = false;
	}

	function reset() {
		accumulatedItems = [];
		endCursor = null;
		hasNextPage = false;
		totalCount = null;
		error = null;
		searchValue = '';
		sortValue = sortDefault;
		filterValues = { ...filterDefaults };
	}

	return {
		get key() {
			return config.key;
		},
		get items() {
			return accumulatedItems;
		},
		get loading() {
			return loading;
		},
		get error() {
			return error;
		},
		get hasMore() {
			return hasNextPage;
		},
		get totalCount() {
			return totalCount;
		},
		loadMore,
		reset,
		get search() {
			return searchValue;
		},
		set search(value: string) {
			searchValue = value;
		},
		get sort() {
			return sortValue;
		},
		set sort(value: string) {
			sortValue = value;
		},
		get sortOptions() {
			return sortOptions;
		},
		get filters() {
			return filterValues;
		},
		get viewMode() {
			return viewModeValue;
		},
		set viewMode(value: ViewMode) {
			viewModeValue = value;
		}
	};
}
