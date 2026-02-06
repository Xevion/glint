<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import { page as pageStore } from '$app/state';
import { api } from '$lib/api';
import type { CaptureWithContext } from '$lib/bindings';
import AdminTable from '$lib/components/AdminTable.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { AdminDetailField, AdminSlideOver } from '$lib/components/admin';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import { cfImageUrl } from '$lib/utils/image';
import { RefreshCw, Trash2 } from '@lucide/svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let captures = $derived(data.captures);
let totalCount = $derived(data.totalCount);
let currentPage = $derived(data.page);
let pageSize = $derived(data.pageSize);
let totalPages = $derived(Math.ceil(totalCount / pageSize));
let shaders = $derived(data.shaders);
let scenes = $derived(data.scenes);
let selected = $state<CaptureWithContext | null>(null);
let refreshing = $state(false);
let error = $state<string | null>(null);
let showDeleteConfirm = $state(false);
let captureToDelete = $state<CaptureWithContext | null>(null);

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
	{ id: 'scene_id', key: 'scene_id', name: 'Scene' },
	{ id: 'profile', key: 'profile', name: 'Profile' },
	{ id: 'resolution', key: 'resolution_width', name: 'Resolution' },
	{ id: 'captured_at', key: 'captured_at', name: 'Captured', component: 'time' as const },
	{ id: 'run', key: 'run_id', name: 'Run' }
];

async function refresh() {
	refreshing = true;
	error = null;
	await invalidateAll();
	refreshing = false;
}

function handleDelete(capture: CaptureWithContext) {
	captureToDelete = capture;
	showDeleteConfirm = true;
}

async function confirmDelete() {
	if (!captureToDelete) return;
	const result = await api.admin.deleteCapture(captureToDelete.id);
	result.match({
		Ok: () => {
			selected = null;
			captureToDelete = null;
			void refresh();
		},
		Err: (err) => {
			error = err.message;
		}
	});
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
				<button class="hover:text-foreground" onclick={() => setFilter('run_id', '')}>x</button>
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
			selectedId={selected?.id}
			onRowClick={(capture: CaptureWithContext) => (selected = capture)}
			getRowId={(c: CaptureWithContext) => c.id}
		>
			{#snippet cell({ columnId, value, row }: { columnId: string; value: unknown; row: CaptureWithContext })}
				{#if columnId === 'preview'}
					{#if value}
						<img src={cfImageUrl(value as string, 'thumbnail')} alt="Capture preview" class="h-12 w-20 rounded object-cover" />
					{:else if row.image_path}
						<div class="flex h-12 w-20 items-center justify-center rounded bg-muted text-xs text-muted-foreground">No URL</div>
					{:else}
						<div class="flex h-12 w-20 items-center justify-center rounded bg-muted text-xs text-muted-foreground">N/A</div>
					{/if}
				{:else if columnId === 'shader'}
					<div>
						<a href="/shaders/{row.shader_slug}" class="font-medium text-primary hover:underline">{row.shader_name}</a>
						<div class="text-xs text-muted-foreground">{row.shader_version}</div>
					</div>
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
						<span class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {statusColors[row.run_status ?? ''] ?? 'bg-gray-100 text-gray-800'}">
							{row.run_status ?? '?'}
						</span>
					</a>
				{:else}
					<span class="text-muted-foreground">—</span>
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
				Showing {(currentPage - 1) * pageSize + 1}–{Math.min(currentPage * pageSize, totalCount)} of {totalCount}
			</div>
			<div class="flex items-center gap-2">
				<Button variant="outline" size="sm" disabled={currentPage <= 1} onclick={() => navigateToPage(currentPage - 1)}>
					Previous
				</Button>
				<span class="text-sm">Page {currentPage} of {totalPages}</span>
				<Button variant="outline" size="sm" disabled={currentPage >= totalPages} onclick={() => navigateToPage(currentPage + 1)}>
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

<AdminSlideOver
	open={selected !== null}
	title={selected ? `${selected.shader_name} - ${selected.shader_version}` : ''}
	description={selected?.profile ? `Profile: ${selected.profile}` : undefined}
	onClose={() => (selected = null)}
>
	{#if selected}
		<dl class="space-y-4">
			{#if selected.image_url}
				<div>
					<img
						src={cfImageUrl(selected.image_url, 'hero')}
						alt="Capture"
						class="w-full rounded-lg border"
					/>
				</div>
			{/if}

			<AdminDetailField label="Capture ID">
				<code class="text-xs">{selected.id}</code>
			</AdminDetailField>

			<AdminDetailField label="Shader">
				<a href="/shaders/{selected.shader_slug}" class="text-primary hover:underline">
					{selected.shader_name}
				</a>
				<span class="text-muted-foreground"> ({selected.shader_version})</span>
			</AdminDetailField>

			<AdminDetailField label="Scene ID">
				{selected.scene_id}
			</AdminDetailField>

			{#if selected.profile}
				<AdminDetailField label="Profile">
					{selected.profile}
				</AdminDetailField>
			{/if}

			<AdminDetailField label="Resolution">
				{selected.resolution_width && selected.resolution_height
					? `${selected.resolution_width}x${selected.resolution_height}`
					: '-'}
			</AdminDetailField>

			<AdminDetailField label="Captured">
				{#if selected.captured_at}
					<TimeAgo timestamp={selected.captured_at} />
				{:else}
					-
				{/if}
			</AdminDetailField>

			{#if selected?.run_id}
				<AdminDetailField label="Run">
					<a href="/admin/runs/{selected.run_id}" class="text-primary hover:underline">
						{selected.run_id.slice(0, 12)}...
					</a>
					{#if selected.run_status}
						<span class="ml-2 inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {statusColors[selected.run_status] ?? ''}">
							{selected.run_status}
						</span>
					{/if}
				</AdminDetailField>
			{/if}
		</dl>
	{/if}

	{#snippet footer()}
		<div class="flex justify-end gap-2">
			<Button variant="destructive" onclick={() => selected && handleDelete(selected)}>
				<Trash2 class="mr-2 h-4 w-4" />
				Delete
			</Button>
		</div>
	{/snippet}
</AdminSlideOver>

<ConfirmDialog
	bind:open={showDeleteConfirm}
	title="Delete Capture"
	description={`Delete capture for ${captureToDelete?.shader_name}?`}
	confirmLabel="Delete"
	onConfirm={confirmDelete}
/>
