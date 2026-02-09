<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import { untrack } from 'svelte';
import { api } from '$lib/api';
import type { UpdateSceneMetadataRequest } from '$lib/api/endpoints/admin';
import type { CaptureWithContext, Scene, WorldWithDetails } from '$lib/bindings';
import CaptureGridAdmin from '$lib/components/CaptureGridAdmin.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { AdminDetailField } from '$lib/components/admin';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import { Textarea } from '$lib/components/ui/textarea';
import { ArrowLeft, RotateCcw, Trash2 } from '@lucide/svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let scene: Scene = $derived(data.scene);
let world: WorldWithDetails | null = $derived(data.world);
let captures: CaptureWithContext[] = $derived(data.captures);
let captureCount: number = $derived(data.captureCount);

let saving = $state(false);
let actionLoading = $state(false);
let error = $state<string | null>(null);
let showDisableConfirm = $state(false);

let editName = $state('');
let editDescription = $state('');

let isDirty = $derived(editName !== scene.name || editDescription !== (scene.description ?? ''));

$effect(() => {
	void scene.id;
	untrack(() => {
		editName = scene.name;
		editDescription = scene.description ?? '';
	});
});

async function handleSave() {
	saving = true;
	error = null;

	try {
		const request: UpdateSceneMetadataRequest = {};
		if (editName !== scene.name) request.name = editName;
		if (editDescription !== (scene.description ?? ''))
			request.description = editDescription || undefined;

		if (Object.keys(request).length === 0) return;

		const result = await api.admin.updateScene(scene.id, request);
		result.match({
			Ok: () => {
				void invalidateAll();
			},
			Err: (err) => {
				error = err.message;
			}
		});
	} finally {
		saving = false;
	}
}

async function confirmDisable() {
	actionLoading = true;
	try {
		const result = await api.admin.disableScene(scene.id);
		result.match({
			Ok: () => {
				void goto('/admin/scenes');
			},
			Err: (err) => {
				error = err.message;
			}
		});
	} finally {
		actionLoading = false;
	}
}

async function handleReactivate() {
	actionLoading = true;
	try {
		const result = await api.admin.reactivateScene(scene.id);
		result.match({
			Ok: () => {
				void invalidateAll();
			},
			Err: (err) => {
				error = err.message;
			}
		});
	} finally {
		actionLoading = false;
	}
}
</script>

<div class="space-y-6">
	<!-- Header -->
	<header class="space-y-2">
		<div class="flex items-center gap-2">
		<a href="/admin/scenes" class="text-muted-foreground hover:text-foreground" aria-label="Back to scenes">
			<ArrowLeft class="h-4 w-4" />
		</a>
			<h1 class="text-2xl font-semibold">{scene.name}</h1>
			{#if scene.active}
				<span
					class="rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-800 dark:bg-green-900 dark:text-green-300"
				>
					Active
				</span>
			{:else}
				<span
					class="rounded-full bg-yellow-100 px-2 py-0.5 text-xs font-medium text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300"
				>
					Inactive
				</span>
			{/if}
		</div>
	</header>

	{#if !scene.active}
		<div
			class="flex items-center justify-between rounded-lg border border-yellow-500 bg-yellow-50 p-3 text-sm text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-300"
		>
			<span>This scene is inactive and will not be included in captures.</span>
		<Button variant="outline" size="sm" onclick={handleReactivate} disabled={actionLoading}>
			<RotateCcw class="mr-1 h-3 w-3" />
			{actionLoading ? 'Reactivating...' : 'Reactivate'}
		</Button>
		</div>
	{/if}

	{#if error}
		<div
			class="rounded-lg border border-destructive bg-destructive/10 p-3 text-sm text-destructive"
		>
			{error}
		</div>
	{/if}

	<!-- Edit Section -->
	<div class="space-y-4 rounded-lg border p-4">
		<div class="grid gap-2">
			<Label for="name">Name</Label>
			<Input id="name" bind:value={editName} />
		</div>

		<div class="grid gap-2">
			<Label for="description">Description</Label>
			<Textarea id="description" bind:value={editDescription} rows={3} />
		</div>

		<div class="flex justify-end">
			<Button onclick={handleSave} disabled={saving || !isDirty}>
				{saving ? 'Saving...' : 'Save Changes'}
			</Button>
		</div>
	</div>

	<!-- Metadata -->
	<div class="rounded-lg border p-4">
		<dl class="space-y-2 text-sm">
			<AdminDetailField label="ID">
				<code class="text-xs">{scene.id}</code>
			</AdminDetailField>
			<AdminDetailField label="Slug">
				{scene.slug}
			</AdminDetailField>
		<AdminDetailField label="World">
			<a
				href="/admin/worlds/{scene.world_id}"
				class="text-primary hover:underline"
			>
				{world?.name ?? scene.world_id}
			</a>
		</AdminDetailField>
			<AdminDetailField label="Position">
				<code class="text-xs">
					{scene.x.toFixed(1)}, {scene.y.toFixed(1)}, {scene.z.toFixed(1)}
				</code>
			</AdminDetailField>
			<AdminDetailField label="Camera">
				<code class="text-xs">
					Yaw: {scene.yaw.toFixed(1)}, Pitch: {scene.pitch.toFixed(1)}
				</code>
			</AdminDetailField>
			<AdminDetailField label="Dimension">
				{scene.dimension}
			</AdminDetailField>
			<AdminDetailField label="Time of Day">
				{scene.time_of_day_ticks} ticks
			</AdminDetailField>
			<AdminDetailField label="Weather">
				<span class="capitalize">{scene.weather}</span>
				{#if scene.weather_intensity > 0}
					<span class="text-muted-foreground">
						(intensity: {scene.weather_intensity.toFixed(2)})
					</span>
				{/if}
			</AdminDetailField>
			{#if scene.biome}
				<AdminDetailField label="Biome">
					{scene.biome}
				</AdminDetailField>
			{/if}
			{#if scene.moon_phase !== null}
				<AdminDetailField label="Moon Phase">
					{scene.moon_phase}
				</AdminDetailField>
			{/if}
			<AdminDetailField label="Created">
				<TimeAgo timestamp={scene.created_at} />
			</AdminDetailField>
		</dl>
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
		<CaptureGridAdmin {captures}>
			{#snippet footer(capture: CaptureWithContext)}
				<div class="p-2">
					<div class="text-sm font-medium">{capture.shader_name}</div>
					<div class="text-xs text-muted-foreground">
						{capture.shader_version}
						{#if capture.profile}
							&middot; {capture.profile}
						{/if}
					</div>
				</div>
			{/snippet}
		</CaptureGridAdmin>
	{/if}
	</div>

	<!-- Actions -->
	{#if scene.active}
		<div class="border-t pt-4">
		<Button variant="destructive" onclick={() => (showDisableConfirm = true)} disabled={actionLoading}>
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
	onConfirm={confirmDisable}
/>
