import type { Component } from 'svelte';
import { LayoutGrid, List, Table2 } from '@lucide/svelte';
import type { FilterDescriptor } from './filter';
import type { FilterState } from './url-sync.svelte';

export type ViewMode = 'grid' | 'row' | 'table' | 'tile';

export const VIEW_MODE_META: Record<
	ViewMode,
	{ icon: Component<{ class?: string; strokeWidth?: number }>; label: string }
> = {
	grid: { icon: LayoutGrid, label: 'Grid view' },
	row: { icon: List, label: 'List view' },
	table: { icon: Table2, label: 'Table view' },
	tile: { icon: LayoutGrid, label: 'Tile view' }
};

export interface SortOption {
	value: string;
	label: string;
}

export interface ListState<T, F extends Record<string, FilterDescriptor<unknown>>> {
	readonly items: T[];
	readonly loading: boolean;
	readonly error: string | null;
	readonly hasMore: boolean;
	readonly totalCount: number | null;
	loadMore: () => Promise<void>;
	reset: () => void;
	search: string;
	sort: string;
	readonly sortOptions: SortOption[];
	readonly filters: FilterState<F>;
	viewMode: ViewMode;
}
