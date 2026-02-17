<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import { api } from '$lib/api';
import type {
	CaptureWithContext,
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
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import { StatusBadge } from '$lib/components/ui/status-badge';
import { Textarea } from '$lib/components/ui/textarea';
import { RotateCcw, Trash2 } from '@lucide/svelte';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let scene: SceneWithVersion = $derived(data.scene);
let captures: CaptureWithContext[] = $derived(data.captures);
let captureCount: number = $derived(data.captureCount);

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
