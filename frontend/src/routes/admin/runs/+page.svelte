<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import type { CaptureRun } from '$lib/bindings';
import { AdminPageHeader } from '$lib/components/admin';
import AdminTable from '$lib/components/AdminTable.svelte';
import { formatDuration } from '$lib/utils/format';
import { statusColorFallback, statusColors } from '$lib/utils/status';
import * as Tooltip from '$lib/components/ui/tooltip';
import { Alert } from '$lib/components/ui/alert';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let runs = $derived(data.runs);
let refreshing = $state(false);

const columns = [
	{ id: 'status', key: 'status', name: 'Status' },
	{ id: 'agent', key: 'agent_id', name: 'Agent' },
	{ id: 'progress', key: 'completed_items', name: 'Progress' },
	{ id: 'duration', key: 'started_at', name: 'Duration' },
	{ id: 'started_at', key: 'started_at', name: 'Started', component: 'time' as const }
];

async function refresh() {
	refreshing = true;
	await Promise.all([invalidateAll(), new Promise((r) => setTimeout(r, 300))]);
	refreshing = false;
}
</script>

<svelte:head><title>Runs - Glint</title></svelte:head>

<div class="space-y-4">
	<AdminPageHeader title="Capture Runs" count={runs.length} {refreshing} onrefresh={refresh} />

	{#if data.error}
		<Alert variant="destructive">Error: {data.error}</Alert>
	{:else if runs.length === 0}
		<p class="text-foreground">No capture runs yet.</p>
	{:else}
		<AdminTable
			data={runs}
			{columns}
			onRowClick={(run: CaptureRun) => void goto(`/admin/runs/${run.id}`)}
			getRowId={(r: CaptureRun) => r.id}
		>
			{#snippet cell({ columnId, row }: { columnId: string; value: unknown; row: CaptureRun })}
				{#if columnId === 'status'}
					<span
						class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {statusColors[row.status] ?? statusColorFallback}"
					>
						{row.status}
					</span>
				{:else if columnId === 'agent'}
					{row.agent_id ?? '—'}
			{:else if columnId === 'progress'}
				{@const total = row.total_items}
				{@const remaining = total - row.completed_items - row.failed_items - row.skipped_items}
				{@const isTimedOut = row.status === 'timed_out'}
			{#if total > 0}
				<Tooltip.Provider delayDuration={0} disableHoverableContent>
					<Tooltip.Root>
						<Tooltip.Trigger class="flex h-2.5 w-28 overflow-hidden rounded-full bg-muted">
							{#if row.completed_items > 0}
								<div
									class="bg-success"
									style="width: {(row.completed_items / total) * 100}%"
								></div>
							{/if}
							{#if row.failed_items > 0}
								<div
									class="bg-destructive"
									style="width: {(row.failed_items / total) * 100}%"
								></div>
							{/if}
							{#if row.skipped_items > 0}
								<div
									class="bg-muted-foreground/30"
									style="width: {(row.skipped_items / total) * 100}%"
								></div>
							{/if}
							{#if remaining > 0}
								<div
									class={isTimedOut ? 'bg-warning' : 'bg-info'}
									style="width: {(remaining / total) * 100}%"
								></div>
							{/if}
						</Tooltip.Trigger>
						<Tooltip.Content>
							<p class="text-xs">
								{row.completed_items} completed, {row.failed_items} failed, {row.skipped_items} skipped, {remaining} remaining
							</p>
						</Tooltip.Content>
					</Tooltip.Root>
				</Tooltip.Provider>
				{:else}
					<span class="text-muted-foreground">&mdash;</span>
				{/if}
				{:else if columnId === 'duration'}
					{formatDuration(row)}
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
</div>
