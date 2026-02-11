<script lang="ts">
import { invalidateAll } from '$app/navigation';
import type { CaptureRun, CaptureRunItemWithContext } from '$lib/bindings';
import RefreshButton from '$lib/components/RefreshButton.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import * as Table from '$lib/components/ui/table';
import { formatDuration } from '$lib/utils/format';
import { statusColorFallback, statusColors } from '$lib/utils/status';
import { ArrowLeft, ExternalLink } from '@lucide/svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let run: CaptureRun = $derived(data.run);
let items: CaptureRunItemWithContext[] = $derived(data.items);
let refreshing = $state(false);
let expandedItem = $state<string | null>(null);

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
	await Promise.all([invalidateAll(), new Promise((r) => setTimeout(r, 300))]);
	refreshing = false;
}

const statCards = $derived([
	{ label: 'Total', value: run.total_items, color: 'text-foreground' },
	{ label: 'Completed', value: run.completed_items, color: 'text-success' },
	{ label: 'Failed', value: run.failed_items, color: 'text-destructive' },
	{ label: 'Skipped', value: run.skipped_items, color: 'text-muted-foreground' }
]);
</script>

<svelte:head><title>Run Details - Glint</title></svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<header class="space-y-2">
		<div class="flex items-center gap-2">
		<a href="/admin/runs" class="text-foreground/70 hover:text-foreground" aria-label="Back to runs">
			<ArrowLeft class="h-4 w-4" />
		</a>
			<h1 class="text-2xl font-semibold">Capture Run</h1>
			<span
class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {statusColors[
				run.status
			] ?? statusColorFallback}"
			>
				{run.status}
			</span>
			<div class="ml-auto">
				<RefreshButton {refreshing} onclick={refresh} />
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
			href="/admin/captures?runId={run.id}"
			class="inline-flex items-center gap-1 text-sm text-primary hover:underline"
		>
			View all captures from this run <ExternalLink class="h-3 w-3" />
		</a>
	</div>

	<!-- Items Table -->
	<Table.Root class="border">
		<Table.Header>
				<Table.Row class="bg-muted/50">
					<Table.Head class="px-4 py-2">Status</Table.Head>
					<Table.Head class="px-4 py-2">Shader</Table.Head>
					<Table.Head class="px-4 py-2">Scene</Table.Head>
					<Table.Head class="px-4 py-2">Profile</Table.Head>
					<Table.Head class="px-4 py-2">Duration</Table.Head>
					<Table.Head class="px-4 py-2">Error</Table.Head>
					<Table.Head class="px-4 py-2">Capture</Table.Head>
				</Table.Row>
			</Table.Header>
			<Table.Body>
				{#each items as item (item.id)}
					<Table.Row
						class="border-b transition-colors hover:bg-muted/50 {item.status === 'failed'
							? 'cursor-pointer'
							: ''}"
						onclick={() => item.status === 'failed' && toggleExpand(item.id)}
					>
						<Table.Cell class="px-4 py-2">
							<span
class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {statusColors[
								item.status
							] ?? statusColorFallback}"
							>
								{item.status}
							</span>
						</Table.Cell>
						<Table.Cell class="px-4 py-2">
							<div>
								<a
									href="/shaders/{item.shader_slug}"
									class="font-medium text-primary hover:underline"
								>
									{item.shader_name}
								</a>
								<div class="text-xs text-muted-foreground">{item.shader_version}</div>
							</div>
						</Table.Cell>
						<Table.Cell class="px-4 py-2">{item.scene_name}</Table.Cell>
						<Table.Cell class="px-4 py-2">{item.profile ?? '\u2014'}</Table.Cell>
						<Table.Cell class="px-4 py-2">{formatMs(item.duration_ms)}</Table.Cell>
					<Table.Cell class="max-w-sm px-4 py-2" title={item.error_message ?? ''}>
						<span class="line-clamp-2">{item.error_message ?? '\u2014'}</span>
					</Table.Cell>
						<Table.Cell class="px-4 py-2">
							{#if item.capture_id}
							<a
								href="/admin/captures/{item.capture_id}"
								class="text-xs text-primary hover:underline"
							>
								View
							</a>
							{:else}
								&mdash;
							{/if}
						</Table.Cell>
					</Table.Row>
					{#if expandedItem === item.id && item.status === 'failed'}
						<Table.Row>
							<Table.Cell colspan={7} class="bg-muted/30 px-4 py-3">
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
							</Table.Cell>
						</Table.Row>
					{/if}
				{/each}
			</Table.Body>
	</Table.Root>
</div>
