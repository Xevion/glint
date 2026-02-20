<script lang="ts">
import type { CaptureRun } from '$lib/bindings';
import Breadcrumb from '$lib/components/Breadcrumb.svelte';
import { DataTable, DataTablePagination, createDataTable } from '$lib/components/data-table';
import { DataList } from '$lib/components/data-list';
import { StatusBadge } from '$lib/components/ui/status-badge';
import { formatElapsedDuration } from '$lib/utils/format';
import type { PageData } from './$types';
import { columns } from './columns.js';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let runs = $derived(data.runs);
const table = createDataTable<CaptureRun>({
	get data() {
		return runs;
	},
	columns,
	pageSize: 25,
	selection: false
});
</script>

<svelte:head><title>Runs - Glint</title></svelte:head>

<div class="space-y-4">
	<Breadcrumb segments={[{ label: 'Capture Runs' }]} />

	<DataList items={runs} error={data.error} emptyMessage="No capture runs yet.">
		{#snippet content()}
			<DataTable {table} getRowHref={(run: CaptureRun) => `/admin/runs/${run.id}`}>
				{#snippet card(run: CaptureRun)}
					{@const remaining = run.total_items - run.completed_items - run.failed_items - run.skipped_items}
					<div class="flex items-start justify-between gap-3">
						<div class="min-w-0 flex-1 space-y-1">
							<div class="flex items-center gap-2">
								<StatusBadge status={run.status}>{run.status}</StatusBadge>
								<span class="text-xs text-muted-foreground">
									{run.agent_id ?? '—'}
								</span>
							</div>
							{#if run.total_items > 0}
								<div class="flex h-2 w-full overflow-hidden rounded-full bg-muted">
									{#if run.completed_items > 0}
										<div
											class="bg-success"
											style="width: {(run.completed_items / run.total_items) * 100}%"
										></div>
									{/if}
									{#if run.failed_items > 0}
										<div
											class="bg-destructive"
											style="width: {(run.failed_items / run.total_items) * 100}%"
										></div>
									{/if}
									{#if run.skipped_items > 0}
										<div
											class="bg-muted-foreground/30"
											style="width: {(run.skipped_items / run.total_items) * 100}%"
										></div>
									{/if}
									{#if remaining > 0}
										<div
											class={run.status === 'timed_out' ? 'bg-warning' : 'bg-info'}
											style="width: {(remaining / run.total_items) * 100}%"
										></div>
									{/if}
								</div>
								<p class="text-xs text-muted-foreground">
									{run.completed_items}/{run.total_items} done
									{#if run.failed_items > 0}, {run.failed_items} failed{/if}
								</p>
							{/if}
						</div>
						<div class="shrink-0 text-right text-xs text-muted-foreground">
							{formatElapsedDuration(run)}
						</div>
					</div>
				{/snippet}
			</DataTable>
			<div class="mt-3">
				<DataTablePagination {table} />
			</div>
		{/snippet}
	</DataList>
</div>
