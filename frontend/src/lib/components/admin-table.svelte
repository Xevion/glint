<!-- eslint-disable-next-line @typescript-eslint/no-explicit-any -->
<script lang="ts" generics="T extends Record<string, any>">
/* eslint-disable @typescript-eslint/no-explicit-any */

import { DataTable } from '@careswitch/svelte-data-table';
import * as Table from '$lib/components/ui/table';
import TimeAgo from './time-ago.svelte';
import { goto } from '$app/navigation';
import { Button } from '$lib/components/ui/button';
import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
import { ExternalLink, Trash2, Ellipsis, CircleX, RotateCcw, LockOpen, Info } from '@lucide/svelte';

interface Column {
	id: string;
	key: string;
	name: string;
	render?: (value: any, row: T) => any;
	component?: 'time' | 'link-button' | 'delete-button' | 'job-actions';
	href?: (row: T) => string;
	onAction?: (action: string, row: T) => void;
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
}

let { data, columns, selectedId = null, onRowClick, getRowId }: Props = $props();

// Find the title column for mobile cards (first column with cardTitle=true, or first column)
const titleColumn = $derived(columns.find((c) => c.cardTitle) ?? columns[0]);
// Other columns to show in card body (excluding title and hidden columns)
const cardBodyColumns = $derived(
	columns.filter((c) => c !== titleColumn && !c.hideOnMobile && c.component !== 'job-actions')
);
// Action columns (shown at bottom of card)
const actionColumns = $derived(columns.filter((c) => c.component === 'job-actions'));

function extractRowId(row: T): string {
	if (getRowId) return getRowId(row);
	if ('id' in row && typeof row.id === 'string') return row.id;
	if ('slug' in row && typeof row.slug === 'string') return row.slug;
	return JSON.stringify(row);
}

const table = $derived(
	new DataTable({
		data,
		columns
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
								{:else if colDef?.component === 'job-actions' && colDef.onAction}
									{@const status = String(row.status)}
									<DropdownMenu.Root>
										<DropdownMenu.Trigger>
											<Button variant="ghost" size="sm">
												<Ellipsis class="h-4 w-4" />
											</Button>
										</DropdownMenu.Trigger>
										<DropdownMenu.Content align="end">
											<DropdownMenu.Item onclick={() => colDef.onAction?.('view-details', row)}>
												<Info class="mr-2 h-4 w-4" />
												View Details
											</DropdownMenu.Item>
											{#if status === 'pending' || status === 'claimed'}
												<DropdownMenu.Item onclick={() => colDef.onAction?.('cancel', row)}>
													<CircleX class="mr-2 h-4 w-4" />
													Cancel Job
												</DropdownMenu.Item>
											{/if}
											{#if status === 'failed'}
												<DropdownMenu.Item onclick={() => colDef.onAction?.('retry', row)}>
													<RotateCcw class="mr-2 h-4 w-4" />
													Retry Job
												</DropdownMenu.Item>
											{/if}
											{#if status === 'claimed' || status === 'running'}
												<DropdownMenu.Item onclick={() => colDef.onAction?.('release', row)}>
													<LockOpen class="mr-2 h-4 w-4" />
													Release Claim
												</DropdownMenu.Item>
											{/if}
											<DropdownMenu.Separator />
											<DropdownMenu.Item
												onclick={() => colDef.onAction?.('delete', row)}
												class="text-destructive focus:text-destructive"
											>
												<Trash2 class="mr-2 h-4 w-4" />
												Delete Job
											</DropdownMenu.Item>
										</DropdownMenu.Content>
									</DropdownMenu.Root>
								{:else if colDef?.render}
									<!--
									WARNING: XSS RISK - render functions return raw HTML.
									All dynamic values MUST be escaped using escapeHtml() from '$lib/utils/display'.
									Example: `<a href="${escapeHtml(url)}">${escapeHtml(name)}</a>`
								-->
									<!-- eslint-disable-next-line svelte/no-at-html-tags -->
									{@html colDef.render(row[column.key], row)}
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
					{#if titleColumn.render}
						<!-- eslint-disable-next-line svelte/no-at-html-tags -->
						{@html titleColumn.render(row[titleColumn.key], row)}
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
								{:else if col.render}
									<!-- eslint-disable-next-line svelte/no-at-html-tags -->
									{@html col.render(row[col.key], row)}
								{:else}
									{row[col.key] ?? '-'}
								{/if}
							</span>
						</div>
					{/each}
				</div>

				<!-- Card actions -->
				{#if actionColumns.length > 0}
					<div class="mt-3 flex items-center justify-end gap-2 border-t pt-3">
						{#each actionColumns as col (col.id)}
							{#if col.component === 'job-actions' && col.onAction}
								{@const status = String(row.status)}
								<DropdownMenu.Root>
									<DropdownMenu.Trigger>
										<Button variant="outline" size="sm" class="min-h-10 min-w-10">
											<Ellipsis class="h-4 w-4" />
											<span class="ml-1">Actions</span>
										</Button>
									</DropdownMenu.Trigger>
									<DropdownMenu.Content align="end">
										<DropdownMenu.Item onclick={() => col.onAction?.('view-details', row)}>
											<Info class="mr-2 h-4 w-4" />
											View Details
										</DropdownMenu.Item>
										{#if status === 'pending' || status === 'claimed'}
											<DropdownMenu.Item onclick={() => col.onAction?.('cancel', row)}>
												<CircleX class="mr-2 h-4 w-4" />
												Cancel Job
											</DropdownMenu.Item>
										{/if}
										{#if status === 'failed'}
											<DropdownMenu.Item onclick={() => col.onAction?.('retry', row)}>
												<RotateCcw class="mr-2 h-4 w-4" />
												Retry Job
											</DropdownMenu.Item>
										{/if}
										{#if status === 'claimed' || status === 'running'}
											<DropdownMenu.Item onclick={() => col.onAction?.('release', row)}>
												<LockOpen class="mr-2 h-4 w-4" />
												Release Claim
											</DropdownMenu.Item>
										{/if}
										<DropdownMenu.Separator />
										<DropdownMenu.Item
											onclick={() => col.onAction?.('delete', row)}
											class="text-destructive focus:text-destructive"
										>
											<Trash2 class="mr-2 h-4 w-4" />
											Delete Job
										</DropdownMenu.Item>
									</DropdownMenu.Content>
								</DropdownMenu.Root>
							{/if}
						{/each}
					</div>
				{/if}
			</button>
		{/each}
	{:else}
		<div class="rounded-lg border bg-card p-8 text-center text-muted-foreground">No results.</div>
	{/if}
</div>
