<script lang="ts">
import { goto } from '$app/navigation';
import type { CaptureRun } from '$lib/bindings';
import { AdminBreadcrumb } from '$lib/components/admin';
import { DataTable, DataTablePagination, createDataTable } from '$lib/components/data-table';
import { Alert } from '$lib/components/ui/alert';
import { formatDuration } from '$lib/utils/format';
import { statusColorFallback, statusColors } from '$lib/utils/status';
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
	<AdminBreadcrumb segments={[{ label: 'Capture Runs' }]} />

	{#if data.error}
		<Alert variant="destructive">Error: {data.error}</Alert>
	{:else if runs.length === 0}
		<p class="text-muted-foreground">No capture runs yet.</p>
	{:else}
		<DataTable {table} onRowClick={(run: CaptureRun) => void goto(`/admin/runs/${run.id}`)}>
			{#snippet card(run: CaptureRun)}
				{@const remaining = run.total_items - run.completed_items - run.failed_items - run.skipped_items}
				<div class="flex items-start justify-between gap-3">
					<div class="min-w-0 flex-1 space-y-1">
						<div class="flex items-center gap-2">
							<span
								class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {statusColors[run.status] ?? statusColorFallback}"
							>
								{run.status}
							</span>
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
						{formatDuration(run)}
					</div>
				</div>
			{/snippet}
		</DataTable>
		<div class="mt-3">
			<DataTablePagination {table} />
		</div>
	{/if}
</div>
