<!-- eslint-disable-next-line @typescript-eslint/no-explicit-any -->
<script lang="ts" generics="T extends Record<string, any>">
import type { Snippet } from 'svelte';
import { DataTable } from '@careswitch/svelte-data-table';
import * as Table from '$lib/components/ui/table';
import TimeAgo from './TimeAgo.svelte';
import { goto } from '$app/navigation';
import { Button } from '$lib/components/ui/button';
import { ExternalLink, Trash2 } from '@lucide/svelte';

interface Column {
	id: string;
	key: string;
	name: string;
	component?: 'time' | 'link-button' | 'delete-button';
	href?: (row: T) => string;
	/** Column to use as card title on mobile (first truthy one wins) */
	cardTitle?: boolean;
	/** Hide this column in mobile card view */
	hideOnMobile?: boolean;
}

interface Props {
	data: T[];
	columns: Column[];
	selectedId?: string | null;
	onRowClick?: (item: T) => void;
	getRowId?: (item: T) => string;
	cell?: Snippet<[{ columnId: string; value: unknown; row: T }]>;
}

let { data, columns, selectedId = null, onRowClick, getRowId, cell }: Props = $props();

// Find the title column for mobile cards (first column with cardTitle=true, or first column)
const titleColumn = $derived(columns.find((c) => c.cardTitle) ?? columns[0]);
// Other columns to show in card body (excluding title and hidden columns)
const cardBodyColumns = $derived(columns.filter((c) => c !== titleColumn && !c.hideOnMobile));

function extractRowId(row: T): string {
	if (getRowId) return getRowId(row);
	if ('id' in row && typeof row.id === 'string') return row.id;
	if ('slug' in row && typeof row.slug === 'string') return row.slug;
	return JSON.stringify(row);
}

const table = $derived(
	new DataTable({
		data,
		columns,
		pageSize: data.length || 1
	})
);
</script>

<!-- Desktop table view -->
<div class="hidden rounded-md border sm:block">
	<Table.Root>
		<Table.Header>
			<Table.Row>
				{#each table.columns as column (column.id)}
					<Table.Head>{column.name}</Table.Head>
				{/each}
			</Table.Row>
		</Table.Header>
		<Table.Body>
			{#if table.rows.length}
				{#each table.rows as row (extractRowId(row))}
					{@const rowId = extractRowId(row)}
					<Table.Row
						class={[
							onRowClick && 'cursor-pointer hover:bg-muted/50',
							selectedId === rowId && 'bg-muted'
						]
							.filter(Boolean)
							.join(' ')}
						onclick={() => onRowClick?.(row)}
					>
						{#each table.columns as column (column.id)}
							<Table.Cell>
								{@const colDef = columns.find((c) => c.id === column.id)}
								{#if colDef?.component === 'time' && row[column.key]}
									<TimeAgo timestamp={row[column.key]} />
								{:else if colDef?.component === 'link-button' && colDef.href}
									<Button
										variant="ghost"
										size="sm"
										onclick={() => {
											if (colDef.href) {
												void goto(colDef.href(row));
											}
										}}
									>
										<ExternalLink class="h-4 w-4" />
									</Button>
								{:else if colDef?.component === 'delete-button'}
									<Button variant="ghost" size="sm" title="Delete (not implemented)">
										<Trash2 class="h-4 w-4" />
									</Button>
							{:else if cell}
									{@render cell({ columnId: column.id, value: row[column.key], row })}
								{:else}
									{row[column.key] ?? '-'}
								{/if}
							</Table.Cell>
						{/each}
					</Table.Row>
				{/each}
			{:else}
				<Table.Row>
					<Table.Cell colspan={columns.length} class="h-24 text-center">No results.</Table.Cell>
				</Table.Row>
			{/if}
		</Table.Body>
	</Table.Root>
</div>

<!-- Mobile card view -->
<div class="flex flex-col gap-3 sm:hidden">
	{#if table.rows.length}
		{#each table.rows as row (extractRowId(row))}
			{@const rowId = extractRowId(row)}
			<button
				type="button"
				class="w-full rounded-lg border bg-card p-4 text-left transition-colors {onRowClick
					? 'cursor-pointer hover:bg-muted/50'
					: ''} {selectedId === rowId ? 'bg-muted ring-2 ring-primary' : ''}"
				onclick={() => onRowClick?.(row)}
				disabled={!onRowClick}
			>
				<!-- Card title -->
				<div class="mb-2 font-medium">
					{#if cell}
						{@render cell({ columnId: titleColumn.id, value: row[titleColumn.key], row })}
					{:else}
						{row[titleColumn.key] ?? '-'}
					{/if}
				</div>

				<!-- Card body fields -->
				<div class="space-y-1 text-sm text-muted-foreground">
					{#each cardBodyColumns as col (col.id)}
						<div class="flex items-center justify-between gap-2">
							<span class="shrink-0 font-medium">{col.name}:</span>
							<span class="truncate text-right">
								{#if col.component === 'time' && row[col.key]}
									<TimeAgo timestamp={row[col.key]} />
								{:else if cell}
									{@render cell({ columnId: col.id, value: row[col.key], row })}
								{:else}
									{row[col.key] ?? '-'}
								{/if}
							</span>
						</div>
					{/each}
				</div>

				</button>
		{/each}
	{:else}
		<div class="rounded-lg border bg-card p-8 text-center text-muted-foreground">No results.</div>
	{/if}
</div>
