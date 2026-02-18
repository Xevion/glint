import {
	type ColumnFiltersState,
	type PaginationState,
	type RowData,
	type RowSelectionState,
	type SortingState,
	type TableOptions,
	type VisibilityState,
	getCoreRowModel,
	getFilteredRowModel,
	getPaginationRowModel,
	getSortedRowModel
} from '@tanstack/table-core';
import { createSvelteTable } from './svelte-table.svelte.js';

/**
 * Options for creating a batteries-included data table.
 * All features are ON by default — pass `false` to disable.
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
}

/**
 * Creates a batteries-included TanStack table with Svelte 5 reactivity.
 *
 * All features (sorting, pagination, filtering, selection, visibility) are
 * enabled by default. Pass `false` to any feature to disable it.
 *
 * @example
 * ```svelte
 * <script lang="ts">
 *   const table = createDataTable({
 *     get data() { return shaders },
 *     columns,
 *     pageSize: 50,
 *   });
 * </script>
 * ```
 */
export function createDataTable<TData extends RowData>(opts: DataTableOptions<TData>) {
	const enableSorting = opts.sorting !== false;
	const enablePagination = opts.pageSize !== false;
	const enableFiltering = opts.filtering !== false;
	const enableSelection = opts.selection !== false;
	const enableVisibility = opts.visibility !== false;

	let sorting = $state<SortingState>([]);
	let pagination = $state<PaginationState>({
		pageIndex: 0,
		pageSize: enablePagination && typeof opts.pageSize === 'number' ? opts.pageSize : 25
	});
	let columnFilters = $state<ColumnFiltersState>([]);
	let rowSelection = $state<RowSelectionState>({});
	let columnVisibility = $state<VisibilityState>({});

	return createSvelteTable({
		get data() {
			return opts.data;
		},
		columns: opts.columns,
		getCoreRowModel: getCoreRowModel(),
		...(enableSorting && {
			getSortedRowModel: getSortedRowModel(),
			onSortingChange: (updater) => {
				sorting = typeof updater === 'function' ? updater(sorting) : updater;
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
		...(enableSelection && {
			onRowSelectionChange: (updater) => {
				rowSelection = typeof updater === 'function' ? updater(rowSelection) : updater;
			}
		}),
		...(enableVisibility && {
			onColumnVisibilityChange: (updater) => {
				columnVisibility = typeof updater === 'function' ? updater(columnVisibility) : updater;
			}
		}),
		state: {
			get sorting() {
				return sorting;
			},
			get pagination() {
				return pagination;
			},
			get columnFilters() {
				return columnFilters;
			},
			get rowSelection() {
				return rowSelection;
			},
			get columnVisibility() {
				return columnVisibility;
			}
		}
	});
}
