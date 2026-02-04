<script lang="ts">
import { onMount } from 'svelte';
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import { Textarea } from '$lib/components/ui/textarea';
import { RefreshCw, Trash2, Save } from '@lucide/svelte';
import AdminTable from '$lib/components/admin-table.svelte';
import { AdminSlideOver, AdminDetailField } from '$lib/components/admin';
import TimeAgo from '$lib/components/time-ago.svelte';
import { api } from '$lib/api';
import type { Shader } from '$lib/bindings';
import type { UpdateShaderRequest } from '$lib/api/endpoints/admin';

let shaders = $state<Shader[]>([]);
let selected = $state<Shader | null>(null);
let loading = $state(true);
let refreshing = $state(false);
let saving = $state(false);
let error = $state<string | null>(null);

// Edit form state
let editName = $state('');
let editDescription = $state('');
let editModrinthId = $state('');
let editCurseforgeId = $state('');
let editWebsiteUrl = $state('');

const columns = [
	{
		id: 'name',
		key: 'name',
		name: 'Name',
		render: (value: string) => `<span class="font-medium">${value}</span>`
	},
	{ id: 'slug', key: 'slug', name: 'Slug' },
	{
		id: 'description',
		key: 'description',
		name: 'Description',
		render: (value: string | null) =>
			value
				? `<span class="line-clamp-1">${value}</span>`
				: '<span class="text-muted-foreground">-</span>'
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
	const result = await api.admin.listShaders();
	if (result.isOk) {
		shaders = result.value;
	} else {
		error = result.error.message;
	}
	loading = false;
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
	if (result.isOk) {
		selected = result.value;
		shaders = shaders.map((s) => (s.id === selected!.id ? result.value : s));
	} else {
		error = result.error.message;
	}
	saving = false;
}

async function handleDelete() {
	if (!selected) return;
	if (!confirm(`Delete shader "${selected.name}"? This cannot be undone.`)) return;

	const result = await api.admin.deleteShader(selected.id);
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
			<h1 class="text-2xl font-semibold">Shaders</h1>
			{#if !loading}
				<span class="text-lg text-muted-foreground">{shaders.length}</span>
			{/if}
		</div>
		<Button variant="outline" size="icon" onclick={load} disabled={refreshing}>
			<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
		</Button>
	</header>

	{#if loading}
		<div class="text-center text-muted-foreground">Loading...</div>
	{:else if error && !selected}
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
		<div class="flex justify-between">
			<Button variant="destructive" onclick={handleDelete}>
				<Trash2 class="mr-2 h-4 w-4" />
				Delete
			</Button>
			<Button onclick={handleSave} disabled={saving}>
				<Save class="mr-2 h-4 w-4" />
				{saving ? 'Saving...' : 'Save Changes'}
			</Button>
		</div>
	{/snippet}
</AdminSlideOver>
