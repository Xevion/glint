<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import { api } from '$lib/api';
import type { UpdateWorldRequest } from '$lib/api/endpoints/admin';
import type { Scene, WorldVersion, WorldWithDetails } from '$lib/bindings';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import WorldUploadDialog from '$lib/components/WorldUploadDialog.svelte';
import { AdminDetailField, AdminDetailHeader } from '$lib/components/admin';
import { Alert } from '$lib/components/ui/alert';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import { StatusBadge } from '$lib/components/ui/status-badge';
import { Textarea } from '$lib/components/ui/textarea';
import { formatBytes } from '$lib/utils/format';
import { ChevronDown, ChevronRight, Trash2 } from '@lucide/svelte';
import { untrack } from 'svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let world: WorldWithDetails = $derived(data.world);
let scenes: Scene[] = $derived(data.world.scenes);

let saving = $state(false);
let actionLoading = $state(false);
let error = $state<string | null>(null);
let showDeleteConfirm = $state(false);

let versions = $state<WorldVersion[]>([]);
let loadingVersions = $state(false);
let showVersions = $state(false);

let editName = $state('');
let editDescription = $state('');

let isDirty = $derived(editName !== world.name || editDescription !== (world.description ?? ''));

$effect(() => {
	void world.id;
	untrack(() => {
		editName = world.name;
		editDescription = world.description ?? '';
	});
});

async function handleSave() {
	saving = true;
	error = null;

	try {
		const request: UpdateWorldRequest = {};
		if (editName !== world.name) request.name = editName;
		if (editDescription !== (world.description ?? ''))
			request.description = editDescription || undefined;

		if (Object.keys(request).length === 0) return;

		const result = await api.admin.updateWorld(world.id, request);
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

async function confirmDelete() {
	actionLoading = true;
	try {
		const result = await api.admin.deleteWorld(world.id);
		result.match({
			Ok: () => {
				void goto('/admin/worlds');
			},
			Err: (err) => {
				error = err.message;
			}
		});
	} finally {
		actionLoading = false;
	}
}

async function toggleVersions() {
	showVersions = !showVersions;
	if (showVersions && versions.length === 0) {
		await loadVersions();
	}
}

async function loadVersions() {
	loadingVersions = true;
	const result = await api.admin.listWorldVersions(world.id);
	if (result.isOk) {
		versions = result.value;
	} else {
		error = result.error.message;
	}
	loadingVersions = false;
}
</script>

<svelte:head><title>{world.name} - Glint</title></svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<AdminDetailHeader
		backHref="/admin/worlds"
		backLabel="Back to worlds"
		title={world.name}
	>
		{#snippet trailing()}
			<code class="rounded bg-muted px-1.5 py-0.5 text-xs">{world.minecraft_version}</code>
		{/snippet}
	</AdminDetailHeader>

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
				<code class="text-xs">{world.id}</code>
			</AdminDetailField>
			<AdminDetailField label="Slug">
				{world.slug}
			</AdminDetailField>
			<AdminDetailField label="Minecraft Version">
				<code class="rounded bg-muted px-1.5 py-0.5 text-xs">{world.minecraft_version}</code>
			</AdminDetailField>
			<AdminDetailField label="Size">
				{world.latest_version?.size_bytes ? formatBytes(world.latest_version.size_bytes) : '-'}
			</AdminDetailField>
			{#if world.latest_version?.file_hash}
				<AdminDetailField label="File Hash">
					<code class="text-xs">{world.latest_version.file_hash}</code>
				</AdminDetailField>
			{/if}
			<AdminDetailField label="Created">
				<TimeAgo timestamp={world.created_at} />
			</AdminDetailField>
			<AdminDetailField label="Updated">
				<TimeAgo timestamp={world.updated_at} />
			</AdminDetailField>
		</dl>
	</div>

	<!-- Versions Section -->
	<div class="space-y-3">
		<div class="flex items-center justify-between">
			<h2 class="text-lg font-medium">Versions</h2>
		<WorldUploadDialog
			worldId={world.id}
			worldName={world.name}
			onWorldCreated={() => {
				versions = [];
				void invalidateAll();
				if (showVersions) void loadVersions();
			}}
		/>
		</div>

		<button
			type="button"
			class="flex w-full items-center gap-2 text-sm text-foreground/70 hover:text-foreground"
			onclick={toggleVersions}
		>
			{#if showVersions}
				<ChevronDown class="h-4 w-4" />
			{:else}
				<ChevronRight class="h-4 w-4" />
			{/if}
			{showVersions ? 'Hide' : 'Show'} version history
		</button>

		{#if showVersions}
			{#if loadingVersions}
				<p class="text-sm text-foreground">Loading versions...</p>
			{:else if versions.length === 0}
				<p class="text-sm text-foreground">No versions found.</p>
			{:else}
				<div class="space-y-2">
					{#each versions as version, i (version.id)}
						<div
							class="flex items-center justify-between rounded-lg border bg-card p-3 text-sm"
						>
							<div class="flex items-center gap-3">
								<div>
									<div class="flex items-center gap-2">
										<TimeAgo timestamp={version.created_at} />
										{#if i === 0}
											<span
												class="rounded-full bg-blue-100 px-2 py-0.5 text-xs text-blue-800 dark:bg-blue-900 dark:text-blue-300"
											>
												Latest
											</span>
										{/if}
									</div>
									<div class="text-xs text-muted-foreground">
										{version.size_bytes != null ? formatBytes(version.size_bytes) : '—'}
										{#if version.file_hash}
											&middot;
											<code class="text-xs">{version.file_hash}</code>
										{/if}
									</div>
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		{/if}
	</div>

	<!-- Scenes Section -->
	<div class="space-y-3">
		<h2 class="text-lg font-medium">Scenes ({scenes.length})</h2>
		{#if scenes.length === 0}
			<p class="text-sm text-foreground">No scenes for this world.</p>
		{:else}
			<div class="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
				{#each scenes as scene (scene.id)}
					<a
						href="/admin/scenes/{scene.id}"
						class="flex items-center justify-between rounded-lg border bg-card p-3 transition-colors hover:bg-muted/50"
					>
						<div>
							<div class="font-medium">{scene.name}</div>
						<div class="text-xs text-muted-foreground">
							{scene.dimension.split(':').pop()}
						</div>
						</div>
					<StatusBadge status={scene.active ? 'active' : 'inactive'}>{scene.active ? 'Active' : 'Inactive'}</StatusBadge>
					</a>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Actions -->
	<div class="border-t pt-4">
		<Button variant="destructive" onclick={() => (showDeleteConfirm = true)} disabled={actionLoading}>
			<Trash2 class="mr-2 h-4 w-4" />
			Delete World
		</Button>
	</div>
</div>

<ConfirmDialog
	bind:open={showDeleteConfirm}
	title="Delete World"
	description={`Delete world "${world.name}"? This will also delete the file from storage.`}
	confirmLabel="Delete"
	onConfirm={confirmDelete}
/>
