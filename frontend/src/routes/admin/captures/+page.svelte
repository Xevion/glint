<script lang="ts">
import { invalidateAll } from '$app/navigation';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import { RefreshCw, Trash2 } from '@lucide/svelte';
import AdminTable from '$lib/components/AdminTable.svelte';
import { AdminSlideOver, AdminDetailField } from '$lib/components/admin';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { api } from '$lib/api';
import type { CaptureWithContext } from '$lib/bindings';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let captures = $derived(data.captures);
let selected = $state<CaptureWithContext | null>(null);
let refreshing = $state(false);
let error = $state<string | null>(null);
let showDeleteConfirm = $state(false);
let captureToDelete = $state<CaptureWithContext | null>(null);

const columns = [
	{ id: 'preview', key: 'screenshot_url', name: 'Preview' },
	{ id: 'shader', key: 'shader_name', name: 'Shader' },
	{ id: 'scene_id', key: 'scene_id', name: 'Scene' },
	{ id: 'profile', key: 'profile', name: 'Profile' },
	{ id: 'resolution', key: 'resolution_width', name: 'Resolution' },
	{ id: 'captured_at', key: 'captured_at', name: 'Captured', component: 'time' as const }
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
			<span class="text-lg text-muted-foreground">{captures.length}</span>
		</div>
		<Button variant="outline" size="icon" onclick={refresh} disabled={refreshing}>
			<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
		</Button>
	</header>

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
						<img src={value as string} alt="Capture preview" class="h-12 w-20 rounded object-cover" />
					{:else if row.screenshot_path}
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
				{:else}
					{value ?? '-'}
				{/if}
			{/snippet}
		</AdminTable>
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
			{#if selected.screenshot_url}
				<div>
					<img
						src={selected.screenshot_url}
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
