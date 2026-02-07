<script lang="ts">
import { invalidateAll } from '$app/navigation';
import type { CaptureRun, CaptureRunItemWithContext } from '$lib/bindings';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { Button } from '$lib/components/ui/button';
import { ArrowLeft, ExternalLink, RefreshCw } from '@lucide/svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let run: CaptureRun = $derived(data.run);
let items: CaptureRunItemWithContext[] = $derived(data.items);
let refreshing = $state(false);
let expandedItem = $state<string | null>(null);

const statusColors: Record<string, string> = {
	running: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300',
	completed: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300',
	partial: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300',
	failed: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300',
	pending: 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-300',
	skipped: 'bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400'
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

function formatMs(ms: number | null): string {
	if (ms == null) return '\u2014';
	if (ms < 1000) return `${ms}ms`;
	return `${(ms / 1000).toFixed(1)}s`;
}

function toggleExpand(id: string) {
	expandedItem = expandedItem === id ? null : id;
}

async function refresh() {
	refreshing = true;
	await invalidateAll();
	refreshing = false;
}

const statCards = $derived([
	{ label: 'Total', value: run.total_items, color: 'text-foreground' },
	{
		label: 'Completed',
		value: run.completed_items,
		color: 'text-green-600 dark:text-green-400'
	},
	{ label: 'Failed', value: run.failed_items, color: 'text-red-600 dark:text-red-400' },
	{ label: 'Skipped', value: run.skipped_items, color: 'text-gray-500' }
]);
</script>

<div class="space-y-6">
	<!-- Header -->
	<header class="space-y-2">
		<div class="flex items-center gap-2">
		<a href="/admin/runs" class="text-muted-foreground hover:text-foreground" aria-label="Back to runs">
			<ArrowLeft class="h-4 w-4" />
		</a>
			<h1 class="text-2xl font-semibold">Capture Run</h1>
			<span
				class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {statusColors[
					run.status
				] ?? ''}"
			>
				{run.status}
			</span>
			<div class="ml-auto">
				<Button variant="outline" size="icon" onclick={refresh} disabled={refreshing}>
					<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
				</Button>
			</div>
		</div>
		<div class="flex flex-wrap gap-x-6 gap-y-1 text-sm text-muted-foreground">
			<span>ID: <code class="text-xs">{run.id}</code></span>
			{#if run.agent_id}
				<span>Agent: {run.agent_id}</span>
			{/if}
			<span>Started: <TimeAgo timestamp={run.started_at} /></span>
			{#if run.completed_at}
				<span>Duration: {formatDuration(run)}</span>
			{/if}
		</div>
	</header>

	<!-- Stat Cards -->
	<div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
		{#each statCards as card (card.label)}
			<div class="rounded-lg border bg-card p-4">
				<div class="text-2xl font-bold {card.color}">{card.value}</div>
				<div class="text-sm text-muted-foreground">{card.label}</div>
			</div>
		{/each}
	</div>

	<!-- Link to captures filtered by this run -->
	<div>
		<a
			href="/admin/captures?run_id={run.id}"
			class="inline-flex items-center gap-1 text-sm text-primary hover:underline"
		>
			View all captures from this run <ExternalLink class="h-3 w-3" />
		</a>
	</div>

	<!-- Items Table -->
	<div class="rounded-md border">
		<table class="w-full text-sm">
			<thead>
				<tr class="border-b bg-muted/50">
					<th class="px-4 py-2 text-left font-medium">Status</th>
					<th class="px-4 py-2 text-left font-medium">Shader</th>
					<th class="px-4 py-2 text-left font-medium">Scene</th>
					<th class="px-4 py-2 text-left font-medium">Profile</th>
					<th class="px-4 py-2 text-left font-medium">Duration</th>
					<th class="px-4 py-2 text-left font-medium">Error</th>
					<th class="px-4 py-2 text-left font-medium">Capture</th>
				</tr>
			</thead>
			<tbody>
				{#each items as item (item.id)}
					<tr
						class="border-b transition-colors hover:bg-muted/50 {item.status === 'failed'
							? 'cursor-pointer'
							: ''}"
						onclick={() => item.status === 'failed' && toggleExpand(item.id)}
					>
						<td class="px-4 py-2">
							<span
								class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {statusColors[
									item.status
								] ?? ''}"
							>
								{item.status}
							</span>
						</td>
						<td class="px-4 py-2">
							<div>
								<a
									href="/shaders/{item.shader_slug}"
									class="font-medium text-primary hover:underline"
								>
									{item.shader_name}
								</a>
								<div class="text-xs text-muted-foreground">{item.shader_version}</div>
							</div>
						</td>
						<td class="px-4 py-2">{item.scene_name}</td>
						<td class="px-4 py-2">{item.profile ?? '\u2014'}</td>
						<td class="px-4 py-2">{formatMs(item.duration_ms)}</td>
						<td class="max-w-48 truncate px-4 py-2" title={item.error_message ?? ''}>
							{item.error_message ?? '\u2014'}
						</td>
						<td class="px-4 py-2">
							{#if item.capture_id}
								<a
									href="/admin/captures?selected={item.capture_id}"
									class="text-xs text-primary hover:underline"
								>
									View
								</a>
							{:else}
								&mdash;
							{/if}
						</td>
					</tr>
					{#if expandedItem === item.id && item.status === 'failed'}
						<tr>
							<td colspan="7" class="bg-muted/30 px-4 py-3">
								<div class="space-y-2">
									{#if item.error_message}
										<div>
											<span class="text-xs font-medium text-muted-foreground">Error:</span>
											<p class="text-sm">{item.error_message}</p>
										</div>
									{/if}
									{#if item.error_log}
										<div>
											<span class="text-xs font-medium text-muted-foreground">Log:</span>
											<pre
												class="mt-1 max-h-64 overflow-auto rounded bg-muted p-3 text-xs">{item.error_log}</pre>
										</div>
									{/if}
								</div>
							</td>
						</tr>
					{/if}
				{/each}
			</tbody>
		</table>
	</div>
</div>
