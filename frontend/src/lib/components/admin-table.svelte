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
}

interface Props {
	data: T[];
	columns: Column[];
	selectedId?: string | null;
	onRowClick?: (item: T) => void;
	getRowId?: (item: T) => string;
}

let { data, columns, selectedId = null, onRowClick, getRowId }: Props = $props();

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

<div class="rounded-md border">
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
