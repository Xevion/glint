<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import { page as pageStore } from '$app/state';
import type { CaptureWithContext } from '$lib/bindings';
import AdminTable from '$lib/components/AdminTable.svelte';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { Button } from '$lib/components/ui/button';
import { RefreshCw } from '@lucide/svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let captures = $derived(data.captures);
let totalCount = $derived(data.totalCount);
let currentPage = $derived(data.page);
let pageSize = $derived(data.pageSize);
let totalPages = $derived(Math.ceil(totalCount / pageSize));
let shaders = $derived(data.shaders);
let scenes = $derived(data.scenes);
let refreshing = $state(false);
let error = $derived(data.error);

const statusColors: Record<string, string> = {
	running: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300',
	completed: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300',
	partial: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300',
	failed: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300'
};

function navigateToPage(p: number) {
	const url = new URL(pageStore.url);
	url.searchParams.set('page', String(p));
	void goto(url.toString(), { keepFocus: true });
}

function changePageSize(size: number) {
	const url = new URL(pageStore.url);
	url.searchParams.set('page_size', String(size));
	url.searchParams.set('page', '1');
	void goto(url.toString(), { keepFocus: true });
}

function setFilter(key: string, value: string) {
	const url = new URL(pageStore.url);
	if (value) {
		url.searchParams.set(key, value);
	} else {
		url.searchParams.delete(key);
	}
	url.searchParams.set('page', '1');
	void goto(url.toString(), { keepFocus: true });
}

const columns = [
	{ id: 'preview', key: 'image_url', name: 'Preview' },
	{ id: 'shader', key: 'shader_name', name: 'Shader' },
	{ id: 'scene', key: 'scene_name', name: 'Scene' },
	{ id: 'profile', key: 'profile', name: 'Profile' },
	{ id: 'resolution', key: 'resolution_width', name: 'Resolution' },
	{ id: 'captured_at', key: 'captured_at', name: 'Captured', component: 'time' as const },
	{ id: 'run', key: 'run_id', name: 'Run' }
];

async function refresh() {
	refreshing = true;
	await invalidateAll();
	refreshing = false;
}
</script>

<div class="space-y-4">
	<header class="flex items-center justify-between">
		<div class="flex items-baseline gap-3">
			<h1 class="text-2xl font-semibold">Captures</h1>
			<span class="text-lg text-muted-foreground">{totalCount}</span>
		</div>
		<Button variant="outline" size="icon" onclick={refresh} disabled={refreshing}>
			<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
		</Button>
	</header>

	<!-- Filter bar -->
	<div class="flex flex-wrap items-center gap-3">
		<select
			class="rounded border bg-background px-2 py-1.5 text-sm"
			value={data.filters.shader ?? ''}
			onchange={(e) => setFilter('shader', e.currentTarget.value)}
		>
			<option value="">All shaders</option>
			{#each shaders as s (s.id)}
				<option value={s.slug}>{s.name}</option>
			{/each}
		</select>

		<select
			class="rounded border bg-background px-2 py-1.5 text-sm"
			value={data.filters.scene ?? ''}
			onchange={(e) => setFilter('scene', e.currentTarget.value)}
		>
			<option value="">All scenes</option>
			{#each scenes as s (s.id)}
				<option value={s.id}>{s.name}</option>
			{/each}
		</select>

		<select
			class="rounded border bg-background px-2 py-1.5 text-sm"
			value={data.filters.status ?? ''}
			onchange={(e) => setFilter('status', e.currentTarget.value)}
		>
			<option value="">All statuses</option>
			<option value="completed">Completed</option>
			<option value="failed">Failed</option>
		</select>

		{#if data.filters.runId}
			<span class="inline-flex items-center gap-1 rounded-full bg-muted px-2.5 py-1 text-xs">
				Run: {data.filters.runId.slice(0, 8)}...
				<button class="hover:text-foreground" onclick={() => setFilter('run_id', '')}>x</button
				>
			</span>
		{/if}
	</div>

	{#if error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
			Error: {error}
		</div>
	{:else if captures.length === 0}
		<p class="text-muted-foreground">No captures yet.</p>
	{:else}
		<AdminTable
			data={captures}
			{columns}
			onRowClick={(capture: CaptureWithContext) => goto(`/admin/captures/${capture.id}`)}
			getRowId={(c: CaptureWithContext) => c.id}
		>
			{#snippet cell({ columnId, value, row }: { columnId: string; value: unknown; row: CaptureWithContext })}
				{#if columnId === 'preview'}
					{#if value}
						<CaptureImage
							src={value as string}
							thumbhash={row.thumbhash}
							preset="thumbnail"
							alt="Capture preview"
							class="h-full w-full object-cover"
							containerClass="h-12 w-20 rounded"
						/>
					{:else if row.image_path}
						<div
							class="flex h-12 w-20 items-center justify-center rounded bg-muted text-xs text-muted-foreground"
						>
							No URL
						</div>
					{:else}
						<div
							class="flex h-12 w-20 items-center justify-center rounded bg-muted text-xs text-muted-foreground"
						>
							N/A
						</div>
					{/if}
				{:else if columnId === 'shader'}
					<div>
						<a
							href="/shaders/{row.shader_slug}"
							class="font-medium text-primary hover:underline"
							onclick={(e) => e.stopPropagation()}>{row.shader_name}</a
						>
						<div class="text-xs text-muted-foreground">{row.shader_version}</div>
					</div>
				{:else if columnId === 'scene'}
					{value ?? row.scene_id}
				{:else if columnId === 'profile'}
					{value ?? '-'}
				{:else if columnId === 'resolution'}
					{row.resolution_width && row.resolution_height
						? `${row.resolution_width}x${row.resolution_height}`
						: '-'}
				{:else if columnId === 'run'}
					{#if row.run_id}
						<a
							href="/admin/runs/{row.run_id}"
							class="inline-flex items-center gap-1"
							onclick={(e) => e.stopPropagation()}
						>
							<span
								class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {statusColors[
									row.run_status ?? ''
								] ?? 'bg-gray-100 text-gray-800'}"
							>
								{row.run_status ?? '?'}
							</span>
						</a>
					{:else}
						<span class="text-muted-foreground">&mdash;</span>
					{/if}
				{:else}
					{value ?? '-'}
				{/if}
			{/snippet}
		</AdminTable>
	{/if}

	{#if totalPages > 1}
		<div class="flex items-center justify-between border-t pt-4">
			<div class="text-sm text-muted-foreground">
				Showing {(currentPage - 1) * pageSize + 1}&ndash;{Math.min(
					currentPage * pageSize,
					totalCount
				)} of {totalCount}
			</div>
			<div class="flex items-center gap-2">
				<Button
					variant="outline"
					size="sm"
					disabled={currentPage <= 1}
					onclick={() => navigateToPage(currentPage - 1)}
				>
					Previous
				</Button>
				<span class="text-sm">Page {currentPage} of {totalPages}</span>
				<Button
					variant="outline"
					size="sm"
					disabled={currentPage >= totalPages}
					onclick={() => navigateToPage(currentPage + 1)}
				>
					Next
				</Button>
			</div>
			<select
				class="rounded border bg-background px-2 py-1 text-sm"
				value={pageSize}
				onchange={(e) => changePageSize(Number(e.currentTarget.value))}
			>
				<option value={25}>25 per page</option>
				<option value={50}>50 per page</option>
				<option value={100}>100 per page</option>
			</select>
		</div>
	{/if}
</div>
