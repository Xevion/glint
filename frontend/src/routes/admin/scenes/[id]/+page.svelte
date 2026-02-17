<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import { api } from '$lib/api';
import type {
	CaptureWithContext,
	ScenePreset,
	SceneWithVersion,
	UpdateSceneMetadataRequest
} from '$lib/bindings';
import { ItemGrid } from '$lib/components/item-grid';
import { freshnessColors } from '$lib/utils/status';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import {
	AdminCaptureCard,
	AdminDetailField,
	AdminDetailHeader,
	createAdminAction,
	createAdminForm
} from '$lib/components/admin';
import { Alert } from '$lib/components/ui/alert';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import * as Dialog from '$lib/components/ui/dialog';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import { StatusBadge } from '$lib/components/ui/status-badge';
import { Textarea } from '$lib/components/ui/textarea';
import { GripVertical, Pencil, Plus, RotateCcw, Trash2 } from '@lucide/svelte';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let scene: SceneWithVersion = $derived(data.scene);
let captures: CaptureWithContext[] = $derived(data.captures);
let captureCount: number = $derived(data.captureCount);
let presets: ScenePreset[] = $derived(data.presets);

const form = createAdminForm({
	source: () => scene,
	fields: {
		name: (s) => s.name,
		description: (s) => s.description ?? ''
	},
	onSave: (changes, source) => {
		const request: UpdateSceneMetadataRequest = {};
		for (const [key, value] of Object.entries(changes)) {
			(request as Record<string, unknown>)[key] = key === 'name' ? value : value || undefined;
		}
		return api.admin.updateScene(source.id, request);
	}
});

const disableAction = createAdminAction({
	action: () => api.admin.disableScene(scene.id),
	onSuccess: () => void goto('/admin/scenes'),
	setError: (msg) => (form.error = msg)
});

const reactivateAction = createAdminAction({
	action: () => api.admin.reactivateScene(scene.id),
	onSuccess: () => void invalidateAll(),
	setError: (msg) => (form.error = msg)
});

let showDisableConfirm = $state(false);

// Create Preset
let showCreatePreset = $state(false);
let newPreset = $state({
	name: '',
	slug: '',
	time_of_day_ticks: 6000,
	weather: 'clear',
	weather_intensity: 0,
	moon_phase: undefined as number | undefined
});
let createPresetError = $state<string | null>(null);
let creatingPreset = $state(false);

function resetCreateForm() {
	newPreset = {
		name: '',
		slug: '',
		time_of_day_ticks: 6000,
		weather: 'clear',
		weather_intensity: 0,
		moon_phase: undefined
	};
	createPresetError = null;
}

async function handleCreatePreset() {
	creatingPreset = true;
	createPresetError = null;
	const result = await api.admin.createPreset(scene.slug, {
		name: newPreset.name,
		slug: newPreset.slug,
		time_of_day_ticks: newPreset.time_of_day_ticks,
		weather: newPreset.weather,
		weather_intensity: newPreset.weather_intensity,
		moon_phase: newPreset.moon_phase
	});
	result.match({
		Ok: () => {
			showCreatePreset = false;
			resetCreateForm();
			void invalidateAll();
		},
		Err: (err) => {
			createPresetError = err.message;
		}
	});
	creatingPreset = false;
}

// Edit Preset
let editingPreset = $state<ScenePreset | null>(null);
let editFields = $state({
	name: '',
	time_of_day_ticks: 0,
	weather: '',
	weather_intensity: 0,
	moon_phase: undefined as number | undefined
});
let editPresetError = $state<string | null>(null);
let savingPreset = $state(false);

function startEdit(preset: ScenePreset) {
	editingPreset = preset;
	editFields = {
		name: preset.name,
		time_of_day_ticks: preset.time_of_day_ticks,
		weather: preset.weather,
		weather_intensity: preset.weather_intensity,
		moon_phase: preset.moon_phase ?? undefined
	};
	editPresetError = null;
}

