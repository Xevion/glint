<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import { api } from '$lib/api';
import type {
	CaptureWithContext,
	SceneWithVersion,
	UpdateSceneMetadataRequest,
	WorldWithDetails
} from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { ItemGrid } from '$lib/components/item-grid';
import { freshnessColors } from '$lib/utils/status';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { AdminDetailField, AdminDetailHeader } from '$lib/components/admin';
import { Alert } from '$lib/components/ui/alert';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import { StatusBadge } from '$lib/components/ui/status-badge';
import { Textarea } from '$lib/components/ui/textarea';
import { RotateCcw, Trash2 } from '@lucide/svelte';
import { untrack } from 'svelte';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let scene: SceneWithVersion = $derived(data.scene);
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
			<Button variant="outline" size="sm" onclick={handleReactivate} disabled={actionLoading}>
				<RotateCcw class="mr-1 h-3 w-3" />
				{actionLoading ? 'Reactivating...' : 'Reactivate'}
			</Button>
		</Alert>
	{/if}

	{#if error}
		<Alert variant="destructive">{error}</Alert>
	{/if}

	<!-- Edit Section -->
	<div class="space-y-4 rounded-lg border bg-card p-4">
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
	<div class="rounded-lg border bg-card p-4">
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
			<a href="/admin/captures/{capture.id}" class="overflow-hidden rounded-lg border transition-colors hover:bg-muted/50">
				{#if capture.image_url}
					<CaptureImage
						src={capture.image_url}
						thumbhash={capture.thumbhash}
						preset="card"
						alt={capture.shader_name}
						class="w-full"
						containerClass="aspect-video w-full"
					/>
				{:else}
					<div class="flex aspect-video w-full items-center justify-center bg-muted text-xs text-muted-foreground">
						No image
					</div>
				{/if}
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
					</div>
				</div>
			</a>
		{/snippet}
		</ItemGrid>
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
