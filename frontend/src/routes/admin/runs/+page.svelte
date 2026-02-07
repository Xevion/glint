<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import type { CaptureRun } from '$lib/bindings';
import AdminTable from '$lib/components/AdminTable.svelte';
import { Button } from '$lib/components/ui/button';
import { RefreshCw } from '@lucide/svelte';
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

const statusColors: Record<string, string> = {
	running: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300',
	completed: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300',
	partial: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300',
	failed: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300',
	timed_out: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-300'
};

function formatDuration(run: CaptureRun): string {
	if (!run.completed_at) return 'In progress';
	const start = new Date(run.started_at).getTime();
	const end = new Date(run.completed_at).getTime();
	const seconds = Math.round((end - start) / 1000);
	if (seconds < 60) return `${seconds}s`;
	const minutes = Math.floor(seconds / 60);
	const remaining = seconds % 60;
	return `${minutes}m ${remaining}s`;
}

async function refresh() {
	refreshing = true;
	await invalidateAll();
	refreshing = false;
}
</script>

<div class="space-y-4">
	<header class="flex items-center justify-between">
		<div class="flex items-baseline gap-3">
			<h1 class="text-2xl font-semibold">Capture Runs</h1>
			<span class="text-lg text-muted-foreground">{runs.length}</span>
		</div>
		<Button variant="outline" size="icon" onclick={refresh} disabled={refreshing}>
			<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
		</Button>
	</header>

	{#if data.error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
			Error: {data.error}
		</div>
	{:else if runs.length === 0}
		<p class="text-muted-foreground">No capture runs yet.</p>
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
						class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {statusColors[row.status] ?? 'bg-gray-100 text-gray-800'}"
					>
						{row.status}
					</span>
				{:else if columnId === 'agent'}
					{row.agent_id ?? '—'}
				{:else if columnId === 'progress'}
					<div class="flex items-center gap-2">
						<span>{row.completed_items}/{row.total_items}</span>
						{#if row.failed_items > 0}
							<span class="text-xs text-red-600 dark:text-red-400">
								({row.failed_items} failed)
							</span>
						{/if}
						{#if row.total_items > 0}
							<div class="h-1.5 w-16 rounded-full bg-muted">
								<div
									class="h-full rounded-full {row.failed_items > 0 ? 'bg-yellow-500' : 'bg-green-500'}"
									style="width: {(row.completed_items / row.total_items) * 100}%"
								></div>
							</div>
						{/if}
					</div>
				{:else if columnId === 'duration'}
					{formatDuration(row)}
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
</div>