async function handleUpdatePreset() {
	if (!editingPreset) return;
	savingPreset = true;
	editPresetError = null;
	const updates: Record<string, unknown> = {};
	if (editFields.name !== editingPreset.name) updates.name = editFields.name;
	if (editFields.time_of_day_ticks !== editingPreset.time_of_day_ticks)
		updates.time_of_day_ticks = editFields.time_of_day_ticks;
	if (editFields.weather !== editingPreset.weather) updates.weather = editFields.weather;
	if (editFields.weather_intensity !== editingPreset.weather_intensity)
		updates.weather_intensity = editFields.weather_intensity;
	if (editFields.moon_phase !== (editingPreset.moon_phase ?? undefined))
		updates.moon_phase = editFields.moon_phase;

	if (Object.keys(updates).length === 0) {
		editingPreset = null;
		savingPreset = false;
		return;
	}

	const result = await api.admin.updatePreset(scene.slug, editingPreset.slug, updates);
	result.match({
		Ok: () => {
			editingPreset = null;
			void invalidateAll();
		},
		Err: (err) => {
			editPresetError = err.message;
		}
	});
	savingPreset = false;
}

// Delete Preset
let deletingPreset = $state<ScenePreset | null>(null);
let showDeletePresetConfirm = $state(false);

async function handleDeletePreset() {
	if (!deletingPreset) return;
	const result = await api.admin.deletePreset(scene.slug, deletingPreset.slug);
	result.match({
		Ok: () => {
			deletingPreset = null;
			showDeletePresetConfirm = false;
			void invalidateAll();
		},
		Err: (err) => {
			form.error = err.message;
			deletingPreset = null;
			showDeletePresetConfirm = false;
		}
	});
}

// Drag-to-Reorder
let dragIndex = $state<number | null>(null);
let dragOverIndex = $state<number | null>(null);

function handleDragStart(e: DragEvent, index: number) {
	dragIndex = index;
	if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
}

function handleDragOver(e: DragEvent, index: number) {
	e.preventDefault();
	if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
	dragOverIndex = index;
}

async function handleDrop(targetIndex: number) {
	if (dragIndex === null || dragIndex === targetIndex) {
		dragIndex = null;
		dragOverIndex = null;
		return;
	}
	const reordered = [...presets];
	const [moved] = reordered.splice(dragIndex, 1);
	reordered.splice(targetIndex, 0, moved);
	const presetIds = reordered.map((p) => p.id);
	dragIndex = null;
	dragOverIndex = null;

	const result = await api.admin.reorderPresets(scene.slug, presetIds);
	result.match({
		Ok: () => void invalidateAll(),
		Err: (err) => {
			form.error = err.message;
		}
	});
}

function handleDragEnd() {
	dragIndex = null;
	dragOverIndex = null;
}
</script>

