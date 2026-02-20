<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import { untrack } from 'svelte';
import { api } from '$lib/api';
import type {
	CaptureWithContext,
	ScenePreset,
	SceneWithVersion,
	UpdateSceneMetadataRequest
} from '$lib/bindings';
import { ItemGrid } from '$lib/components/item-grid';
import {
	TimeOfDaySlider,
	WeatherToggle,
	WeatherIntensitySlider,
	MoonPhaseSelector
} from '$lib/components/preset-controls';
import { formatDecimal, formatMoonPhase, formatTimeTicks } from '$lib/utils/format';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import Breadcrumb from '$lib/components/Breadcrumb.svelte';
import CaptureCard from '$lib/components/CaptureCard.svelte';
import DetailField from '$lib/components/DetailField.svelte';
import { createAction } from '$lib/components/action-form.svelte';
import { Alert } from '$lib/components/ui/alert';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import * as Dialog from '$lib/components/ui/dialog';
import * as FolderCard from '$lib/components/folder-card';
import * as Form from '$lib/components/ui/form';
import { Input } from '$lib/components/ui/input';
import { StatusBadge } from '$lib/components/ui/status-badge';
import { Textarea } from '$lib/components/ui/textarea';
import { GripVertical, LoaderCircle, Pencil, Plus, RotateCcw, Trash2 } from '@lucide/svelte';
import { defaults, superForm } from 'sveltekit-superforms';
import { zod4Client } from 'sveltekit-superforms/adapters';
import { toast } from 'svelte-sonner';
import { sceneFormSchema, createPresetSchema, editPresetSchema } from './schema.js';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let scene: SceneWithVersion = $derived(data.scene);
let captures: CaptureWithContext[] = $derived(data.captures);
let captureCount: number = $derived(data.captureCount);
let presets: ScenePreset[] = $derived(data.presets);

const initialScene = untrack(() => scene);
const sceneSuperform = superForm(
	defaults(
		{
			name: initialScene.name,
			description: initialScene.description ?? ''
		},
		zod4Client(sceneFormSchema)
	),
	{
		SPA: true,
		validators: zod4Client(sceneFormSchema),
		resetForm: false,
		onUpdate({ form }) {
			if (!form.valid) return;
			void (async () => {
				const request: UpdateSceneMetadataRequest = {
					name: form.data.name,
					description: form.data.description || undefined
				};
				const result = await api.admin.updateScene(scene.id, request);
				result.match({
					Ok: () => {
						toast.success('Scene updated');
						void invalidateAll();
					},
					Err: (err) => toast.error(err.message)
				});
			})();
		}
	}
);
const {
	form: sceneFormData,
	tainted: sceneTainted,
	submitting: sceneSubmitting,
	enhance: sceneEnhance
} = sceneSuperform;
let sceneIsDirty = $derived($sceneTainted != null && Object.values($sceneTainted).some(Boolean));

let showCreatePreset = $state(false);

const createPresetSuperform = superForm(
	defaults(
		{
			name: '',
			slug: '',
			time_of_day_ticks: 6000,
			weather: 'clear' as 'clear' | 'rain' | 'thunder' | 'snow',
			weather_intensity: 0,
			moon_phase: undefined as number | undefined
		},
		zod4Client(createPresetSchema)
	),
	{
		SPA: true,
		validators: zod4Client(createPresetSchema),
		resetForm: false,
		onUpdate({ form }) {
			if (!form.valid) return;
			void (async () => {
				const result = await api.admin.createPreset(scene.slug, {
					name: form.data.name,
					slug: form.data.slug,
					time_of_day_ticks: form.data.time_of_day_ticks,
					weather: form.data.weather,
					weather_intensity: form.data.weather_intensity,
					moon_phase: form.data.moon_phase
				});
				result.match({
					Ok: () => {
						showCreatePreset = false;
						createPresetSuperform.reset();
						void invalidateAll();
					},
					Err: (err) => toast.error(err.message)
				});
			})();
		}
	}
);
const {
	form: createFormData,
	submitting: createSubmitting,
	enhance: createEnhance
} = createPresetSuperform;

