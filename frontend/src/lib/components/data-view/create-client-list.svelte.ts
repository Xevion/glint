import type { FilterDescriptor } from './filter';
import type { ListState, SortOption, ViewMode } from './types';
import { type FilterState, setupUrlSync } from './url-sync.svelte';

interface SearchConfig<T> {
	/** Client-side filter function. */
	clientFilter?: (item: T, query: string) => boolean;
	/** Debounce delay in ms for URL sync (default: 300). */
	debounce?: number;
}

interface SortConfig<T> {
	options: SortOption[];
	default: string;
	/** Maps sort value to comparator. Omit to handle sorting externally. */
	compare?: (sortValue: string) => ((a: T, b: T) => number) | undefined;
}

export interface ClientListConfig<T, F extends Record<string, FilterDescriptor<unknown>>> {
	/** Unique identity extractor for DOM keying. */
	key: (item: T) => string | number;
	/** Reactive getter for the full item array. */
	items: () => T[];
	/** Search configuration. */
	search?: SearchConfig<T>;
	/** Sort configuration. */
	sort?: SortConfig<T>;
	/** Filter descriptors. */
	filters?: F;
	/** Predicate applied after search filter. Return true to keep. */
	applyFilters?: (item: T, filters: FilterState<F>) => boolean;
	/** Whether to sync state to URL search params. */
	syncUrl?: boolean;
	/** Initial view mode. */
	viewMode?: ViewMode;
}

/**
 * Creates a reactive list state for client-side data.
 *
 * All items are provided upfront via the `items` getter. Search filtering
 * happens client-side. No pagination or GraphQL involved.
 */
export function createClientList<
	T,
	F extends Record<string, FilterDescriptor<unknown>> = Record<string, never>
>(config: ClientListConfig<T, F>): ListState<T, F> {
	const searchDebounce = config.search?.debounce ?? 300;
	const sortDefault = config.sort?.default ?? '';
	const sortOptions = config.sort?.options ?? [];

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

	const clientFilter = config.search?.clientFilter;
	const filteredItems = $derived.by(() => {
		let result = config.items();
		if (clientFilter && searchValue) {
			const q = searchValue.toLowerCase();
			result = result.filter((item) => clientFilter(item, q));
		}
		if (config.applyFilters) {
			result = result.filter((item) => config.applyFilters!(item, filterValues));
		}
		if (config.sort?.compare) {
			const cmp = config.sort.compare(sortValue);
			if (cmp) result = [...result].sort(cmp);
		}
		return result;
	});

	function reset() {
		searchValue = '';
		sortValue = sortDefault;
		filterValues = { ...filterDefaults };
	}

	return {
		get key() {
			return config.key;
		},
		get items() {
			return filteredItems;
		},
		get loading() {
			return false;
		},
		get error() {
			return null;
		},
		get hasMore() {
			return false;
		},
		get totalCount() {
			return filteredItems.length;
		},
		loadMore: () => Promise.resolve(),
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
