import { getContext } from 'svelte';
import type { FilterDescriptor } from './filter';
import type { ListState, ViewMode } from './types';

export const DATA_VIEW_KEY = Symbol('data-view');
export const DATA_VIEW_TABLE_KEY = Symbol('data-view-table');

/**
 * Context shape shared between DataView and its children.
 * Children read items, loading, and pagination state from this.
 * Toolbar components can also access the full ListState when available.
 */
export interface DataViewContext<T> {
	readonly items: T[];
	readonly loading: boolean;
	readonly error: string | null;
	readonly hasMore: boolean;
	readonly totalCount: number | null;
	readonly viewMode: ViewMode;
	loadMore: () => Promise<void>;
	/** The full list state, if created via createListState. */
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	readonly listState?: ListState<T, Record<string, FilterDescriptor<any>>>;
}

/**
 * Read the DataView context. Must be called inside a DataView descendant.
 */
export function getDataViewContext<T>(): DataViewContext<T> {
	return getContext<DataViewContext<T>>(DATA_VIEW_KEY);
}