let editingPreset = $state<ScenePreset | null>(null);

const editPresetSuperform = superForm(
	defaults(
		{
			name: '',
			time_of_day_ticks: 6000,
			weather: 'clear' as 'clear' | 'rain' | 'thunder' | 'snow',
			weather_intensity: 0,
			moon_phase: undefined as number | undefined
		},
		zod4Client(editPresetSchema)
	),
	{
		SPA: true,
		validators: zod4Client(editPresetSchema),
		resetForm: false,
		onUpdate({ form }) {
			if (!form.valid || !editingPreset) return;
			void (async () => {
				const preset = editingPreset;
				const updates: Record<string, unknown> = {};
				if (form.data.name !== preset.name) updates.name = form.data.name;
				if (form.data.time_of_day_ticks !== preset.time_of_day_ticks)
					updates.time_of_day_ticks = form.data.time_of_day_ticks;
				if (form.data.weather !== preset.weather) updates.weather = form.data.weather;
				if (form.data.weather_intensity !== preset.weather_intensity)
					updates.weather_intensity = form.data.weather_intensity;
				if (form.data.moon_phase !== (preset.moon_phase ?? undefined))
					updates.moon_phase = form.data.moon_phase;

				if (Object.keys(updates).length === 0) {
					editingPreset = null;
					return;
				}

				const result = await api.admin.updatePreset(scene.slug, preset.slug, updates);
				result.match({
					Ok: () => {
						editingPreset = null;
						void invalidateAll();
					},
					Err: (err) => toast.error(err.message)
				});
			})();
		}
	}
);
const {
	form: editFormData,
	submitting: editSubmitting,
	enhance: editEnhance
} = editPresetSuperform;

function startEdit(preset: ScenePreset) {
	editingPreset = preset;
	$editFormData.name = preset.name;
	$editFormData.time_of_day_ticks = preset.time_of_day_ticks;
	$editFormData.weather = preset.weather as 'clear' | 'rain' | 'thunder' | 'snow';
	$editFormData.weather_intensity = preset.weather_intensity;
	$editFormData.moon_phase = preset.moon_phase ?? undefined;
}

const disableAction = createAction({
	action: () => api.admin.disableScene(scene.id),
	onSuccess: () => void goto('/admin/scenes'),
	setError: (msg: string) => toast.error(msg)
});

const reactivateAction = createAction({
	action: () => api.admin.reactivateScene(scene.id),
	onSuccess: () => void invalidateAll(),
	setError: (msg: string) => toast.error(msg)
});