<svelte:head><title>{scene.name} - Glint</title></svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<AdminDetailHeader
		backHref="/admin/scenes"
		backLabel="Back to scenes"
		title={scene.name}
	>
		{#snippet trailing()}
			<StatusBadge status={scene.active ? 'active' : 'inactive'}>{scene.active ? 'Active' : 'Inactive'}</StatusBadge>
		{/snippet}
	</AdminDetailHeader>

	{#if !scene.active}
		<Alert variant="warning" class="flex items-center justify-between">
			<span>This scene is inactive and will not be included in captures.</span>
		<Button variant="outline" size="sm" onclick={reactivateAction.execute} disabled={reactivateAction.loading}>
			<RotateCcw class="mr-1 h-3 w-3" />
			{reactivateAction.loading ? 'Reactivating...' : 'Reactivate'}
		</Button>
		</Alert>
	{/if}

	{#if form.error}
		<Alert variant="destructive">{form.error}</Alert>
	{/if}

	<!-- Edit Section -->
	<div class="space-y-4 rounded-lg border bg-card p-4">
		<div class="grid gap-2">
			<Label for="name">Name</Label>
			<Input id="name" bind:value={form.fields.name} />
		</div>

		<div class="grid gap-2">
			<Label for="description">Description</Label>
			<Textarea id="description" bind:value={form.fields.description} rows={3} />
		</div>

		<div class="flex justify-end">
			<Button onclick={form.save} disabled={form.saving || !form.isDirty}>
				{form.saving ? 'Saving...' : 'Save Changes'}
			</Button>
		</div>
	</div>

	<!-- Metadata -->
	<div class="rounded-lg border bg-card p-4">
		<dl class="space-y-2 text-sm">
			<AdminDetailField label="ID">
				<code class="text-xs">{scene.id}</code>
			</AdminDetailField>
			<AdminDetailField label="Slug">
				{scene.slug}
			</AdminDetailField>
		<AdminDetailField label="Position">
			<code class="text-xs">
				{scene.version.x.toFixed(1)}, {scene.version.y.toFixed(1)}, {scene.version.z.toFixed(1)}
			</code>
		</AdminDetailField>
		<AdminDetailField label="Camera">
			<code class="text-xs">
				Yaw: {scene.version.yaw.toFixed(1)}, Pitch: {scene.version.pitch.toFixed(1)}
			</code>
		</AdminDetailField>
		<AdminDetailField label="Dimension">
			{scene.dimension}
		</AdminDetailField>
		<AdminDetailField label="Time of Day">
			{scene.version.time_of_day_ticks} ticks
		</AdminDetailField>
		<AdminDetailField label="Weather">
			<span class="capitalize">{scene.version.weather}</span>
			{#if scene.version.weather_intensity > 0}
				<span class="text-muted-foreground">
					(intensity: {scene.version.weather_intensity.toFixed(2)})
				</span>
			{/if}
		</AdminDetailField>
		{#if scene.version.biome}
			<AdminDetailField label="Biome">
				{scene.version.biome}
			</AdminDetailField>
		{/if}
		{#if scene.version.moon_phase != null}
			<AdminDetailField label="Moon Phase">
				{scene.version.moon_phase}
			</AdminDetailField>
		{/if}
			<AdminDetailField label="Created">
				<TimeAgo timestamp={scene.created_at} />
			</AdminDetailField>
		</dl>
	</div>

	<!-- Presets Section -->
	<div class="space-y-3">
		<div class="flex items-center justify-between">
			<h2 class="text-lg font-medium">Presets ({presets.length})</h2>
			<Button size="sm" onclick={() => { resetCreateForm(); showCreatePreset = true; }}>
				<Plus class="mr-1 h-3 w-3" />
				New Preset
			</Button>
		</div>

		{#if presets.length === 0}
			<p class="text-sm text-muted-foreground">No presets configured.</p>
		{:else}
		<div role="list" class="flex flex-col overflow-hidden rounded-lg border border-border">
			{#each presets as preset, i (preset.id)}
				<div
					role="listitem"
					draggable="true"
					ondragstart={(e) => handleDragStart(e, i)}
					ondragover={(e) => handleDragOver(e, i)}
					ondrop={() => handleDrop(i)}
					ondragend={handleDragEnd}
					class="flex items-center gap-4 border-b border-border bg-card p-3 last:border-b-0 {dragOverIndex === i && dragIndex !== i ? 'bg-muted/50' : ''}"
				>
					<GripVertical class="h-4 w-4 shrink-0 cursor-grab text-muted-foreground/50" />
					<div class="min-w-0 flex-1">
						<div class="flex items-center gap-2">
							<span class="text-sm font-medium">{preset.name}</span>
							<code class="rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">{preset.slug}</code>
							{#if i === 0}
								<span class="rounded-full bg-primary/10 px-2 py-0.5 text-[10px] font-medium text-primary">Default</span>
							{/if}
						</div>
						<div class="mt-1 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
							<span>{preset.time_of_day_ticks} ticks</span>
							<span class="capitalize">{preset.weather}{preset.weather_intensity > 0 ? ` (${preset.weather_intensity.toFixed(2)})` : ''}</span>
							{#if preset.moon_phase != null}
								<span>Moon: {preset.moon_phase}</span>
							{/if}
						</div>
					</div>
					<div class="flex shrink-0 items-center gap-1">
						<Button variant="ghost" size="icon" class="h-7 w-7" onclick={() => startEdit(preset)}>
							<Pencil class="h-3.5 w-3.5" />
						</Button>
						<Button
							variant="ghost"
							size="icon"
							class="h-7 w-7 text-destructive/70 hover:text-destructive"
							onclick={() => { deletingPreset = preset; showDeletePresetConfirm = true; }}
							disabled={presets.length <= 1}
						>
							<Trash2 class="h-3.5 w-3.5" />
						</Button>
					</div>
				</div>
			{/each}
		</div>
		{/if}
	</div>

	<!-- Captures Section -->
	<div class="space-y-3">
		<div class="flex items-center justify-between">
			<h2 class="text-lg font-medium">Captures ({captureCount})</h2>
			{#if captureCount > 0}
				<a
					href="/admin/captures?scene={scene.id}"
					class="text-sm text-primary hover:underline"
				>
					View all
				</a>
			{/if}
		</div>
	{#if captures.length === 0}
		<p class="text-sm text-muted-foreground">No captures yet.</p>
	{:else}
		<ItemGrid items={captures} key={(c: CaptureWithContext) => c.id} size="small">
	{#snippet card(capture: CaptureWithContext)}
		<AdminCaptureCard {capture}>
			<div class="p-2">
				<div class="flex items-center justify-between">
					<div class="text-sm font-medium">{capture.shader_name}</div>
					{#if capture.freshness !== 'fresh'}
					<span class="rounded-full px-1.5 py-0.5 text-[10px] font-medium {freshnessColors[capture.freshness]}">
						{capture.freshness}
					</span>
				{/if}
				</div>
			<div class="text-xs text-muted-foreground">
				{capture.shader_version}
			{#if capture.profile_name}
				&middot; {capture.profile_name}
			{/if}
			{#if capture.preset_name}
				&middot; {capture.preset_name}
			{/if}
			</div>
			</div>
		</AdminCaptureCard>
	{/snippet}
		</ItemGrid>
	{/if}
	</div>

	<!-- Actions -->
	{#if scene.active}
		<div class="border-t pt-4">
		<Button variant="destructive" onclick={() => (showDisableConfirm = true)} disabled={disableAction.loading}>
			<Trash2 class="mr-2 h-4 w-4" />
			Disable Scene
		</Button>
		</div>
	{/if}
</div>

<ConfirmDialog
	bind:open={showDisableConfirm}
	title="Disable Scene"
	description={`Disable scene "${scene.name}"? It will be hidden from captures.`}
	confirmLabel="Disable"
	onConfirm={disableAction.execute}
/>

<ConfirmDialog
	bind:open={showDeletePresetConfirm}
	title="Delete Preset"
	description={`Delete preset "${deletingPreset?.name}"? This cannot be undone.`}
	confirmLabel="Delete"
	onConfirm={handleDeletePreset}
/>

<!-- Create Preset Dialog -->
<Dialog.Root bind:open={showCreatePreset}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>New Preset</Dialog.Title>
			<Dialog.Description>Add a new preset to this scene.</Dialog.Description>
		</Dialog.Header>

		{#if createPresetError}
			<Alert variant="destructive">{createPresetError}</Alert>
		{/if}

		<div class="grid gap-4 py-4">
			<div class="grid gap-2">
				<Label for="preset-name">Name</Label>
				<Input id="preset-name" bind:value={newPreset.name} required />
			</div>
			<div class="grid gap-2">
				<Label for="preset-slug">Slug</Label>
				<Input id="preset-slug" bind:value={newPreset.slug} required />
			</div>
			<div class="grid gap-2">
				<Label for="preset-time">Time of Day (ticks)</Label>
				<Input id="preset-time" type="number" min={0} max={24000} bind:value={newPreset.time_of_day_ticks} />
			</div>
			<div class="grid gap-2">
				<Label for="preset-weather">Weather</Label>
				<select
					id="preset-weather"
					class="h-9 rounded-md border border-input bg-background px-3 text-sm shadow-xs transition-[color,box-shadow] outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:bg-input/30"
					bind:value={newPreset.weather}
				>
					<option value="clear">Clear</option>
					<option value="rain">Rain</option>
					<option value="thunder">Thunder</option>
					<option value="snow">Snow</option>
				</select>
			</div>
			<div class="grid gap-2">
				<Label for="preset-intensity">Weather Intensity</Label>
				<Input id="preset-intensity" type="number" min={0} max={1} step={0.01} bind:value={newPreset.weather_intensity} />
			</div>
			<div class="grid gap-2">
				<Label for="preset-moon">Moon Phase (0-7, optional)</Label>
				<Input
					id="preset-moon"
					type="number"
					min={0}
					max={7}
					value={newPreset.moon_phase ?? ''}
					oninput={(e: Event & { currentTarget: HTMLInputElement }) => {
						const v = e.currentTarget.value;
						newPreset.moon_phase = v === '' ? undefined : Number(v);
					}}
				/>
			</div>
		</div>

		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showCreatePreset = false)}>Cancel</Button>
			<Button onclick={handleCreatePreset} disabled={creatingPreset || !newPreset.name || !newPreset.slug}>
				{creatingPreset ? 'Creating...' : 'Create'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- Edit Preset Dialog -->
<Dialog.Root
	open={editingPreset !== null}
	onOpenChange={(open) => { if (!open) editingPreset = null; }}
>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Edit Preset</Dialog.Title>
			<Dialog.Description>Update preset settings.</Dialog.Description>
		</Dialog.Header>

		{#if editPresetError}
			<Alert variant="destructive">{editPresetError}</Alert>
		{/if}

		<div class="grid gap-4 py-4">
			<div class="grid gap-2">
				<Label for="edit-name">Name</Label>
				<Input id="edit-name" bind:value={editFields.name} required />
			</div>
			<div class="grid gap-2">
				<Label for="edit-time">Time of Day (ticks)</Label>
				<Input id="edit-time" type="number" min={0} max={24000} bind:value={editFields.time_of_day_ticks} />
			</div>
			<div class="grid gap-2">
				<Label for="edit-weather">Weather</Label>
				<select
					id="edit-weather"
					class="h-9 rounded-md border border-input bg-background px-3 text-sm shadow-xs transition-[color,box-shadow] outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:bg-input/30"
					bind:value={editFields.weather}
				>
					<option value="clear">Clear</option>
					<option value="rain">Rain</option>
					<option value="thunder">Thunder</option>
					<option value="snow">Snow</option>
				</select>
			</div>
			<div class="grid gap-2">
				<Label for="edit-intensity">Weather Intensity</Label>
				<Input id="edit-intensity" type="number" min={0} max={1} step={0.01} bind:value={editFields.weather_intensity} />
			</div>
			<div class="grid gap-2">
				<Label for="edit-moon">Moon Phase (0-7, optional)</Label>
				<Input
					id="edit-moon"
					type="number"
					min={0}
					max={7}
					value={editFields.moon_phase ?? ''}
					oninput={(e: Event & { currentTarget: HTMLInputElement }) => {
						const v = e.currentTarget.value;
						editFields.moon_phase = v === '' ? undefined : Number(v);
					}}
				/>
			</div>
		</div>

		<Dialog.Footer>
			<Button variant="outline" onclick={() => (editingPreset = null)}>Cancel</Button>
			<Button onclick={handleUpdatePreset} disabled={savingPreset || !editFields.name}>
				{savingPreset ? 'Saving...' : 'Save Changes'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
