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
import { formatMoonPhase, formatTimeTicks } from '$lib/utils/display';
import { freshnessColors } from '$lib/utils/status';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import {
	AdminBreadcrumb,
	AdminCaptureCard,
	AdminDetailField,
	createAdminAction
} from '$lib/components/admin';
import { Alert } from '$lib/components/ui/alert';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import * as Dialog from '$lib/components/ui/dialog';
import * as FolderCard from '$lib/components/folder-card';
import * as Form from '$lib/components/ui/form';
import * as Select from '$lib/components/ui/select';
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

// ── Scene Edit Form (superforms SPA) ────────────────────────
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

// ── Create Preset Form (superforms SPA) ─────────────────────
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

// ── Edit Preset Form (superforms SPA) ───────────────────────
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

// ── Non-form actions ────────────────────────────────────────
const disableAction = createAdminAction({
	action: () => api.admin.disableScene(scene.id),
	onSuccess: () => void goto('/admin/scenes'),
	setError: (msg) => toast.error(msg)
});

const reactivateAction = createAdminAction({
	action: () => api.admin.reactivateScene(scene.id),
	onSuccess: () => void invalidateAll(),
	setError: (msg) => toast.error(msg)
});

let showDisableConfirm = $state(false);

// ── Delete Preset ───────────────────────────────────────────
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

// ── Drag-to-Reorder ─────────────────────────────────────────
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

const WEATHER_OPTIONS = [
	{ value: 'clear', label: 'Clear' },
	{ value: 'rain', label: 'Rain' },
	{ value: 'thunder', label: 'Thunder' },
	{ value: 'snow', label: 'Snow' }
] as const;
</script>

