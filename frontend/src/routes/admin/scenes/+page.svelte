<script lang="ts">
import { onMount } from 'svelte';
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import { Textarea } from '$lib/components/ui/textarea';
import * as Checkbox from '$lib/components/ui/checkbox';
import { RefreshCw, Trash2, Save, RotateCcw } from '@lucide/svelte';
import AdminTable from '$lib/components/admin-table.svelte';
import { AdminSlideOver, AdminDetailField } from '$lib/components/admin';
import TimeAgo from '$lib/components/time-ago.svelte';
import { api } from '$lib/api';
import type { SceneWithWorld } from '$lib/bindings';
import type { UpdateSceneMetadataRequest } from '$lib/api/endpoints/admin';
import { escapeHtml } from '$lib/utils/display';

let scenes = $state<SceneWithWorld[]>([]);
let selected = $state<SceneWithWorld | null>(null);
let loading = $state(true);
let refreshing = $state(false);
let saving = $state(false);
let error = $state<string | null>(null);
let showInactive = $state(false);

// Edit form state
let editName = $state('');
let editDescription = $state('');

// Derived filtered scenes
const filteredScenes = $derived(showInactive ? scenes : scenes.filter((s) => s.active));

const columns = [
	{
		id: 'name',
		key: 'name',
		name: 'Name',
		render: (value: string, row: SceneWithWorld) => {
			const safeName = escapeHtml(value);
			const inactive = !row.active
				? '<span class="ml-2 rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">Inactive</span>'
				: '';
			return `<span class="font-medium">${safeName}</span>${inactive}`;
		}
	},
	{ id: 'slug', key: 'slug', name: 'Slug' },
	{
		id: 'world',
		key: 'world_name',
		name: 'World',
		render: (value: string | null) =>
			value ? `<span>${escapeHtml(value)}</span>` : '<span class="text-muted-foreground">-</span>'
	},
	{
		id: 'dimension',
		key: 'dimension',
		name: 'Dimension',
		render: (value: string) => {
			const short = value.split(':').pop() ?? value;
			return `<span class="text-xs">${escapeHtml(short)}</span>`;
		}
	},
	{
		id: 'weather',
		key: 'weather',
		name: 'Weather',
		render: (value: string) => `<span class="text-xs capitalize">${escapeHtml(value)}</span>`
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
	const result = await api.admin.listScenes();
	if (result.isOk) {
		scenes = result.value;
	} else {
		error = result.error.message;
	}
	loading = false;
	refreshing = false;
}

function openScene(scene: SceneWithWorld) {
	selected = scene;
	editName = scene.name;
	editDescription = scene.description ?? '';
}

async function handleSave() {
	if (!selected) return;

	saving = true;
	error = null;

	const request: UpdateSceneMetadataRequest = {};
	if (editName !== selected.name) request.name = editName;
	if (editDescription !== (selected.description ?? ''))
		request.description = editDescription || undefined;

	if (Object.keys(request).length === 0) {
		saving = false;
		return;
	}

	const result = await api.admin.updateScene(selected.id, request);
	if (result.isOk) {
		// Merge result into selected (keeps world_name, world_slug)
		selected = { ...selected, ...result.value };
		scenes = scenes.map((s) => (s.id === selected!.id ? { ...s, ...result.value } : s));
	} else {
		error = result.error.message;
	}
	saving = false;
}

async function handleDisable() {
	if (!selected) return;
	if (!confirm(`Disable scene "${selected.name}"? It will be hidden from capture jobs.`)) return;

	const result = await api.admin.disableScene(selected.id);
	if (result.isOk) {
		selected = null;
		void load();
	} else {
		error = result.error.message;
	}
}

async function handleReactivate() {
	if (!selected) return;

	const result = await api.admin.reactivateScene(selected.id);
	if (result.isOk) {
		// Update the scene in the list
		selected = { ...selected, active: true };
		scenes = scenes.map((s) => (s.id === selected!.id ? { ...s, active: true } : s));
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
			<h1 class="text-2xl font-semibold">Scenes</h1>
			{#if !loading}
				<span class="text-lg text-muted-foreground">{filteredScenes.length}</span>
				{#if showInactive && scenes.some((s) => !s.active)}
					<span class="text-sm text-muted-foreground">
						({scenes.filter((s) => !s.active).length} inactive)
					</span>
				{/if}
			{/if}
		</div>
		<div class="flex items-center gap-4">
			<label class="flex items-center gap-2 text-sm">
				<Checkbox.Root
					checked={showInactive}
					onCheckedChange={(v) => (showInactive = v === true)}
				/>
				<span>Show inactive</span>
			</label>
			<Button variant="outline" size="icon" onclick={load} disabled={refreshing}>
				<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
			</Button>
		</div>
	</header>

	{#if loading}
		<div class="text-center text-muted-foreground">Loading...</div>
	{:else if error && !selected}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
			Error: {error}
		</div>
	{:else if filteredScenes.length === 0}
		<p class="text-muted-foreground">
			{showInactive ? 'No scenes yet.' : 'No active scenes. Enable "Show inactive" to see disabled scenes.'}
		</p>
	{:else}
		<AdminTable
			data={filteredScenes}
			{columns}
			selectedId={selected?.id}
			onRowClick={openScene}
			getRowId={(s: SceneWithWorld) => s.id}
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

		{#if !selected.active}
			<div class="mb-4 rounded-lg border border-yellow-500 bg-yellow-50 p-3 text-sm text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-300">
				This scene is inactive and will not be included in capture jobs.
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
					<AdminDetailField label="World">
						{selected.world_name ?? '-'}
						{#if selected.world_slug}
							<span class="text-muted-foreground">({selected.world_slug})</span>
						{/if}
					</AdminDetailField>
					<AdminDetailField label="Status">
						{#if selected.active}
							<span class="text-green-600 dark:text-green-400">Active</span>
						{:else}
							<span class="text-yellow-600 dark:text-yellow-400">Inactive</span>
						{/if}
					</AdminDetailField>
					<AdminDetailField label="Position">
						<code class="text-xs">
							{selected.x.toFixed(1)}, {selected.y.toFixed(1)}, {selected.z.toFixed(1)}
						</code>
					</AdminDetailField>
					<AdminDetailField label="Camera">
						<code class="text-xs">
							Yaw: {selected.yaw.toFixed(1)}, Pitch: {selected.pitch.toFixed(1)}
						</code>
					</AdminDetailField>
					<AdminDetailField label="Dimension">
						{selected.dimension}
					</AdminDetailField>
					<AdminDetailField label="Time of Day">
						{selected.time_of_day_ticks} ticks
					</AdminDetailField>
					<AdminDetailField label="Weather">
						<span class="capitalize">{selected.weather}</span>
						{#if selected.weather_intensity > 0}
							<span class="text-muted-foreground">
								(intensity: {selected.weather_intensity.toFixed(2)})
							</span>
						{/if}
					</AdminDetailField>
					{#if selected.biome}
						<AdminDetailField label="Biome">
							{selected.biome}
						</AdminDetailField>
					{/if}
					{#if selected.moon_phase !== null}
						<AdminDetailField label="Moon Phase">
							{selected.moon_phase}
						</AdminDetailField>
					{/if}
					<AdminDetailField label="Created">
						<TimeAgo timestamp={selected.created_at} />
					</AdminDetailField>
				</dl>
			</div>
		</div>
	{/if}

	{#snippet footer()}
		<div class="flex justify-between">
			{#if selected?.active}
				<Button variant="destructive" onclick={handleDisable}>
					<Trash2 class="mr-2 h-4 w-4" />
					Disable
				</Button>
			{:else}
				<Button variant="outline" onclick={handleReactivate}>
					<RotateCcw class="mr-2 h-4 w-4" />
					Reactivate
				</Button>
			{/if}
			<Button onclick={handleSave} disabled={saving}>
				<Save class="mr-2 h-4 w-4" />
				{saving ? 'Saving...' : 'Save Changes'}
			</Button>
		</div>
	{/snippet}
</AdminSlideOver>