let showDisableConfirm = $state(false);

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
			toast.error(err.message);
			deletingPreset = null;
			showDeletePresetConfirm = false;
		}
	});
}

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
		Err: (err) => toast.error(err.message)
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
	<Breadcrumb
		segments={[{ label: 'Scenes', href: '/admin/scenes' }, { label: scene.name }]}
	>
		{#snippet trailing()}
			<StatusBadge class="ml-2" status={scene.active ? 'active' : 'inactive'}>{scene.active ? 'Active' : 'Inactive'}</StatusBadge>
			<span class="ml-3 text-xs text-muted-foreground">Created <TimeAgo timestamp={scene.created_at} /></span>
		{/snippet}
	</Breadcrumb>

	{#if !scene.active}
		<Alert variant="warning" class="flex items-center justify-between">
			<span>This scene is inactive and will not be included in captures.</span>
			<Button variant="outline" size="sm" onclick={reactivateAction.execute} disabled={reactivateAction.loading}>
				<RotateCcw class="mr-1 h-3 w-3" />
				{reactivateAction.loading ? 'Reactivating...' : 'Reactivate'}
			</Button>
		</Alert>
	{/if}

	<FolderCard.Root value="details">
		{#snippet tabs()}
			<FolderCard.Tab value="details">Details</FolderCard.Tab>
			<FolderCard.Tab value="captures">Captures ({captureCount})</FolderCard.Tab>
		{/snippet}

		{#snippet trailing()}
			{#if scene.active}
				<Button
					variant="destructive"
					size="sm"
					onclick={() => (showDisableConfirm = true)}
					disabled={disableAction.loading}
				>
					<Trash2 class="mr-2 h-4 w-4" />
					Disable
				</Button>
			{/if}
		{/snippet}

		<FolderCard.Content value="details">
			<div class="space-y-8">
				<!-- Editing Section -->
				<section class="space-y-4">
					<h3 class="text-sm font-medium">Editing</h3>
					<form use:sceneEnhance class="space-y-4">
						<div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
							<Form.Field form={sceneSuperform} name="name">
								<Form.Control>
									{#snippet children({ props })}
										<Form.Label>Name</Form.Label>
										<Input {...props} bind:value={$sceneFormData.name} />
									{/snippet}
								</Form.Control>
								<Form.FieldErrors />
							</Form.Field>

							<div class="space-y-2">
								<label for="scene-slug" class="text-sm font-medium text-muted-foreground">Slug</label>
								<Input id="scene-slug" value={scene.slug} disabled class="font-mono text-xs" />
							</div>

							<div class="space-y-2">
								<label for="scene-id" class="text-sm font-medium text-muted-foreground">ID</label>
								<Input id="scene-id" value={scene.id} disabled class="font-mono text-xs" />
							</div>
						</div>

						<Form.Field form={sceneSuperform} name="description">
							<Form.Control>
								{#snippet children({ props })}
									<Form.Label>Description</Form.Label>
									<Textarea {...props} bind:value={$sceneFormData.description} rows={3} />
								{/snippet}
							</Form.Control>
							<Form.FieldErrors />
						</Form.Field>

						<div class="flex justify-end gap-2">
							<Button
								type="button"
								variant="outline"
								disabled={!sceneIsDirty || $sceneSubmitting}
								onclick={() => sceneSuperform.reset()}
							>
								Reset
							</Button>
							<Form.Button disabled={!sceneIsDirty || $sceneSubmitting}>
								{#if $sceneSubmitting}
									<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
								{/if}
								{$sceneSubmitting ? 'Saving...' : 'Save Changes'}
							</Form.Button>
						</div>
					</form>
				</section>

				<!-- Scene Info Section -->
				<section class="space-y-4">
					<h3 class="text-sm font-medium">Scene Info</h3>
					<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
						<!-- Location & Camera -->
						<div class="space-y-3 rounded-lg border border-border p-4">
							<h4 class="text-xs font-medium uppercase tracking-wide text-muted-foreground">Location & Camera</h4>
							<dl class="space-y-2 text-sm">
								<DetailField label="Position">
									<code class="text-xs">
										{formatDecimal(scene.version.x, 1)}, {formatDecimal(scene.version.y, 1)}, {formatDecimal(scene.version.z, 1)}
									</code>
								</DetailField>
								<DetailField label="Camera">
									<code class="text-xs">
										Yaw: {formatDecimal(scene.version.yaw, 1)}, Pitch: {formatDecimal(scene.version.pitch, 1)}
									</code>
								</DetailField>
								<DetailField label="Dimension">
									{scene.dimension}
								</DetailField>
								{#if scene.version.biome}
									<DetailField label="Biome">
										{scene.version.biome}
									</DetailField>
								{/if}
							</dl>
						</div>

						<!-- Environment -->
						<div class="space-y-3 rounded-lg border border-border p-4">
							<h4 class="text-xs font-medium uppercase tracking-wide text-muted-foreground">Environment</h4>
							<dl class="space-y-2 text-sm">
								<DetailField label="Time of Day">
									{formatTimeTicks(scene.version.time_of_day_ticks)}
								</DetailField>
								<DetailField label="Weather">
									<span class="capitalize">{scene.version.weather}</span>
									{#if scene.version.weather_intensity > 0}
										<span class="text-muted-foreground">
											(intensity: {formatDecimal(scene.version.weather_intensity)})
										</span>
									{/if}
								</DetailField>
								{#if scene.version.moon_phase != null}
									<DetailField label="Moon Phase">
										{formatMoonPhase(scene.version.moon_phase)}
									</DetailField>
								{/if}
							</dl>
						</div>
					</div>


				</section>

				<!-- Presets Section -->
				<section class="space-y-3">
					<h3 class="text-sm font-medium">Presets ({presets.length})</h3>
					<div role="list" class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
						<!-- Add Preset Card -->
						<button
							type="button"
							onclick={() => { createPresetSuperform.reset(); showCreatePreset = true; }}
							class="flex flex-col items-center justify-center gap-2 rounded-lg border-2 border-dashed border-border px-3 py-6 text-sm text-muted-foreground transition-colors hover:border-primary/40 hover:bg-muted/50 hover:text-foreground"
						>
							<Plus class="h-5 w-5" />
							<span>Add preset</span>
						</button>

						{#each presets as preset, i (preset.id)}
							<div
								role="listitem"
								draggable="true"
								ondragstart={(e) => handleDragStart(e, i)}
								ondragover={(e) => handleDragOver(e, i)}
								ondrop={() => handleDrop(i)}
								ondragend={handleDragEnd}
								class="flex flex-col rounded-lg border border-border bg-card transition-colors {dragOverIndex === i && dragIndex !== i ? 'border-primary/40 bg-muted/50' : ''} {dragIndex === i ? 'opacity-50' : ''}"
							>
								<!-- Header row -->
								<div class="flex items-center gap-2 border-b border-border px-3 py-2">
									<GripVertical class="h-4 w-4 shrink-0 cursor-grab text-muted-foreground opacity-30 transition-opacity hover:opacity-70" />
									<div class="flex min-w-0 flex-1 items-center gap-2">
										<span class="truncate text-sm font-medium">{preset.name}</span>
										{#if i === 0}
											<StatusBadge status="active" class="px-1.5 text-[10px]">Default</StatusBadge>
										{/if}
									</div>
									<div class="flex shrink-0 items-center gap-0.5">
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
								<!-- Details -->
								<div class="space-y-1 px-3 py-2.5">
									<code class="text-xs text-muted-foreground">{preset.slug}</code>
									<div class="flex flex-wrap gap-x-3 gap-y-0.5 text-xs text-muted-foreground">
										<span>{formatTimeTicks(preset.time_of_day_ticks)}</span>
										<span class="capitalize">{preset.weather}{preset.weather_intensity > 0 ? ` (${formatDecimal(preset.weather_intensity)})` : ''}</span>
										{#if preset.moon_phase != null}
											<span>{formatMoonPhase(preset.moon_phase)}</span>
										{/if}
									</div>
								</div>
							</div>
						{/each}
					</div>
				</section>
			</div>
		</FolderCard.Content>

		<!-- Tab: Captures -->
		<FolderCard.Content value="captures">
			{#if captures.length === 0}
				<div class="flex flex-col items-center justify-center py-12 text-center">
					<p class="text-sm text-muted-foreground">No captures have been taken for this scene yet.</p>
				</div>
			{:else}
				<div class="space-y-3">
					<div class="flex items-center justify-between">
						<p class="text-sm text-muted-foreground">
							Showing {captures.length} of {captureCount} captures
						</p>
						<a
							href="/admin/captures?scene={scene.id}"
							class="text-sm text-primary hover:underline"
						>
							View all
						</a>
					</div>
					<ItemGrid items={captures} key={(c: CaptureWithContext) => c.id} size="small">
						{#snippet card(capture: CaptureWithContext)}
							<CaptureCard {capture}>
								<div class="p-2">
									<div class="flex items-center justify-between">
										<div class="text-sm font-medium">{capture.shader_name}</div>
										{#if capture.freshness !== 'fresh'}
											<StatusBadge status={capture.freshness} class="px-1.5 text-[10px]">{capture.freshness}</StatusBadge>
										{/if}
									</div>
									<div class="text-xs text-muted-foreground">
										{capture.shader_version}
										{#if capture.profile_display_name}
											&middot; {capture.profile_display_name}
										{/if}
										{#if capture.preset_name}
											&middot; {capture.preset_name}
										{/if}
									</div>
								</div>
							</CaptureCard>
						{/snippet}
					</ItemGrid>
				</div>
			{/if}
		</FolderCard.Content>
	</FolderCard.Root>
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
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>New Preset</Dialog.Title>
			<Dialog.Description>Add a new environment preset to this scene.</Dialog.Description>
		</Dialog.Header>

		<form use:createEnhance class="space-y-5">
			<div class="grid grid-cols-2 gap-4">
				<Form.Field form={createPresetSuperform} name="name">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Name</Form.Label>
							<Input {...props} bind:value={$createFormData.name} placeholder="e.g., Sunset" />
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				<Form.Field form={createPresetSuperform} name="slug">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Slug</Form.Label>
							<Input {...props} bind:value={$createFormData.slug} placeholder="e.g., sunset" class="font-mono" />
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>
			</div>

			<div class="space-y-1.5">
				<span class="text-sm font-medium">Time of Day</span>
				<TimeOfDaySlider bind:value={$createFormData.time_of_day_ticks} />
			</div>

			<div class="space-y-1.5">
				<span class="text-sm font-medium">Weather</span>
				<WeatherToggle bind:value={$createFormData.weather} />
			</div>

			{#if $createFormData.weather !== 'clear'}
				<WeatherIntensitySlider bind:value={$createFormData.weather_intensity} />
			{/if}

			<MoonPhaseSelector bind:value={$createFormData.moon_phase} />

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (showCreatePreset = false)}>Cancel</Button>
				<Form.Button disabled={$createSubmitting}>
					{#if $createSubmitting}
						<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
					{/if}
					{$createSubmitting ? 'Creating...' : 'Create'}
				</Form.Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Edit Preset Dialog -->
<Dialog.Root
	open={editingPreset !== null}
	onOpenChange={(open) => { if (!open) editingPreset = null; }}
>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>Edit Preset</Dialog.Title>
			<Dialog.Description>
				{#if editingPreset}
					Update settings for "{editingPreset.name}".
				{:else}
					Update preset settings.
				{/if}
			</Dialog.Description>
		</Dialog.Header>

		<form use:editEnhance class="space-y-5">
			<Form.Field form={editPresetSuperform} name="name">
				<Form.Control>
					{#snippet children({ props })}
						<Form.Label>Name</Form.Label>
						<Input {...props} bind:value={$editFormData.name} />
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<div class="space-y-1.5">
				<span class="text-sm font-medium">Time of Day</span>
				<TimeOfDaySlider bind:value={$editFormData.time_of_day_ticks} />
			</div>

			<div class="space-y-1.5">
				<span class="text-sm font-medium">Weather</span>
				<WeatherToggle bind:value={$editFormData.weather} />
			</div>

			{#if $editFormData.weather !== 'clear'}
				<WeatherIntensitySlider bind:value={$editFormData.weather_intensity} />
			{/if}

			<MoonPhaseSelector bind:value={$editFormData.moon_phase} />

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (editingPreset = null)}>Cancel</Button>
				<Form.Button disabled={$editSubmitting}>
					{#if $editSubmitting}
						<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
					{/if}
					{$editSubmitting ? 'Saving...' : 'Save Changes'}
				</Form.Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
