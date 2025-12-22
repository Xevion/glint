<!-- eslint-disable-next-line @typescript-eslint/no-explicit-any -->
<script lang="ts" generics="T extends Record<string, any>">
	/* eslint-disable @typescript-eslint/no-explicit-any */
	/* eslint-disable @typescript-eslint/no-unsafe-argument */
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { DataTable } from '@careswitch/svelte-data-table';
	import * as Table from '$lib/components/ui/table';
	import TimeAgo from './time-ago.svelte';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { ExternalLink, Trash2 } from '@lucide/svelte';
	import { ensureUtc } from '$lib/utils/time';

	interface Column {
		id: string;
		key: string;
		name: string;
		render?: (value: any, row: T) => any;
		component?: 'time' | 'link-button' | 'delete-button';
		href?: (row: T) => string;
	}

	interface Props {
		data: T[];
		columns: Column[];
	}

	let { data, columns }: Props = $props();

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
				{#each table.rows as row (row.id ?? row.slug ?? JSON.stringify(row))}
					<Table.Row>
						{#each table.columns as column (column.id)}
							<Table.Cell>
								{@const colDef = columns.find((c) => c.id === column.id)}
								{#if colDef?.component === 'time' && row[column.key]}
									<TimeAgo timestamp={ensureUtc(row[column.key])} />
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
								{:else if colDef?.render}
									<!-- render functions must return safe HTML - sanitization is caller's responsibility -->
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
