<script lang="ts">
import { onMount } from 'svelte';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import { RefreshCw, Trash2 } from '@lucide/svelte';
import AdminTable from '$lib/components/admin-table.svelte';
import { AdminSlideOver, AdminDetailField } from '$lib/components/admin';
import TimeAgo from '$lib/components/time-ago.svelte';
import { api } from '$lib/api';
import type { CaptureWithContext } from '$lib/bindings';
import { escapeHtml } from '$lib/utils/display';

let captures = $state<CaptureWithContext[]>([]);
let selected = $state<CaptureWithContext | null>(null);
let loading = $state(true);
let refreshing = $state(false);
let error = $state<string | null>(null);
let showDeleteConfirm = $state(false);
let captureToDelete = $state<CaptureWithContext | null>(null);

const columns = [
	{
		id: 'preview',
		key: 'screenshot_url',
		name: 'Preview',
		render: (value: string | null, row: CaptureWithContext) => {
			if (value) {
				const safeUrl = escapeHtml(value);
				return `<img src="${safeUrl}" alt="Capture preview" class="h-12 w-20 rounded object-cover" />`;
			} else if (row.screenshot_path) {
				return '<div class="flex h-12 w-20 items-center justify-center rounded bg-muted text-xs text-muted-foreground">No URL</div>';
			}
			return '<div class="flex h-12 w-20 items-center justify-center rounded bg-muted text-xs text-muted-foreground">N/A</div>';
		}
	},
	{
		id: 'shader',
		key: 'shader_name',
		name: 'Shader',
		render: (_value: unknown, row: CaptureWithContext) => {
			const safeName = escapeHtml(row.shader_name);
			const safeSlug = escapeHtml(row.shader_slug);
			const safeVersion = escapeHtml(row.shader_version);
			return `<div><a href="/shaders/${safeSlug}" class="font-medium text-primary hover:underline">${safeName}</a><div class="text-xs text-muted-foreground">${safeVersion}</div></div>`;
		}
	},
	{ id: 'scene_id', key: 'scene_id', name: 'Scene' },
	{
		id: 'profile',
		key: 'profile',
		name: 'Profile',
		render: (value: string | null) => value ?? '-'
	},
	{
		id: 'resolution',
		key: 'resolution_width',
		name: 'Resolution',
		render: (_value: unknown, row: CaptureWithContext) =>
			row.resolution_width && row.resolution_height
				? `${row.resolution_width}x${row.resolution_height}`
				: '-'
	},
	{
		id: 'captured_at',
		key: 'captured_at',
		name: 'Captured',
		component: 'time' as const
	}
];

async function load() {
	refreshing = true;
	error = null;
	const result = await api.admin.listCaptures();
	if (result.isOk) {
		captures = result.value;
	} else {
		error = result.error.message;
	}
	loading = false;
	refreshing = false;
}

function handleDelete(capture: CaptureWithContext) {
	captureToDelete = capture;
	showDeleteConfirm = true;
}

async function confirmDelete() {
	if (!captureToDelete) return;
	const result = await api.admin.deleteCapture(captureToDelete.id);
	if (result.isOk) {
		selected = null;
		captureToDelete = null;
		void load();
	} else {
		error = result.error.message;
	}
}

onMount(() => {
	void load();
});
</script>

<div class="space-y-4">
	<header class="flex items-center justify-between">
		<div class="flex items-baseline gap-3">
			<h1 class="text-2xl font-semibold">Captures</h1>
			{#if !loading}
				<span class="text-lg text-muted-foreground">{captures.length}</span>
			{/if}
		</div>
		<Button variant="outline" size="icon" onclick={load} disabled={refreshing}>
			<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
		</Button>
	</header>

	{#if loading}
		<div class="text-center text-muted-foreground">Loading...</div>
	{:else if error}
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
		/>
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
