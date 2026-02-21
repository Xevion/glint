<script lang="ts">
import { goto, invalidate } from '$app/navigation';
import { page } from '$app/state';
import Breadcrumb from '$lib/components/Breadcrumb.svelte';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import RefreshButton from '$lib/components/RefreshButton.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import {
	createClientList,
	DataView,
	Grid,
	Search,
	Toolbar,
	ViewToggle
} from '$lib/components/data-view';
import { PopoverMultiSelect } from '$lib/components/filters';
import * as Dialog from '$lib/components/ui/dialog';
import { StatusBadge, type StatusBadgeStatus } from '$lib/components/ui/status-badge';
import * as TableUI from '$lib/components/ui/table';
import { formatDuration, formatElapsedDuration } from '$lib/utils/format';
import type { PageData } from './$types';
import type { AdminCaptureRunData, AdminCaptureRunItem } from './queries';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let run: AdminCaptureRunData = $derived(data.run);
let items: AdminCaptureRunItem[] = $derived(run.items);
let refreshing = $state(false);
let errorItem = $state<AdminCaptureRunItem | null>(null);
let statusFilter = $state<string[]>([]);

const statusOptions = [
	{ value: 'PENDING', label: 'Pending' },
	{ value: 'RUNNING', label: 'Running' },
	{ value: 'COMPLETED', label: 'Completed' },
	{ value: 'FAILED', label: 'Failed' },
	{ value: 'SKIPPED', label: 'Skipped' }
];

const STATUS_COLORS: Record<string, string> = {
	PENDING: 'text-muted-foreground',
	RUNNING: 'text-info',
	COMPLETED: 'text-success',
	FAILED: 'text-destructive',
	SKIPPED: 'text-muted-foreground'
};

function normalizeStatus(status: string): StatusBadgeStatus {
	return status.toLowerCase().replace(/_/g, '_') as NonNullable<StatusBadgeStatus>;
}

const list = createClientList<AdminCaptureRunItem>({
	key: (i) => i.id,
	items: () => {
		if (statusFilter.length === 0) return items;
		return items.filter((i) => statusFilter.includes(i.status));
	},
	search: {
		clientFilter: (item, q) =>
			item.shaderName.toLowerCase().includes(q) || item.sceneName.toLowerCase().includes(q)
	},
	viewMode: 'table'
});

let remaining = $derived(run.totalItems - run.completedItems - run.failedItems - run.skippedItems);
let isTimedOut = $derived(run.status === 'TIMED_OUT');

let uniqueShaders = $derived(new Set(items.map((i) => i.shaderName)).size);
let uniqueScenes = $derived(new Set(items.map((i) => i.sceneName)).size);
let successRate = $derived(
	run.totalItems > 0 ? Math.round((run.completedItems / run.totalItems) * 100) : 0
);
let avgDuration = $derived.by(() => {
	const durations = items.filter((i) => i.durationMs != null).map((i) => i.durationMs!);
	if (durations.length === 0) return null;
	return durations.reduce((a, b) => a + b, 0) / durations.length;
});

async function refresh() {
	refreshing = true;
	await Promise.all([
		invalidate(`glint:admin:run:${page.params.id}`),
		new Promise((r) => setTimeout(r, 300))
	]);
	refreshing = false;
}

function handleRowClick(item: AdminCaptureRunItem) {
	if (item.captureId) {
		void goto(`/admin/captures/${item.captureId}`);
	} else if (item.status === 'FAILED') {
		errorItem = item;
	}
}
</script>

