<script lang="ts" generics="T">
import { FlexRender } from '$lib/components/data-table';
import * as TableUI from '$lib/components/ui/table';
import type { ColumnDef } from '@tanstack/table-core';
import type { Snippet } from 'svelte';
import { setContext } from 'svelte';
import { DATA_VIEW_TABLE_KEY, getDataViewContext } from './context';
import { createDataTable } from './table-bridge.svelte';

interface Props {
	/** TanStack column definitions. */
	columns: ColumnDef<T>[];
	/** URL generator for row navigation. */
	getRowHref?: (item: T) => string;
	/** Row click handler for non-navigation actions. */
	onRowClick?: (item: T) => void;
	/** Mobile card fallback (hidden on sm+). */
	card?: Snippet<[T]>;
	/** Custom empty state for the table. */
	empty?: Snippet;
	/** Page size for TanStack client-side pagination (false to disable). */
	pageSize?: number | false;
	/**
	 * Maps column IDs (accessorKey) to server sort field strings.
	 * When the parent DataView has a list state, this bridges column header
	 * clicks to the list's sort → URL sync → server re-fetch.
	 *
	 * Example: `{ capturedAt: 'capturedAt', shaderName: 'name' }`
	 */
	sortMapping?: Record<string, string>;
	/** Container class override. */
	class?: string;
}

let {
	columns,
	getRowHref,
	onRowClick,
	card,
	empty,
	pageSize = false,
	sortMapping,
	class: className
}: Props = $props();

const ctx = getDataViewContext<T>();

// Bridge column sorting to the list's server-driven sort when both
// sortMapping and a list state (with sort support) are available.
// svelte-ignore state_referenced_locally
const sortAccessor =
	sortMapping && ctx.listState
		? {
				get: () => ctx.listState!.sort,
				set: (v: string) => {
					ctx.listState!.sort = v;
				}
			}
		: undefined;

// svelte-ignore state_referenced_locally
const table = createDataTable<T>({
	get data() {
		return ctx.items;
	},
	columns,
	pageSize,
	selection: false,
	sortMapping,
	sort: sortAccessor
});

// Expose table instance to child pagination component
setContext(DATA_VIEW_TABLE_KEY, table);
</script>

{#if ctx.viewMode === 'table'}
<div class={className}>
	<!-- Desktop table (hidden on mobile if card snippet is provided) -->
	<div class={card ? 'hidden sm:block' : ''}>
		<div class="rounded-md border">
			<TableUI.Root>
				<TableUI.Header>
					{#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
						<TableUI.Row>
							{#each headerGroup.headers as header (header.id)}
								{@const defSize = header.column.columnDef.size}
								{@const defMinSize = header.column.columnDef.minSize}
								{@const colStyle =
									[
										defSize != null ? `width:${defSize}px` : undefined,
										defMinSize != null ? `min-width:${defMinSize}px` : undefined
									]
										.filter(Boolean)
										.join(';') || undefined}
								<TableUI.Head style={colStyle}>
									{#if !header.isPlaceholder}
										<FlexRender
											content={header.column.columnDef.header}
											context={header.getContext()}
										/>
									{/if}
								</TableUI.Head>
							{/each}
						</TableUI.Row>
					{/each}
				</TableUI.Header>
				<TableUI.Body>
					{#if table.getRowModel().rows.length}
						{#each table.getRowModel().rows as row (row.id)}
							<TableUI.Row
								class="{getRowHref || onRowClick ? 'cursor-pointer' : ''} {getRowHref ? 'relative' : ''}"
								onclick={getRowHref
									? undefined
									: onRowClick
										? () => onRowClick(row.original)
										: undefined}
							>
								{#each row.getVisibleCells() as cell, i (cell.id)}
									<TableUI.Cell>
										{#if i === 0 && getRowHref}
											<a
												href={getRowHref(row.original)}
												class="absolute inset-0 z-0"
												tabindex={-1}
												aria-hidden="true"
											></a>
										{/if}
										<FlexRender
											content={cell.column.columnDef.cell}
											context={cell.getContext()}
										/>
									</TableUI.Cell>
								{/each}
							</TableUI.Row>
						{/each}
					{:else}
						<TableUI.Row>
							<TableUI.Cell
								colspan={table.getAllColumns().length}
								class="h-24 text-center"
							>
								{#if empty}
									{@render empty()}
								{:else}
									No results.
								{/if}
							</TableUI.Cell>
						</TableUI.Row>
					{/if}
				</TableUI.Body>
			</TableUI.Root>
		</div>
	</div>

	<!-- Mobile card view (shown below sm when card snippet is provided) -->
	{#if card}
		<div class="flex flex-col gap-3 sm:hidden">
			{#if table.getRowModel().rows.length}
				{#each table.getRowModel().rows as row (row.id)}
					<svelte:element
						this={getRowHref ? 'a' : onRowClick ? 'button' : 'div'}
						href={getRowHref ? getRowHref(row.original) : undefined}
						type={!getRowHref && onRowClick ? 'button' : undefined}
						role={!getRowHref && onRowClick ? 'button' : undefined}
						class="w-full rounded-lg border bg-card p-4 text-left transition-colors
							{onRowClick || getRowHref ? 'hover:bg-muted/50' : ''}"
						onclick={!getRowHref && onRowClick
							? () => onRowClick(row.original)
							: undefined}
					>
						{@render card(row.original)}
					</svelte:element>
				{/each}
			{:else if empty}
				{@render empty()}
			{:else}
				<div class="rounded-lg border bg-card p-8 text-center text-muted-foreground">
					No results.
				</div>
			{/if}
		</div>
	{/if}
</div>
{/if}
