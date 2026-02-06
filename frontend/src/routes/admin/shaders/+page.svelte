<script lang="ts">
import { invalidateAll } from '$app/navigation';
import { api } from '$lib/api';
import type { UpdateShaderRequest } from '$lib/api/endpoints/admin';
import type { Shader } from '$lib/bindings';
import AdminTable from '$lib/components/AdminTable.svelte';
import AdoptShaderDialog from '$lib/components/AdoptShaderDialog.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { AdminDetailField, AdminSlideOver } from '$lib/components/admin';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import { Textarea } from '$lib/components/ui/textarea';
import { RefreshCw, Trash2 } from '@lucide/svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let shaders = $derived(data.shaders);
let selected = $state<Shader | null>(null);
let refreshing = $state(false);
let saving = $state(false);
let error = $state<string | null>(null);
let showDeleteConfirm = $state(false);

// Edit form state
let editName = $state('');
let editDescription = $state('');
let editModrinthId = $state('');
let editCurseforgeId = $state('');
let editWebsiteUrl = $state('');

const columns = [
	{ id: 'name', key: 'name', name: 'Name' },
	{ id: 'slug', key: 'slug', name: 'Slug' },
	{ id: 'description', key: 'description', name: 'Description' },
	{ id: 'created_at', key: 'created_at', name: 'Created', component: 'time' as const }
];

async function refresh() {
	refreshing = true;
	error = null;
	await invalidateAll();
	refreshing = false;
}

function openShader(shader: Shader) {
	selected = shader;
	// Initialize edit form
	editName = shader.name;
	editDescription = shader.description ?? '';
	editModrinthId = shader.modrinth_id ?? '';
	editCurseforgeId = shader.curseforge_id ?? '';
	editWebsiteUrl = shader.website_url ?? '';
}

async function handleSave() {
	if (!selected) return;

	saving = true;
	error = null;

	const request: UpdateShaderRequest = {};
	if (editName !== selected.name) request.name = editName;
	if (editDescription !== (selected.description ?? ''))
		request.description = editDescription || undefined;
	if (editModrinthId !== (selected.modrinth_id ?? ''))
		request.modrinth_id = editModrinthId || undefined;
	if (editCurseforgeId !== (selected.curseforge_id ?? ''))
		request.curseforge_id = editCurseforgeId || undefined;
	if (editWebsiteUrl !== (selected.website_url ?? ''))
		request.website_url = editWebsiteUrl || undefined;

	// Only update if there are changes
	if (Object.keys(request).length === 0) {
		saving = false;
		return;
	}

	const result = await api.admin.updateShader(selected.id, request);
	result.match({
		Ok: (value) => {
			selected = value;
			void refresh();
		},
		Err: (err) => {
			error = err.message;
		}
	});
	saving = false;
}

function handleDelete() {
	if (!selected) return;
	showDeleteConfirm = true;
}

async function confirmDelete() {
	if (!selected) return;

	const result = await api.admin.deleteShader(selected.id);
	result.match({
		Ok: () => {
			selected = null;
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
			<h1 class="text-2xl font-semibold">Shaders</h1>
			<span class="text-lg text-muted-foreground">{shaders.length}</span>
		</div>
		<div class="flex items-center gap-2">
			<AdoptShaderDialog onShaderAdopted={refresh} />
			<Button variant="outline" size="icon" onclick={refresh} disabled={refreshing}>
				<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
			</Button>
		</div>
	</header>

	{#if error && !selected}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
			Error: {error}
		</div>
	{:else if shaders.length === 0}
		<p class="text-muted-foreground">No shaders yet.</p>
	{:else}
		<AdminTable
			data={shaders}
			{columns}
			selectedId={selected?.id}
			onRowClick={openShader}
			getRowId={(s: Shader) => s.id}
		>
			{#snippet cell({ columnId, value })}
				{#if columnId === 'name'}
					<span class="font-medium">{value}</span>
				{:else if columnId === 'description'}
					{#if value}
						<span class="line-clamp-1">{value}</span>
					{:else}
						<span class="text-muted-foreground">-</span>
					{/if}
				{:else}
					{value ?? '-'}
				{/if}
			{/snippet}
		</AdminTable>
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

			<div class="grid gap-2">
				<Label for="modrinth_id">Modrinth ID</Label>
				<Input id="modrinth_id" bind:value={editModrinthId} placeholder="e.g., abc123" />
			</div>

			<div class="grid gap-2">
				<Label for="curseforge_id">CurseForge ID</Label>
				<Input id="curseforge_id" bind:value={editCurseforgeId} placeholder="e.g., 123456" />
			</div>

			<div class="grid gap-2">
				<Label for="website_url">Website URL</Label>
				<Input
					id="website_url"
					bind:value={editWebsiteUrl}
					placeholder="e.g., https://example.com"
				/>
			</div>

			<div class="border-t pt-4">
				<dl class="space-y-2 text-sm">
					<AdminDetailField label="ID">
						<code class="text-xs">{selected.id}</code>
					</AdminDetailField>
					<AdminDetailField label="Slug">
						{selected.slug}
					</AdminDetailField>
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
	title="Delete Shader"
	description={`Delete shader "${selected?.name}"? This cannot be undone.`}
	confirmLabel="Delete"
	onConfirm={confirmDelete}
/>
