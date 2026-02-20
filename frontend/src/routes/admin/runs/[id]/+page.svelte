<script lang="ts">
import { invalidateAll } from '$app/navigation';
import { AdminBreadcrumb } from '$lib/components/admin';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import RefreshButton from '$lib/components/RefreshButton.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import * as Table from '$lib/components/ui/table';
import { formatDuration } from '$lib/utils/format';
import { StatusBadge, type StatusBadgeStatus } from '$lib/components/ui/status-badge';
import { ExternalLink } from '@lucide/svelte';
import type { PageData } from './$types';
import type { AdminCaptureRunData, AdminCaptureRunItem } from './queries';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let run: AdminCaptureRunData = $derived(data.run);
let items: AdminCaptureRunItem[] = $derived(run.items);
let refreshing = $state(false);
let expandedItem = $state<string | null>(null);

function formatMs(ms: number | null | undefined): string {
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

// Map GraphQL enum values (RUNNING, COMPLETED, etc.) to the snake_case keys used by StatusBadge
function normalizeStatus(status: string): StatusBadgeStatus {
	return status.toLowerCase().replace(/_/g, '_') as NonNullable<StatusBadgeStatus>;
}

const statCards = $derived([
	{ label: 'Total', value: run.totalItems, color: 'text-foreground' },
	{ label: 'Completed', value: run.completedItems, color: 'text-success' },
	{ label: 'Failed', value: run.failedItems, color: 'text-destructive' },
	{ label: 'Skipped', value: run.skippedItems, color: 'text-muted-foreground' }
]);
</script>

<svelte:head><title>Run Details - Glint</title></svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="space-y-2">
		<div class="flex items-center gap-2">
			<AdminBreadcrumb
				segments={[{ label: 'Runs', href: '/admin/runs' }, { label: 'Capture Run' }]}
			>
				{#snippet trailing()}
					<StatusBadge status={normalizeStatus(run.status)} class="ml-2">{normalizeStatus(run.status)}</StatusBadge>
				{/snippet}
			</AdminBreadcrumb>
			<div class="ml-auto">
				<RefreshButton {refreshing} onclick={refresh} />
			</div>
		</div>
		<div class="flex flex-wrap gap-x-6 gap-y-1 text-sm text-foreground">
			<span>ID: <code class="text-xs">{run.id}</code></span>
			{#if run.agentId}
				<span>Agent: {run.agentId}</span>
			{/if}
			<span>Started: <TimeAgo timestamp={run.startedAt} /></span>
			{#if run.completedAt}
				<span
					>Duration: {formatDuration({
						started_at: run.startedAt,
						completed_at: run.completedAt
					})}</span
				>
			{/if}
		</div>
	</div>

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
				<Table.Head class="w-20 px-4 py-2">Preview</Table.Head>
				<Table.Head class="px-4 py-2">Status</Table.Head>
				<Table.Head class="px-4 py-2">Shader</Table.Head>
				<Table.Head class="px-4 py-2">Scene</Table.Head>
				<Table.Head class="px-4 py-2">Duration</Table.Head>
				<Table.Head class="px-4 py-2">Error</Table.Head>
			</Table.Row>
		</Table.Header>
		<Table.Body>
			{#each items as item (item.id)}
				<Table.Row
					class="relative border-b transition-colors hover:bg-muted/50 {item.captureId ? 'cursor-pointer' : ''} {item.status === 'FAILED' ? 'cursor-pointer' : ''}"
					onclick={() => item.status === 'FAILED' && toggleExpand(item.id)}
				>
					<Table.Cell class="px-4 py-2">
						{#if item.captureId}
							<a
								href="/admin/captures/{item.captureId}"
								class="absolute inset-0 z-0"
								tabindex="-1"
								aria-hidden="true"
							></a>
						{/if}
						{#if item.imagePath ?? item.thumbhash}
							<CaptureImage
								src={item.imagePath}
								thumbhash={item.thumbhash}
								preset="thumbnail"
								alt="Capture preview"
								class="h-full w-full object-cover"
								containerClass="h-12 w-20 rounded"
							/>
						{:else}
							<div
								class="flex h-12 w-20 items-center justify-center rounded bg-muted text-xs text-muted-foreground"
							>
								N/A
							</div>
						{/if}
					</Table.Cell>
					<Table.Cell class="px-4 py-2">
						<StatusBadge status={normalizeStatus(item.status)}>{normalizeStatus(item.status)}</StatusBadge>
					</Table.Cell>
					<Table.Cell class="px-4 py-2">
						<div>
							<a
								href="/shaders/{item.shaderSlug}"
								class="relative z-10 font-medium text-primary hover:underline"
							>
								{item.shaderName}
							</a>
							<div class="text-xs text-muted-foreground">
								{item.shaderVersion}
								{#if item.profileDisplayName}
									&middot; {item.profileDisplayName}
								{/if}
							</div>
						</div>
					</Table.Cell>
					<Table.Cell class="px-4 py-2">{item.sceneName}</Table.Cell>
					<Table.Cell class="px-4 py-2">{formatMs(item.durationMs)}</Table.Cell>
					<Table.Cell class="max-w-sm px-4 py-2" title={item.errorMessage ?? ''}>
						<span class="line-clamp-2">{item.errorMessage ?? '\u2014'}</span>
					</Table.Cell>
				</Table.Row>
				{#if expandedItem === item.id && item.status === 'FAILED'}
					<Table.Row>
						<Table.Cell colspan={6} class="bg-muted/30 px-4 py-3">
							<div class="space-y-2">
								{#if item.errorMessage}
									<div>
										<span class="text-xs font-medium text-muted-foreground">Error:</span>
										<p class="text-sm">{item.errorMessage}</p>
									</div>
								{/if}
								{#if item.errorLog}
									<div>
										<span class="text-xs font-medium text-muted-foreground">Log:</span>
										<pre
											class="mt-1 max-h-64 overflow-auto rounded bg-muted p-3 text-xs">{item.errorLog}</pre>
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
