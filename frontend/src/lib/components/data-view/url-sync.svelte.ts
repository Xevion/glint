import { replaceState } from '$app/navigation';
import { page } from '$app/state';
import { SvelteURL } from 'svelte/reactivity';
import { untrack } from 'svelte';
import {
	type FilterDescriptor,
	isFilterDefault,
	parseFilterValue,
	serializeFilterValue
} from './filter';
import { VIEW_MODE_META, type ViewMode } from './types';

/** Infer the reactive filter state shape from a filter config object. */
export type FilterState<F extends Record<string, FilterDescriptor<unknown>>> = {
	[K in keyof F]: F[K] extends FilterDescriptor<infer T> ? T | null : never;
};

/** Getter/setter pair for bridging $state across module boundaries. */
interface Accessor<T> {
	get: () => T;
	set: (v: T) => void;
}

export interface UrlSyncOptions<F extends Record<string, FilterDescriptor<unknown>>> {
	sort: Accessor<string>;
	search: Accessor<string>;
	filters: Accessor<FilterState<F>>;
	viewMode: Accessor<ViewMode>;
	defaults: { viewMode: ViewMode; sort: string };
	filterConfig: F;
	filterDefaults: FilterState<F>;
	searchDebounce: number;
}

const VALID_VIEW_MODES: ReadonlySet<string> = new Set(Object.keys(VIEW_MODE_META));

/**
 * Bidirectional URL ↔ state sync.
 *
 * On mount (and whenever `page.url` changes from navigation), URL params are
 * parsed into component state. When the user changes state interactively,
 * state is written back to the URL via `replaceState`.
 *
 * Must be called during component initialization (inside a reactive root).
 */
export function setupUrlSync<F extends Record<string, FilterDescriptor<unknown>>>(
	opts: UrlSyncOptions<F>
): void {
	function applyUrlToState(sp: URLSearchParams) {
		opts.search.set(sp.get('q') ?? '');
		opts.sort.set(sp.get('sort') ?? opts.defaults.sort);
		const viewParam = sp.get('view');
		opts.viewMode.set(
			viewParam && VALID_VIEW_MODES.has(viewParam) ? (viewParam as ViewMode) : opts.defaults.viewMode
		);
		const filters = { ...opts.filterDefaults };
		for (const key of Object.keys(opts.filterConfig)) {
			const urlVal = sp.get(key);
			if (urlVal !== null) {
				const descriptor = opts.filterConfig[key as keyof F];
				(filters as Record<string, unknown>)[key] = parseFilterValue(urlVal, descriptor);
			}
		}
		opts.filters.set(filters);
	}

	// ── State → URL ──

	function writeStateToUrl() {
		const sort = opts.sort.get();
		const search = opts.search.get();
		const filters = opts.filters.get();
		const view = opts.viewMode.get();

		const url = new SvelteURL(page.url.pathname, page.url.origin);
		if (sort && sort !== opts.defaults.sort) url.searchParams.set('sort', sort);
		if (search) url.searchParams.set('q', search);
		if (view !== opts.defaults.viewMode) url.searchParams.set('view', view);
		for (const key of Object.keys(opts.filterConfig)) {
			const val = (filters as Record<string, unknown>)[key];
			const def = (opts.filterDefaults as Record<string, unknown>)[key];
			if (!isFilterDefault(val, def)) {
				const serialized = serializeFilterValue(val);
				if (serialized) url.searchParams.set(key, serialized);
			}
		}
		replaceState(`${url.pathname}${url.search}`, {});
	}

	// Synchronous URL → state on mount (avoids flash on first render).
	applyUrlToState(page.url.searchParams);

	// Reactive URL → state: page.url only changes on real SvelteKit navigation
	// (popstate, goto, link click), NOT on replaceState, so this cannot loop
	// with the state→URL effect below.
	$effect(() => {
		const sp = page.url.searchParams;
		untrack(() => applyUrlToState(sp));
	});

	// Reactive state → URL: skip the first run (triggered by URL→state above),
	// then write on every subsequent state change.
	let firstRun = true;
	let prevSearch = opts.search.get();
	let prevSort = opts.sort.get();
	let prevFilterSnapshot = JSON.stringify(opts.filters.get());
	let prevView = opts.viewMode.get();
	let searchDebounceTimer: ReturnType<typeof setTimeout> | undefined;

	$effect(() => {
		const sort = opts.sort.get();
		const search = opts.search.get();
		const filters = opts.filters.get();
		const view = opts.viewMode.get();
		const filterSnapshot = JSON.stringify(filters);

		if (firstRun) {
			firstRun = false;
			prevSearch = search;
			prevSort = sort;
			prevFilterSnapshot = filterSnapshot;
			prevView = view;
			return;
		}

		const searchChanged = search !== prevSearch;
		const otherChanged =
			sort !== prevSort || view !== prevView || filterSnapshot !== prevFilterSnapshot;

		prevSearch = search;
		prevSort = sort;
		prevFilterSnapshot = filterSnapshot;
		prevView = view;

		if (!searchChanged && !otherChanged) return;

		if (searchChanged && !otherChanged) {
			searchDebounceTimer = setTimeout(() => writeStateToUrl(), opts.searchDebounce);
		} else {
			untrack(() => writeStateToUrl());
		}

		// Clears pending debounce timer on re-run and on destroy.
		return () => clearTimeout(searchDebounceTimer);
	});
}
