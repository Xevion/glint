<script lang="ts">
import { Button } from '$lib/components/ui/button';
import * as Select from '$lib/components/ui/select';
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from '@lucide/svelte';
import type { Table } from '@tanstack/table-core';
import { getContext } from 'svelte';
import { DATA_VIEW_TABLE_KEY, getDataViewContext } from './context';

interface InfiniteProps {
	style: 'infinite';
	/** Root margin for the IntersectionObserver (default: '200px'). */
	rootMargin?: string;
}

interface LoadMoreProps {
	style: 'load-more';
	/** Label for the load more button. */
	label?: string;
}

interface PagesProps {
	style: 'pages';
	/** Available page size options (only used when reading from TanStack context). */
	pageSizeOptions?: number[];
	/** Explicit page (1-indexed) for server-side pagination. */
	page?: number;
	/** Explicit total pages for server-side pagination. */
	totalPages?: number;
	/** Callback when page changes (server-side pagination). */
	onPageChange?: (page: number) => void;
}

type Props = InfiniteProps | LoadMoreProps | PagesProps;

let props: Props = $props();

const ctx = getDataViewContext<unknown>();
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const tanstackTable = getContext<Table<any> | undefined>(DATA_VIEW_TABLE_KEY);

let sentinelEl = $state<HTMLDivElement | null>(null);

$effect(() => {
	if (props.style !== 'infinite') return;
	const el = sentinelEl;
	if (!el) return;

	const rootMargin = props.rootMargin ?? '200px';

	const observer = new IntersectionObserver(
		(entries) => {
			const entry = entries[0];
			if (entry?.isIntersecting && ctx.hasMore && !ctx.loading) {
				void ctx.loadMore();
			}
		},
		{ rootMargin }
	);

	observer.observe(el);
	return () => observer.disconnect();
});

const tanstackPageIndex = $derived(tanstackTable?.getState().pagination.pageIndex ?? 0);
const tanstackPageCount = $derived(tanstackTable?.getPageCount() ?? 0);
const tanstackTotalRows = $derived(tanstackTable?.getFilteredRowModel().rows.length ?? 0);
</script>

{#if props.style === 'infinite'}
	{#if ctx.hasMore || ctx.loading}
		<div bind:this={sentinelEl} class="flex justify-center py-8">
			{#if ctx.loading}
				<div
					role="status"
					aria-label="Loading more items"
					class="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent"
				></div>
			{/if}
		</div>
	{/if}

{:else if props.style === 'load-more'}
	{#if ctx.hasMore}
		<div class="mt-6 flex justify-center">
			<Button
				variant="outline"
				onclick={() => void ctx.loadMore()}
				disabled={ctx.loading}
			>
				{#if ctx.loading}
					Loading...
				{:else}
					{props.label ?? 'Load More'}
				{/if}
			</Button>
		</div>
	{/if}

{:else if props.style === 'pages'}
	{@const pageSizeOptions = props.pageSizeOptions ?? [10, 25, 50, 100]}
	{@const isServerSide = props.page != null && props.totalPages != null}

	{#if isServerSide}
		<!-- Server-side pagination -->
		{@const currentPage = props.page!}
		{@const totalPages = props.totalPages!}
		{#if totalPages > 1}
			<div class="flex items-center justify-between gap-4 px-1">
				<div class="text-sm text-muted-foreground">
					{#if ctx.totalCount != null}
						{ctx.totalCount} total
					{/if}
				</div>
				<div class="flex items-center gap-4">
					<span class="text-sm text-muted-foreground tabular-nums">
						Page {currentPage} of {totalPages}
					</span>
					<div class="flex items-center gap-1">
						<Button
							variant="outline"
							size="icon-sm"
							onclick={() => props.onPageChange?.(currentPage - 1)}
							disabled={currentPage <= 1}
							title="Previous page"
						>
							<ChevronLeft class="h-4 w-4" />
						</Button>
						<Button
							variant="outline"
							size="icon-sm"
							onclick={() => props.onPageChange?.(currentPage + 1)}
							disabled={currentPage >= totalPages}
							title="Next page"
						>
							<ChevronRight class="h-4 w-4" />
						</Button>
					</div>
				</div>
			</div>
		{/if}
	{:else if tanstackTable}
		<!-- TanStack client-side pagination -->
		<div class="flex items-center justify-between gap-4 px-1">
			<div class="text-sm text-muted-foreground">
				{tanstackTotalRows} row(s)
			</div>
			<div class="flex items-center gap-4">
				<div class="flex items-center gap-2">
					<span class="text-sm text-muted-foreground">Rows per page</span>
					<Select.Root
						type="single"
						value={String(tanstackTable.getState().pagination.pageSize)}
						onValueChange={(v) => {
							if (v) tanstackTable.setPageSize(Number(v));
						}}
					>
						<Select.Trigger size="sm" class="w-auto">
							{tanstackTable.getState().pagination.pageSize}
						</Select.Trigger>
						<Select.Content>
							{#each pageSizeOptions as size (size)}
								<Select.Item value={String(size)} label={String(size)} />
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<span class="text-sm text-muted-foreground tabular-nums">
					Page {tanstackPageIndex + 1} of {tanstackPageCount}
				</span>

				<div class="flex items-center gap-1">
					<Button
						variant="outline"
						size="icon-sm"
						onclick={() => tanstackTable.firstPage()}
						disabled={!tanstackTable.getCanPreviousPage()}
						title="First page"
					>
						<ChevronsLeft class="h-4 w-4" />
					</Button>
					<Button
						variant="outline"
						size="icon-sm"
						onclick={() => tanstackTable.previousPage()}
						disabled={!tanstackTable.getCanPreviousPage()}
						title="Previous page"
					>
						<ChevronLeft class="h-4 w-4" />
					</Button>
					<Button
						variant="outline"
						size="icon-sm"
						onclick={() => tanstackTable.nextPage()}
						disabled={!tanstackTable.getCanNextPage()}
						title="Next page"
					>
						<ChevronRight class="h-4 w-4" />
					</Button>
					<Button
						variant="outline"
						size="icon-sm"
						onclick={() => tanstackTable.lastPage()}
						disabled={!tanstackTable.getCanNextPage()}
						title="Last page"
					>
						<ChevronsRight class="h-4 w-4" />
					</Button>
				</div>
			</div>
		</div>
	{/if}
{/if}
