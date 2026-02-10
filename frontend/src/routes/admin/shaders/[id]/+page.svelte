<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import { untrack } from 'svelte';
import { api } from '$lib/api';
import type { UpdateShaderRequest } from '$lib/api/endpoints/admin';
import type { CaptureWithContext, ShaderVersion, ShaderWithCaptures } from '$lib/bindings';
import CaptureGridAdmin from '$lib/components/CaptureGridAdmin.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { AdminDetailField, AdminDetailHeader } from '$lib/components/admin';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import { Textarea } from '$lib/components/ui/textarea';
import * as Table from '$lib/components/ui/table';
import { Alert } from '$lib/components/ui/alert';
import { formatGameVersions } from '$lib/utils/display';
import { Trash2 } from '@lucide/svelte';
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

let isDirty = $derived(
	editName !== shader.name ||
		editDescription !== (shader.description ?? '') ||
		editModrinthId !== (shader.modrinth_id ?? '') ||
		editCurseforgeId !== (shader.curseforge_id ?? '') ||
		editWebsiteUrl !== (shader.website_url ?? '')
);

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

<svelte:head><title>{shader.name} - Glint</title></svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<AdminDetailHeader
		backHref="/admin/shaders"
		backLabel="Back to shaders"
		title={shader.name}
	>
		{#snippet trailing()}
			{#if shader.icon_url}
				<img
					src={shader.icon_url}
					alt="{shader.name} icon"
					class="h-6 w-6 rounded"
				/>
			{/if}
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
			<Button onclick={handleSave} disabled={saving || !isDirty}>
				{saving ? 'Saving...' : 'Save Changes'}
			</Button>
		</div>
	</div>

	<!-- Metadata -->
	<div class="rounded-lg border bg-card p-4">
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
		<Table.Root class="border">
			<Table.Header>
				<Table.Row class="bg-muted/50">
					<Table.Head class="px-4 py-2">Version</Table.Head>
					<Table.Head class="px-4 py-2">Game Versions</Table.Head>
					<Table.Head class="px-4 py-2">Channel</Table.Head>
					<Table.Head class="px-4 py-2">Created</Table.Head>
				</Table.Row>
			</Table.Header>
			<Table.Body>
				{#each versions as version (version.id)}
					<Table.Row class="last:border-0">
						<Table.Cell class="px-4 py-2 font-medium">{version.version}</Table.Cell>
						<Table.Cell class="px-4 py-2 text-xs text-muted-foreground">
							{formatGameVersions(version.game_versions)}
						</Table.Cell>
						<Table.Cell class="px-4 py-2 text-xs capitalize">
							{version.release_channel ?? '-'}
						</Table.Cell>
						<Table.Cell class="px-4 py-2">
							<TimeAgo timestamp={version.created_at} />
						</Table.Cell>
					</Table.Row>
				{/each}
			</Table.Body>
		</Table.Root>
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
		<CaptureGridAdmin {captures} alt={(c: CaptureWithContext) => c.scene_name ?? c.scene_id}>
		{#snippet footer(capture: CaptureWithContext)}
			<div class="p-2">
				<div class="flex items-center justify-between">
					<div class="text-sm font-medium">{capture.scene_name ?? capture.scene_id}</div>
					{#if capture.freshness !== 'fresh'}
						{@const colors = { stale: 'bg-warning/15 text-warning', superseded: 'bg-muted text-muted-foreground', fresh: '' }}
						<span class="rounded-full px-1.5 py-0.5 text-[10px] font-medium {colors[capture.freshness]}">
							{capture.freshness}
						</span>
					{/if}
				</div>
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
