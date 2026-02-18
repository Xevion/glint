<script lang="ts" generics="TData">
import type { Table } from '@tanstack/table-core';
import * as TableUI from '$lib/components/ui/table';
import FlexRender from './flex-render.svelte';
import type { Snippet } from 'svelte';

interface Props {
	table: Table<TData>;
	/** Optional row click handler (e.g. for navigation). */
	onRowClick?: (item: TData) => void;
	/** Optional toolbar rendered above the table (filters, search, actions). */
	toolbar?: Snippet;
	/** Optional mobile card layout. If provided, cards show below `sm` and table hides. */
	card?: Snippet<[TData]>;
	/** Optional custom empty state. */
	empty?: Snippet;
}

let { table, onRowClick, toolbar, card, empty }: Props = $props();
</script>

<div class="space-y-3">
	{#if toolbar}
		{@render toolbar()}
	{/if}

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
								data-state={row.getIsSelected() ? 'selected' : undefined}
								class={onRowClick ? 'cursor-pointer' : ''}
								onclick={() => onRowClick?.(row.original)}
							>
								{#each row.getVisibleCells() as cell (cell.id)}
									<TableUI.Cell>
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
					this={onRowClick ? 'button' : 'div'}
					type={onRowClick ? 'button' : undefined}
					role={onRowClick ? 'button' : undefined}
					class="w-full rounded-lg border bg-card p-4 text-left transition-colors
						{onRowClick ? 'hover:bg-muted/50' : ''}
						{row.getIsSelected() ? 'bg-muted ring-2 ring-primary' : ''}"
					onclick={onRowClick ? () => onRowClick(row.original) : undefined}
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
