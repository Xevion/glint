<script lang="ts" generics="TData">
import type { Table } from '@tanstack/table-core';
import { Button } from '$lib/components/ui/button';
import * as Select from '$lib/components/ui/select';
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from '@lucide/svelte';

interface Props {
	table: Table<TData>;
	/** Available page size options. Defaults to [10, 25, 50, 100]. */
	pageSizeOptions?: number[];
}

let { table, pageSizeOptions = [10, 25, 50, 100] }: Props = $props();

const pageIndex = $derived(table.getState().pagination.pageIndex);
const pageCount = $derived(table.getPageCount());
const totalRows = $derived(table.getFilteredRowModel().rows.length);
const selectedRows = $derived(table.getFilteredSelectedRowModel().rows.length);
const hasSelection = $derived(selectedRows > 0);
</script>

<div class="flex items-center justify-between gap-4 px-1">
	<!-- Selection count (left side) -->
	<div class="text-sm text-muted-foreground">
		{#if hasSelection}
			{selectedRows} of {totalRows} row(s) selected
		{:else}
			{totalRows} row(s)
		{/if}
	</div>

	<div class="flex items-center gap-4">
		<!-- Page size selector -->
		<div class="flex items-center gap-2">
			<span class="text-sm text-muted-foreground">Rows per page</span>
			<Select.Root
				type="single"
				value={String(table.getState().pagination.pageSize)}
				onValueChange={(v) => {
					if (v) table.setPageSize(Number(v));
				}}
			>
				<Select.Trigger size="sm" class="w-auto">
					{table.getState().pagination.pageSize}
				</Select.Trigger>
				<Select.Content>
					{#each pageSizeOptions as size (size)}
						<Select.Item value={String(size)} label={String(size)} />
					{/each}
				</Select.Content>
			</Select.Root>
		</div>

		<!-- Page indicator -->
		<span class="text-sm text-muted-foreground tabular-nums">
			Page {pageIndex + 1} of {pageCount}
		</span>

		<!-- Navigation buttons -->
		<div class="flex items-center gap-1">
			<Button
				variant="outline"
				size="icon-sm"
				onclick={() => table.firstPage()}
				disabled={!table.getCanPreviousPage()}
				title="First page"
			>
				<ChevronsLeft class="h-4 w-4" />
			</Button>
			<Button
				variant="outline"
				size="icon-sm"
				onclick={() => table.previousPage()}
				disabled={!table.getCanPreviousPage()}
				title="Previous page"
			>
				<ChevronLeft class="h-4 w-4" />
			</Button>
			<Button
				variant="outline"
				size="icon-sm"
				onclick={() => table.nextPage()}
				disabled={!table.getCanNextPage()}
				title="Next page"
			>
				<ChevronRight class="h-4 w-4" />
			</Button>
			<Button
				variant="outline"
				size="icon-sm"
				onclick={() => table.lastPage()}
				disabled={!table.getCanNextPage()}
				title="Last page"
			>
				<ChevronsRight class="h-4 w-4" />
			</Button>
		</div>
	</div>
</div>
