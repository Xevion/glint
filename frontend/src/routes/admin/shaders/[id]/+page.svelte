<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import { untrack } from 'svelte';
import { api } from '$lib/api';
import type { UpdateShaderRequest } from '$lib/api/endpoints/admin';
import type { CaptureWithContext, ShaderVersion, ShaderWithCaptures } from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { AdminDetailField } from '$lib/components/admin';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import { Textarea } from '$lib/components/ui/textarea';
import { ArrowLeft, Trash2 } from '@lucide/svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let shader: ShaderWithCaptures = $derived(data.shader);
let versions: ShaderVersion[] = $derived(data.shader.versions);
let captures: CaptureWithContext[] = $derived(data.shader.captures);

let saving = $state(false);
let actionLoading = $state(false);
let error = $state<string | null>(null);
let showDeleteConfirm = $state(false);

let editName = $state('');
let editDescription = $state('');
let editModrinthId = $state('');
let editCurseforgeId = $state('');
let editWebsiteUrl = $state('');

$effect(() => {
	void shader.id;
	untrack(() => {
		editName = shader.name;
		editDescription = shader.description ?? '';
		editModrinthId = shader.modrinth_id ?? '';
		editCurseforgeId = shader.curseforge_id ?? '';
		editWebsiteUrl = shader.website_url ?? '';
	});
});

async function handleSave() {
	saving = true;
	error = null;

	try {
		const request: UpdateShaderRequest = {};
		if (editName !== shader.name) request.name = editName;
		if (editDescription !== (shader.description ?? ''))
			request.description = editDescription || undefined;
		if (editModrinthId !== (shader.modrinth_id ?? ''))
			request.modrinth_id = editModrinthId || undefined;
		if (editCurseforgeId !== (shader.curseforge_id ?? ''))
			request.curseforge_id = editCurseforgeId || undefined;
		if (editWebsiteUrl !== (shader.website_url ?? ''))
			request.website_url = editWebsiteUrl || undefined;

		if (Object.keys(request).length === 0) return;

		const result = await api.admin.updateShader(shader.id, request);
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
		const result = await api.admin.deleteShader(shader.id);
		result.match({
			Ok: () => {
				void goto('/admin/shaders');
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
		<a href="/admin/shaders" class="text-muted-foreground hover:text-foreground" aria-label="Back to shaders">
			<ArrowLeft class="h-4 w-4" />
		</a>
			<h1 class="text-2xl font-semibold">{shader.name}</h1>
			{#if shader.icon_url}
				<img
					src={shader.icon_url}
					alt="{shader.name} icon"
					class="h-6 w-6 rounded"
				/>
			{/if}
		</div>
	</header>

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

		<div class="flex justify-end">
			<Button onclick={handleSave} disabled={saving}>
				{saving ? 'Saving...' : 'Save Changes'}
			</Button>
		</div>
	</div>

	<!-- Metadata -->
	<div class="rounded-lg border p-4">
		<dl class="space-y-2 text-sm">
			<AdminDetailField label="ID">
				<code class="text-xs">{shader.id}</code>
			</AdminDetailField>
			<AdminDetailField label="Slug">
				{shader.slug}
			</AdminDetailField>
			{#if shader.license_id}
				<AdminDetailField label="License">
					{shader.license_id}
				</AdminDetailField>
			{/if}
			{#if shader.upstream_downloads}
				<AdminDetailField label="Upstream Downloads">
					{shader.upstream_downloads.toLocaleString()}
				</AdminDetailField>
			{/if}
			{#if shader.last_synced_at}
				<AdminDetailField label="Last Synced">
					<TimeAgo timestamp={shader.last_synced_at} />
				</AdminDetailField>
			{/if}
			<AdminDetailField label="Created">
				<TimeAgo timestamp={shader.created_at} />
			</AdminDetailField>
			<AdminDetailField label="Updated">
				<TimeAgo timestamp={shader.updated_at} />
			</AdminDetailField>
		</dl>
	</div>

	<!-- Versions Section -->
	<div class="space-y-3">
		<h2 class="text-lg font-medium">Versions ({versions.length})</h2>
		{#if versions.length === 0}
			<p class="text-sm text-muted-foreground">No versions yet.</p>
		{:else}
			<div class="rounded-md border">
				<table class="w-full text-sm">
					<thead>
						<tr class="border-b bg-muted/50">
							<th class="px-4 py-2 text-left font-medium">Version</th>
							<th class="px-4 py-2 text-left font-medium">Game Versions</th>
							<th class="px-4 py-2 text-left font-medium">Channel</th>
							<th class="px-4 py-2 text-left font-medium">Created</th>
						</tr>
					</thead>
					<tbody>
						{#each versions as version (version.id)}
							<tr class="border-b last:border-0">
								<td class="px-4 py-2 font-medium">{version.version}</td>
								<td class="px-4 py-2 text-xs text-muted-foreground">
									{version.game_versions ?? '-'}
								</td>
								<td class="px-4 py-2 text-xs capitalize">
									{version.release_channel ?? '-'}
								</td>
								<td class="px-4 py-2">
									<TimeAgo timestamp={version.created_at} />
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</div>

	<!-- Captures Section -->
	<div class="space-y-3">
		<div class="flex items-center justify-between">
			<h2 class="text-lg font-medium">Captures ({captures.length})</h2>
			{#if captures.length > 0}
				<a
					href="/admin/captures?shader={shader.slug}"
					class="text-sm text-primary hover:underline"
				>
					View all
				</a>
			{/if}
		</div>
		{#if captures.length === 0}
			<p class="text-sm text-muted-foreground">No captures yet.</p>
		{:else}
			<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
				{#each captures as capture (capture.id)}
					<a
						href="/admin/captures/{capture.id}"
						class="overflow-hidden rounded-lg border transition-colors hover:bg-muted/50"
					>
						{#if capture.image_url}
							<CaptureImage
								src={capture.image_url}
								thumbhash={capture.thumbhash}
								preset="card"
								alt={capture.scene_name ?? capture.scene_id}
								class="w-full"
								containerClass="aspect-video w-full"
							/>
						{:else}
							<div class="flex aspect-video w-full items-center justify-center bg-muted text-xs text-muted-foreground">
								No image
							</div>
						{/if}
						<div class="p-2">
							<div class="text-sm font-medium">{capture.scene_name ?? capture.scene_id}</div>
							<div class="text-xs text-muted-foreground">
								{capture.shader_version}
								{#if capture.profile}
									&middot; {capture.profile}
								{/if}
							</div>
						</div>
					</a>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Actions -->
	<div class="border-t pt-4">
		<Button variant="destructive" onclick={() => (showDeleteConfirm = true)} disabled={actionLoading}>
			<Trash2 class="mr-2 h-4 w-4" />
			Delete Shader
		</Button>
	</div>
</div>

<ConfirmDialog
	bind:open={showDeleteConfirm}
	title="Delete Shader"
	description={`Delete shader "${shader.name}"? This cannot be undone.`}
	confirmLabel="Delete"
	onConfirm={confirmDelete}
/>
