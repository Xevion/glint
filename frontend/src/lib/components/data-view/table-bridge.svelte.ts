import { createSvelteTable } from '$lib/components/data-table/svelte-table.svelte';
import { SvelteMap } from 'svelte/reactivity';
import {
	type ColumnFiltersState,
	type PaginationState,
	type RowData,
	type SortingState,
	type TableOptions,
	type VisibilityState,
	getCoreRowModel,
	getFilteredRowModel,
	getPaginationRowModel,
	getSortedRowModel
} from '@tanstack/table-core';

/**
 * Options for creating a batteries-included data table within the data-view system.
 */
export interface DataTableOptions<TData extends RowData> {
	/** Reactive data array. Use a getter for reactivity: `get data() { return items }` */
	data: TData[];
	/** TanStack column definitions. */
	columns: TableOptions<TData>['columns'];
	/** Default page size (default: 25). Pass `false` to disable pagination entirely. */
	pageSize?: number | false;
	/** Pass `false` to disable client-side sorting. */
	sorting?: false;
	/** Pass `false` to disable client-side filtering. */
	filtering?: false;
	/** Pass `false` to disable row selection. */
	selection?: false;
	/** Pass `false` to disable column visibility toggling. */
	visibility?: false;
	/**
	 * Maps column IDs (accessorKey) to server sort field strings.
	 * When provided with `sort`, enables server-driven sorting: column header
	 * clicks write to the external sort state instead of sorting client-side.
	 *
	 * Example: `{ capturedAt: 'capturedAt', shaderName: 'name' }`
	 */
	sortMapping?: Record<string, string>;
	/**
	 * External sort state accessor (typically wired to `list.sort`).
	 * Required alongside `sortMapping` for server-driven sorting.
	 */
	sort?: { get: () => string; set: (v: string) => void };
}

/** Parse a sort string like `"field"` or `"-field"` into TanStack SortingState. */
function parseSortString(
	sort: string,
	reverseMap: Map<string, { id: string; desc: boolean }>
): SortingState {
	if (!sort) return [];
	const entry = reverseMap.get(sort);
	return entry ? [entry] : [];
}

/** Convert TanStack SortingState to a sort string like `"field"` or `"-field"`. */
function toSortString(state: SortingState, mapping: Record<string, string>): string {
	if (state.length === 0) return '';
	const { id, desc } = state[0];
	const key = mapping[id];
	if (!key) return '';
	return desc ? `-${key}` : key;
}

/**
 * Creates a batteries-included TanStack table with Svelte 5 reactivity.
 * Internal to the data-view system.
 *
 * When `sortMapping` and `sort` are both provided, sorting becomes server-driven:
 * column header clicks update the external sort string (→ URL sync → server re-fetch),
 * and the table reads sort state back from the external string for header indicators.
 * Client-side row reordering is disabled in this mode.
 */
export function createDataTable<TData extends RowData>(opts: DataTableOptions<TData>) {
	const enableSorting = opts.sorting !== false;
	const serverSort = enableSorting && opts.sortMapping != null && opts.sort != null;
	const enablePagination = opts.pageSize !== false;
	const enableFiltering = opts.filtering !== false;
	const _enableSelection = opts.selection !== false;
	const enableVisibility = opts.visibility !== false;

	// Build reverse mapping: sort string → { columnId, desc }
	const reverseSortMap: SvelteMap<string, { id: string; desc: boolean }> | undefined = serverSort
		? new SvelteMap<string, { id: string; desc: boolean }>(
				Object.entries(opts.sortMapping!).flatMap(([colId, sortKey]) => [
					[sortKey, { id: colId, desc: false }],
					[`-${sortKey}`, { id: colId, desc: true }]
				])
			)
		: undefined;

	let localSorting = $state<SortingState>([]);
	let pagination = $state<PaginationState>({
		pageIndex: 0,
		pageSize: enablePagination && typeof opts.pageSize === 'number' ? opts.pageSize : 25
	});
	let columnFilters = $state<ColumnFiltersState>([]);
	let columnVisibility = $state<VisibilityState>({});

	return createSvelteTable({
		get data() {
			return opts.data;
		},
		columns: opts.columns,
		getCoreRowModel: getCoreRowModel(),
		...(enableSorting && {
			// Only sort rows client-side when NOT using server-driven sorting.
			// In server mode we still track sorting state for header indicators.
			...(!serverSort && { getSortedRowModel: getSortedRowModel() }),
			onSortingChange: (updater) => {
				if (serverSort) {
					const current = parseSortString(opts.sort!.get(), reverseSortMap!);
					const newState = typeof updater === 'function' ? updater(current) : updater;
					opts.sort!.set(toSortString(newState, opts.sortMapping!));
				} else {
					localSorting = typeof updater === 'function' ? updater(localSorting) : updater;
				}
			}
		}),
		...(enablePagination && {
			getPaginationRowModel: getPaginationRowModel(),
			onPaginationChange: (updater) => {
				pagination = typeof updater === 'function' ? updater(pagination) : updater;
			}
		}),
		...(enableFiltering && {
			getFilteredRowModel: getFilteredRowModel(),
			onColumnFiltersChange: (updater) => {
				columnFilters = typeof updater === 'function' ? updater(columnFilters) : updater;
			}
		}),
		...(enableVisibility && {
			onColumnVisibilityChange: (updater) => {
				columnVisibility = typeof updater === 'function' ? updater(columnVisibility) : updater;
			}
		}),
		state: {
			get sorting() {
				// Server mode: derive sort state from external sort string so column
				// headers reflect the active sort (including changes from toolbar/URL).
				if (serverSort) return parseSortString(opts.sort!.get(), reverseSortMap!);
				return localSorting;
			},
			get pagination() {
				return pagination;
			},
			get columnFilters() {
				return columnFilters;
			},
			get columnVisibility() {
				return columnVisibility;
			}
		}
	});
}