<svelte:head><title>Run Details - Glint</title></svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<header class="flex items-center gap-2">
		<Breadcrumb segments={[{ label: 'Capture Runs', href: '/admin/runs' }, { label: 'Run' }]}>
			{#snippet trailing()}
				<StatusBadge status={normalizeStatus(run.status)} class="ml-2"
					>{normalizeStatus(run.status)}</StatusBadge
				>
			{/snippet}
		</Breadcrumb>
		<div class="ml-auto">
			<RefreshButton {refreshing} onclick={refresh} />
		</div>
	</header>

	<!-- Info Panel -->
	<div class="rounded-lg border bg-card">
		<!-- Progress bar — full width -->
		{#if run.totalItems > 0}
			<div class="flex h-2 overflow-hidden rounded-t-lg bg-muted">
				{#if run.completedItems > 0}
					<div
						class="bg-success"
						style="width: {(run.completedItems / run.totalItems) * 100}%"
					></div>
				{/if}
				{#if run.failedItems > 0}
					<div
						class="bg-destructive"
						style="width: {(run.failedItems / run.totalItems) * 100}%"
					></div>
				{/if}
				{#if run.skippedItems > 0}
					<div
						class="bg-muted-foreground/30"
						style="width: {(run.skippedItems / run.totalItems) * 100}%"
					></div>
				{/if}
				{#if remaining > 0}
					<div
						class={isTimedOut ? 'bg-warning' : 'bg-info'}
						style="width: {(remaining / run.totalItems) * 100}%"
					></div>
				{/if}
			</div>
		{/if}

		<!-- Status pills + metadata -->
		<div class="p-4">
			<!-- Status count pills -->
			<div class="flex flex-wrap gap-2">
				{#if run.completedItems > 0}
					<span
						class="inline-flex items-center gap-1.5 rounded-md bg-muted px-2 py-0.5 text-xs font-medium"
					>
						<span class="h-1.5 w-1.5 rounded-full bg-success"></span>
						{run.completedItems} completed
					</span>
				{/if}
				{#if run.failedItems > 0}
					<span
						class="inline-flex items-center gap-1.5 rounded-md bg-muted px-2 py-0.5 text-xs font-medium"
					>
						<span class="h-1.5 w-1.5 rounded-full bg-destructive"></span>
						{run.failedItems} failed
					</span>
				{/if}
				{#if run.skippedItems > 0}
					<span
						class="inline-flex items-center gap-1.5 rounded-md bg-muted px-2 py-0.5 text-xs font-medium"
					>
						<span class="h-1.5 w-1.5 rounded-full bg-muted-foreground/40"></span>
						{run.skippedItems} skipped
					</span>
				{/if}
				{#if remaining > 0}
					<span
						class="inline-flex items-center gap-1.5 rounded-md bg-muted px-2 py-0.5 text-xs font-medium"
					>
						<span
							class="h-1.5 w-1.5 rounded-full {isTimedOut ? 'bg-warning' : 'bg-info'}"
						></span>
						{remaining}
						{isTimedOut ? 'timed out' : 'pending'}
					</span>
				{/if}
			</div>

			<!-- Two-column metadata grid -->
			<div class="mt-4 grid gap-x-8 gap-y-4 sm:grid-cols-2">
				<!-- Left column -->
				<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
					<dt class="text-muted-foreground">ID</dt>
					<dd><code class="text-xs">{run.id}</code></dd>

					<dt class="text-muted-foreground">Agent</dt>
					<dd>{run.agentId ?? '\u2014'}</dd>

					<dt class="text-muted-foreground">Started</dt>
					<dd><TimeAgo timestamp={run.startedAt} /></dd>

					<dt class="text-muted-foreground">Duration</dt>
					<dd>
						{formatElapsedDuration({
							started_at: run.startedAt,
							completed_at: run.completedAt
						})}
					</dd>
				</dl>

				<!-- Right column -->
				<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
					<dt class="text-muted-foreground">Resolution</dt>
					<dd>{run.resolutionWidth}&times;{run.resolutionHeight}</dd>

					<dt class="text-muted-foreground">Format</dt>
					<dd>{run.imageFormat}</dd>

					<dt class="text-muted-foreground">Shaders</dt>
					<dd>{uniqueShaders}</dd>

					<dt class="text-muted-foreground">Scenes</dt>
					<dd>{uniqueScenes}</dd>

					<dt class="text-muted-foreground">Success rate</dt>
					<dd>{successRate}%</dd>

					{#if avgDuration != null}
						<dt class="text-muted-foreground">Avg capture</dt>
						<dd>{formatDuration(avgDuration)}</dd>
					{/if}
				</dl>
			</div>
		</div>
	</div>

	<!-- Items -->
	<Toolbar>
		<PopoverMultiSelect label="Status" options={statusOptions} bind:value={statusFilter} />
		<Search bind:value={list.search} placeholder="Search shaders or scenes..." />
		<div class="ml-auto">
			<ViewToggle modes={['table', 'grid']} bind:mode={list.viewMode} />
		</div>
	</Toolbar>

	<DataView list={list}>
		<!-- Table view -->
		{#if list.viewMode === 'table'}
			<div class="rounded-md border">
				<TableUI.Root>
					<TableUI.Header>
						<TableUI.Row>
							<TableUI.Head class="w-20">Preview</TableUI.Head>
							<TableUI.Head>Shader</TableUI.Head>
							<TableUI.Head>Scene</TableUI.Head>
							<TableUI.Head class="w-[100px]">Status</TableUI.Head>
							<TableUI.Head>Error</TableUI.Head>
						</TableUI.Row>
					</TableUI.Header>
					<TableUI.Body>
						{#each list.items as item (item.id)}
							<TableUI.Row
								class="cursor-pointer hover:bg-muted/50"
								onclick={() => handleRowClick(item)}
							>
								<TableUI.Cell>
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
								</TableUI.Cell>
								<TableUI.Cell>
									<div>
										<span class="font-medium">{item.shaderName}</span>
										<div class="text-xs text-muted-foreground">
											{item.shaderVersion}
											{#if item.profileDisplayName}
												&middot; {item.profileDisplayName}
											{/if}
										</div>
									</div>
								</TableUI.Cell>
								<TableUI.Cell>{item.sceneName}</TableUI.Cell>
								<TableUI.Cell>
									<span class={STATUS_COLORS[item.status] ?? 'text-foreground'}>
										{item.status.toLowerCase()}
									</span>
								</TableUI.Cell>
								<TableUI.Cell class="max-w-sm" title={item.errorMessage ?? ''}>
									<span class="line-clamp-2"
										>{item.errorMessage ?? '\u2014'}</span
									>
								</TableUI.Cell>
							</TableUI.Row>
						{/each}
						{#if list.items.length === 0}
							<TableUI.Row>
								<TableUI.Cell colspan={5} class="h-24 text-center">
									No items found.
								</TableUI.Cell>
							</TableUI.Row>
						{/if}
					</TableUI.Body>
				</TableUI.Root>
			</div>
		{/if}

		<!-- Grid view -->
		<Grid size="small">
			{#snippet card(item: AdminCaptureRunItem)}
				<svelte:element
					this={item.captureId ? 'a' : 'div'}
					href={item.captureId ? `/admin/captures/${item.captureId}` : undefined}
					class="block rounded-lg border bg-card transition-colors hover:bg-muted/50 {item.status === 'FAILED' && !item.captureId ? 'cursor-pointer border-l-2 border-l-destructive' : ''}"
					onclick={item.status === 'FAILED' && !item.captureId
						? () => {
								errorItem = item;
							}
						: undefined}
					role={item.status === 'FAILED' && !item.captureId ? 'button' : undefined}
				>
					<div class="overflow-hidden rounded-t-lg">
						{#if item.imagePath ?? item.thumbhash}
							<CaptureImage
								src={item.imagePath}
								thumbhash={item.thumbhash}
								preset="card"
								alt="{item.shaderName} capture"
								class="w-full"
								containerClass="aspect-video w-full"
							/>
						{:else}
							<div
								class="flex aspect-video w-full items-center justify-center bg-muted text-xs text-muted-foreground"
							>
								No image
							</div>
						{/if}
					</div>
					<div class="p-2">
						<div class="text-sm font-medium">
							{item.shaderName}
							<span class="font-normal text-muted-foreground">
								{item.shaderVersion}
							</span>
						</div>
						<div class="text-xs text-muted-foreground">
							{item.sceneName}
							&middot;
							<span class={STATUS_COLORS[item.status] ?? ''}>
								{item.status.toLowerCase()}
							</span>
						</div>
					</div>
				</svelte:element>
			{/snippet}
		</Grid>
	</DataView>
</div>

<!-- Error Detail Dialog -->
<Dialog.Root
	open={errorItem !== null}
	onOpenChange={(open) => {
		if (!open) errorItem = null;
	}}
>
	<Dialog.Content class="sm:max-w-lg">
		{#if errorItem}
			<Dialog.Header>
				<Dialog.Title>Capture Failed &mdash; {errorItem.shaderName}</Dialog.Title>
				{#if errorItem.sceneName}
					<Dialog.Description>{errorItem.sceneName}</Dialog.Description>
				{/if}
			</Dialog.Header>
			<div class="space-y-3">
				{#if errorItem.errorMessage}
					<div>
						<div class="text-xs font-medium text-muted-foreground">Error</div>
						<p class="mt-1 text-sm">{errorItem.errorMessage}</p>
					</div>
				{/if}
				{#if errorItem.errorLog}
					<div>
						<div class="text-xs font-medium text-muted-foreground">Log</div>
						<pre
							class="mt-1 max-h-64 overflow-auto rounded bg-muted p-3 text-xs">{errorItem.errorLog}</pre>
					</div>
				{/if}
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
