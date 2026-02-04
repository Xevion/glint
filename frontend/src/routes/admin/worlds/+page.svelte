<script lang="ts">
import { onMount } from 'svelte';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import { Textarea } from '$lib/components/ui/textarea';
import { RefreshCw, Trash2 } from '@lucide/svelte';
import AdminTable from '$lib/components/admin-table.svelte';
import { AdminSlideOver, AdminDetailField } from '$lib/components/admin';
import WorldUploadDialog from '$lib/components/world-upload-dialog.svelte';
import TimeAgo from '$lib/components/time-ago.svelte';
import { api } from '$lib/api';
import type { World } from '$lib/bindings';
import type { UpdateWorldRequest } from '$lib/api/endpoints/admin';
import { escapeHtml, formatBytes } from '$lib/utils/display';

let worlds = $state<World[]>([]);
let selected = $state<World | null>(null);
let loading = $state(true);
let refreshing = $state(false);
let saving = $state(false);
let error = $state<string | null>(null);
let showDeleteConfirm = $state(false);

// Edit form state
let editName = $state('');
let editDescription = $state('');

const columns = [
	{
		id: 'name',
		key: 'name',
		name: 'Name',
		render: (value: string) => `<span class="font-medium">${escapeHtml(value)}</span>`
	},
	{ id: 'slug', key: 'slug', name: 'Slug' },
	{
		id: 'minecraft_version',
		key: 'minecraft_version',
		name: 'MC Version',
		render: (v: string) =>
			`<code class="rounded bg-muted px-1.5 py-0.5 text-xs">${escapeHtml(v)}</code>`
	},
	{
		id: 'size',
		key: 'size_bytes',
		name: 'Size',
		render: (bytes: number | null) => (bytes ? formatBytes(bytes) : '-')
	},
	{
		id: 'created_at',
		key: 'created_at',
		name: 'Created',
		component: 'time' as const
	}
];

async function load() {
	refreshing = true;
	error = null;
	const result = await api.worlds.listWorlds();
	if (result.isOk) {
		worlds = result.value;
	} else {
		error = result.error.message;
	}
	loading = false;
	refreshing = false;
}

function openWorld(world: World) {
	selected = world;
	editName = world.name;
	editDescription = world.description ?? '';
}

async function handleSave() {
	if (!selected) return;

	saving = true;
	error = null;

	const request: UpdateWorldRequest = {};
	if (editName !== selected.name) request.name = editName;
	if (editDescription !== (selected.description ?? ''))
		request.description = editDescription || undefined;

	if (Object.keys(request).length === 0) {
		saving = false;
		return;
	}

	const result = await api.admin.updateWorld(selected.id, request);
	if (result.isOk) {
		selected = result.value;
		worlds = worlds.map((w) => (w.id === selected!.id ? result.value : w));
	} else {
		error = result.error.message;
	}
	saving = false;
}

function handleDelete() {
	if (!selected) return;
	showDeleteConfirm = true;
}

async function confirmDelete() {
	if (!selected) return;

	const result = await api.admin.deleteWorld(selected.id);
	if (result.isOk) {
		selected = null;
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
			<h1 class="text-2xl font-semibold">Worlds</h1>
			{#if !loading}
				<span class="text-lg text-muted-foreground">{worlds.length}</span>
			{/if}
		</div>
		<div class="flex items-center gap-2">
			<Button variant="outline" size="icon" onclick={load} disabled={refreshing}>
				<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
			</Button>
			<WorldUploadDialog onWorldCreated={load} />
		</div>
	</header>

	{#if loading}
		<div class="text-center text-muted-foreground">Loading...</div>
	{:else if error && !selected}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
			Error: {error}
		</div>
	{:else if worlds.length === 0}
		<p class="text-muted-foreground">No worlds uploaded yet.</p>
	{:else}
		<AdminTable
			data={worlds}
			{columns}
			selectedId={selected?.id}
			onRowClick={openWorld}
			getRowId={(w: World) => w.id}
		/>
	{/if}
</div>

<AdminSlideOver
	open={selected !== null}
	title={selected?.name ?? ''}
	description={selected?.slug}
	onClose={() => (selected = null)}
	width="wide"
>
	{#if selected}
		{#if error}
			<div
				class="mb-4 rounded-lg border border-destructive bg-destructive/10 p-3 text-sm text-destructive"
			>
				{error}
			</div>
		{/if}

		<div class="space-y-4">
			<div class="grid gap-2">
				<Label for="name">Name</Label>
				<Input id="name" bind:value={editName} />
			</div>

			<div class="grid gap-2">
				<Label for="description">Description</Label>
				<Textarea id="description" bind:value={editDescription} rows={3} />
			</div>

			<div class="border-t pt-4">
				<dl class="space-y-2 text-sm">
					<AdminDetailField label="ID">
						<code class="text-xs">{selected.id}</code>
					</AdminDetailField>
					<AdminDetailField label="Slug">
						{selected.slug}
					</AdminDetailField>
					<AdminDetailField label="Minecraft Version">
						<code class="rounded bg-muted px-1.5 py-0.5 text-xs">
							{selected.minecraft_version}
						</code>
					</AdminDetailField>
					<AdminDetailField label="Size">
						{selected.size_bytes ? formatBytes(selected.size_bytes) : '-'}
					</AdminDetailField>
					{#if selected.file_url}
						<AdminDetailField label="File URL">
							<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
							<a
								href={selected.file_url}
								class="text-xs text-primary hover:underline"
								target="_blank"
							>
								{selected.file_url}
							</a>
						</AdminDetailField>
					{/if}
					{#if selected.file_hash}
						<AdminDetailField label="File Hash">
							<code class="text-xs">{selected.file_hash}</code>
						</AdminDetailField>
					{/if}
					<AdminDetailField label="Created">
						<TimeAgo timestamp={selected.created_at} />
					</AdminDetailField>
					<AdminDetailField label="Updated">
						<TimeAgo timestamp={selected.updated_at} />
					</AdminDetailField>
				</dl>
			</div>
		</div>
	{/if}

	{#snippet footer()}
		<div class="flex justify-end">
			<div class="inline-flex">
				<Button
					variant="destructive"
					onclick={handleDelete}
					class="rounded-r-none border-r-0"
					size="icon"
				>
					<Trash2 class="h-4 w-4" />
					<span class="sr-only">Delete</span>
				</Button>
				<Button onclick={handleSave} disabled={saving} class="rounded-l-none">
					{saving ? 'Saving...' : 'Save Changes'}
				</Button>
			</div>
		</div>
	{/snippet}
</AdminSlideOver>

<ConfirmDialog
	bind:open={showDeleteConfirm}
	title="Delete World"
	description={`Delete world "${selected?.name}"? This will also delete the file from storage.`}
	confirmLabel="Delete"
	onConfirm={confirmDelete}
/>
