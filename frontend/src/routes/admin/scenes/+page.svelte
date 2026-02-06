<script lang="ts">
import { invalidateAll } from '$app/navigation';
import { api } from '$lib/api';
import type { UpdateSceneMetadataRequest } from '$lib/api/endpoints/admin';
import type { SceneWithWorld } from '$lib/bindings';
import AdminTable from '$lib/components/AdminTable.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { AdminDetailField, AdminSlideOver } from '$lib/components/admin';
import { Button } from '$lib/components/ui/button';
import * as Checkbox from '$lib/components/ui/checkbox';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import { Textarea } from '$lib/components/ui/textarea';
import { RefreshCw, RotateCcw, Trash2 } from '@lucide/svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let scenes: SceneWithWorld[] = $derived(data.scenes);
let selected = $state<SceneWithWorld | null>(null);
let refreshing = $state(false);
let saving = $state(false);
let error = $state<string | null>(null);
let showInactive = $state(false);
let showDisableConfirm = $state(false);

// Edit form state
let editName = $state('');
let editDescription = $state('');

// Derived filtered scenes
const filteredScenes = $derived(showInactive ? scenes : scenes.filter((s) => s.active));

const columns = [
	{ id: 'name', key: 'name', name: 'Name' },
	{ id: 'slug', key: 'slug', name: 'Slug' },
	{ id: 'world', key: 'world_name', name: 'World' },
	{ id: 'dimension', key: 'dimension', name: 'Dimension' },
	{ id: 'weather', key: 'weather', name: 'Weather' },
	{ id: 'created_at', key: 'created_at', name: 'Created', component: 'time' as const }
];

async function refresh() {
	refreshing = true;
	error = null;
	await invalidateAll();
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
	result.match({
		Ok: (value) => {
			selected = { ...selected!, ...value };
			void refresh();
		},
		Err: (err) => {
			error = err.message;
		}
	});
	saving = false;
}

function handleDisable() {
	if (!selected) return;
	showDisableConfirm = true;
}

async function confirmDisable() {
	if (!selected) return;

	const result = await api.admin.disableScene(selected.id);
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

async function handleReactivate() {
	if (!selected) return;

	const result = await api.admin.reactivateScene(selected.id);
	result.match({
		Ok: () => {
			selected = { ...selected!, active: true };
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
			<h1 class="text-2xl font-semibold">Scenes</h1>
			<span class="text-lg text-muted-foreground">{filteredScenes.length}</span>
			{#if showInactive && scenes.some((s) => !s.active)}
				<span class="text-sm text-muted-foreground">
					({scenes.filter((s) => !s.active).length} inactive)
				</span>
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
			<Button variant="outline" size="icon" onclick={refresh} disabled={refreshing}>
				<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
			</Button>
		</div>
	</header>

	{#if error && !selected}
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
		>
			{#snippet cell({ columnId, value, row }: { columnId: string; value: unknown; row: SceneWithWorld })}
				{#if columnId === 'name'}
					<span class="font-medium">{value}</span>
					{#if !row.active}
						<span class="ml-2 rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">Inactive</span>
					{/if}
				{:else if columnId === 'world'}
					{#if value}
						<span>{value}</span>
					{:else}
						<span class="text-muted-foreground">-</span>
					{/if}
				{:else if columnId === 'dimension'}
					<span class="text-xs">{typeof value === 'string' ? value.split(':').pop() : value}</span>
				{:else if columnId === 'weather'}
					<span class="text-xs capitalize">{value}</span>
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

		{#if !selected.active}
			<div class="mb-4 rounded-lg border border-yellow-500 bg-yellow-50 p-3 text-sm text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-300">
				This scene is inactive and will not be included in captures.
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
		<div class="flex justify-end">
			<div class="inline-flex">
				{#if selected?.active}
					<Button
						variant="destructive"
						onclick={handleDisable}
						class="rounded-r-none border-r-0"
						size="icon"
					>
						<Trash2 class="h-4 w-4" />
						<span class="sr-only">Disable</span>
					</Button>
				{:else}
					<Button
						variant="outline"
						onclick={handleReactivate}
						class="rounded-r-none border-r-0"
						size="icon"
					>
						<RotateCcw class="h-4 w-4" />
						<span class="sr-only">Reactivate</span>
					</Button>
				{/if}
				<Button onclick={handleSave} disabled={saving} class="rounded-l-none">
					{saving ? 'Saving...' : 'Save Changes'}
				</Button>
			</div>
		</div>
	{/snippet}
</AdminSlideOver>

<ConfirmDialog
	bind:open={showDisableConfirm}
	title="Disable Scene"
	description={`Disable scene "${selected?.name}"? It will be hidden from captures.`}
	confirmLabel="Disable"
	onConfirm={confirmDisable}
/>