<svelte:head><title>{scene.name} - Glint</title></svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<AdminBreadcrumb
		segments={[{ label: 'Scenes', href: '/admin/scenes' }, { label: scene.name }]}
	>
		{#snippet trailing()}
			<StatusBadge class="ml-2" status={scene.active ? 'active' : 'inactive'}>{scene.active ? 'Active' : 'Inactive'}</StatusBadge>
		{/snippet}
	</AdminBreadcrumb>

	{#if !scene.active}
		<Alert variant="warning" class="flex items-center justify-between">
			<span>This scene is inactive and will not be included in captures.</span>
			<Button variant="outline" size="sm" onclick={reactivateAction.execute} disabled={reactivateAction.loading}>
				<RotateCcw class="mr-1 h-3 w-3" />
				{reactivateAction.loading ? 'Reactivating...' : 'Reactivate'}
			</Button>
		</Alert>
	{/if}

	<FolderCard.Root value="editing">
		{#snippet tabs()}
			<FolderCard.Tab value="editing">Editing & Presets</FolderCard.Tab>
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

		<!-- Tab 1: Editing & Presets -->
		<FolderCard.Content value="editing">
			<div class="space-y-6">
				<!-- Scene Edit Form -->
				<form use:sceneEnhance class="space-y-4">
					<Form.Field form={sceneSuperform} name="name">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>Name</Form.Label>
								<Input {...props} bind:value={$sceneFormData.name} />
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

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

				<!-- Presets Section -->
				<div class="space-y-3">
					<div class="flex items-center justify-between">
						<h3 class="text-sm font-medium">Presets ({presets.length})</h3>
						<Button size="sm" onclick={() => { createPresetSuperform.reset(); showCreatePreset = true; }}>
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
											<span>{formatTimeTicks(preset.time_of_day_ticks)}</span>
											<span class="capitalize">{preset.weather}{preset.weather_intensity > 0 ? ` (${preset.weather_intensity.toFixed(2)})` : ''}</span>
											{#if preset.moon_phase != null}
												<span>{formatMoonPhase(preset.moon_phase)}</span>
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
			</div>
		</FolderCard.Content>

		<!-- Tab 2: Details -->
		<FolderCard.Content value="details">
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
					{formatTimeTicks(scene.version.time_of_day_ticks)}
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
						{formatMoonPhase(scene.version.moon_phase)}
					</AdminDetailField>
				{/if}
				<AdminDetailField label="Created">
					<TimeAgo timestamp={scene.created_at} />
				</AdminDetailField>
			</dl>
		</FolderCard.Content>

		<!-- Tab 3: Captures -->
		<FolderCard.Content value="captures">
			{#if captures.length === 0}
				<p class="text-sm text-muted-foreground">No captures yet.</p>
			{:else}
				<div class="space-y-3">
					<div class="flex justify-end">
						<a
							href="/admin/captures?scene={scene.id}"
							class="text-sm text-primary hover:underline"
						>
							View all
						</a>
					</div>
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
										{#if capture.profile_display_name}
											&middot; {capture.profile_display_name}
										{/if}
										{#if capture.preset_name}
											&middot; {capture.preset_name}
										{/if}
									</div>
								</div>
							</AdminCaptureCard>
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
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>New Preset</Dialog.Title>
			<Dialog.Description>Add a new preset to this scene.</Dialog.Description>
		</Dialog.Header>

		<form use:createEnhance class="space-y-4">
			<Form.Field form={createPresetSuperform} name="name">
				<Form.Control>
					{#snippet children({ props })}
						<Form.Label>Name</Form.Label>
						<Input {...props} bind:value={$createFormData.name} />
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field form={createPresetSuperform} name="slug">
				<Form.Control>
					{#snippet children({ props })}
						<Form.Label>Slug</Form.Label>
						<Input {...props} bind:value={$createFormData.slug} />
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field form={createPresetSuperform} name="time_of_day_ticks">
				<Form.Control>
					{#snippet children({ props })}
						<Form.Label>Time of Day (ticks)</Form.Label>
						<Input {...props} type="number" min={0} max={24000} bind:value={$createFormData.time_of_day_ticks} />
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field form={createPresetSuperform} name="weather">
				<Form.Control>
					{#snippet children({ props }: { props: { name: string } })}
						<Form.Label>Weather</Form.Label>
						<Select.Root
							type="single"
							value={$createFormData.weather}
							onValueChange={(v: string) => {
								$createFormData.weather = v as 'clear' | 'rain' | 'thunder' | 'snow';
							}}
							name={props.name}
						>
							<Select.Trigger class="w-full">
								<span class="capitalize">{$createFormData.weather}</span>
							</Select.Trigger>
							<Select.Content>
								{#each WEATHER_OPTIONS as opt (opt.value)}
									<Select.Item value={opt.value}>{opt.label}</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field form={createPresetSuperform} name="weather_intensity">
				<Form.Control>
					{#snippet children({ props })}
						<Form.Label>Weather Intensity</Form.Label>
						<Input {...props} type="number" min={0} max={1} step={0.01} bind:value={$createFormData.weather_intensity} />
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field form={createPresetSuperform} name="moon_phase">
				<Form.Control>
					{#snippet children({ props })}
						<Form.Label>Moon Phase (0-7, optional)</Form.Label>
						<Input
							{...props}
							type="number"
							min={0}
							max={7}
							value={$createFormData.moon_phase ?? ''}
							oninput={(e: Event & { currentTarget: HTMLInputElement }) => {
								const v = e.currentTarget.value;
								$createFormData.moon_phase = v === '' ? undefined : Number(v);
							}}
						/>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

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
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Edit Preset</Dialog.Title>
			<Dialog.Description>Update preset settings.</Dialog.Description>
		</Dialog.Header>

		<form use:editEnhance class="space-y-4">
			<Form.Field form={editPresetSuperform} name="name">
				<Form.Control>
					{#snippet children({ props })}
						<Form.Label>Name</Form.Label>
						<Input {...props} bind:value={$editFormData.name} />
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field form={editPresetSuperform} name="time_of_day_ticks">
				<Form.Control>
					{#snippet children({ props })}
						<Form.Label>Time of Day (ticks)</Form.Label>
						<Input {...props} type="number" min={0} max={24000} bind:value={$editFormData.time_of_day_ticks} />
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field form={editPresetSuperform} name="weather">
				<Form.Control>
					{#snippet children({ props }: { props: { name: string } })}
						<Form.Label>Weather</Form.Label>
						<Select.Root
							type="single"
							value={$editFormData.weather}
							onValueChange={(v: string) => {
								$editFormData.weather = v as 'clear' | 'rain' | 'thunder' | 'snow';
							}}
							name={props.name}
						>
							<Select.Trigger class="w-full">
								<span class="capitalize">{$editFormData.weather}</span>
							</Select.Trigger>
							<Select.Content>
								{#each WEATHER_OPTIONS as opt (opt.value)}
									<Select.Item value={opt.value}>{opt.label}</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field form={editPresetSuperform} name="weather_intensity">
				<Form.Control>
					{#snippet children({ props })}
						<Form.Label>Weather Intensity</Form.Label>
						<Input {...props} type="number" min={0} max={1} step={0.01} bind:value={$editFormData.weather_intensity} />
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field form={editPresetSuperform} name="moon_phase">
				<Form.Control>
					{#snippet children({ props })}
						<Form.Label>Moon Phase (0-7, optional)</Form.Label>
						<Input
							{...props}
							type="number"
							min={0}
							max={7}
							value={$editFormData.moon_phase ?? ''}
							oninput={(e: Event & { currentTarget: HTMLInputElement }) => {
								const v = e.currentTarget.value;
								$editFormData.moon_phase = v === '' ? undefined : Number(v);
							}}
						/>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

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
