<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import { api } from '$lib/api';
import type { UpdateShaderRequest } from '$lib/api/endpoints/admin';
import type { CaptureWithContext, ShaderVersionDetail, ShaderWithCaptures } from '$lib/bindings';
import CaptureGridAdmin from '$lib/components/CaptureGridAdmin.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { AdminDetailField, AdminDetailHeader } from '$lib/components/admin';
import { Alert } from '$lib/components/ui/alert';
import { Badge } from '$lib/components/ui/badge';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import * as Dialog from '$lib/components/ui/dialog';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import * as Table from '$lib/components/ui/table';
import { Textarea } from '$lib/components/ui/textarea';
import { formatGameVersions } from '$lib/utils/display';
import { CircleAlert, Link, LoaderCircle, RefreshCw, Trash2 } from '@lucide/svelte';
import { untrack } from 'svelte';
import type { PageData } from './$types';

const PLATFORM_URL_PATTERN =
	/^https?:\/\/(www\.)?(modrinth\.com\/shader\/|curseforge\.com\/minecraft\/shaders\/)/i;

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let shader: ShaderWithCaptures = $derived(data.shader);
let versions: ShaderVersionDetail[] = $derived(data.shader.versions);
let captures: CaptureWithContext[] = $derived(data.shader.captures);

let saving = $state(false);
let syncing = $state(false);
let actionLoading = $state(false);
let error = $state<string | null>(null);
let syncSuccess = $state(false);
let showDeleteConfirm = $state(false);
let linkDialogOpen = $state(false);
let linkUrl = $state('');
let linkError = $state<string | null>(null);
let linking = $state(false);

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

let hasLinkedPlatform = $derived(!!shader.modrinth_id || !!shader.curseforge_id);
let canLinkMorePlatforms = $derived(!shader.modrinth_id || !shader.curseforge_id);

/** Compute sync staleness: how old the last sync is */
let syncAge = $derived.by(() => {
	if (!shader.last_synced_at) return null;
	const ms = Date.now() - new Date(shader.last_synced_at).getTime();
	return { hours: ms / (1000 * 60 * 60), days: ms / (1000 * 60 * 60 * 24) };
});

let syncStatus = $derived.by<'never' | 'fresh' | 'stale' | 'very-stale'>(() => {
	if (!hasLinkedPlatform) return 'never';
	if (!syncAge) return 'never';
	if (syncAge.days > 7) return 'very-stale';
	if (syncAge.days > 1) return 'stale';
	return 'fresh';
});

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

async function handleSync() {
	syncing = true;
	error = null;
	syncSuccess = false;

	try {
		const result = await api.admin.syncShader(shader.id);
		result.match({
			Ok: () => {
				syncSuccess = true;
				void invalidateAll();
				setTimeout(() => (syncSuccess = false), 3000);
			},
			Err: (err) => {
				error = err.message;
			}
		});
	} finally {
		syncing = false;
	}
}

function isValidPlatformUrl(input: string): boolean {
	return PLATFORM_URL_PATTERN.test(input.trim());
}

async function handleLink() {
	const trimmed = linkUrl.trim();
	if (!trimmed) return;

	if (!isValidPlatformUrl(trimmed)) {
		linkError = 'Please enter a valid Modrinth or CurseForge shader URL.';
		return;
	}

	linking = true;
	linkError = null;

	try {
		const result = await api.admin.linkShaderPlatform(shader.id, trimmed);
		result.match({
			Ok: () => {
				linkDialogOpen = false;
				linkUrl = '';
				linkError = null;
				void invalidateAll();
			},
			Err: (err) => {
				if (err.statusCode === 409) {
					linkError = 'This platform is already linked to this shader.';
				} else {
					linkError = err.message;
				}
			}
		});
	} finally {
		linking = false;
	}
}

function handleLinkDialogChange(newOpen: boolean) {
	linkDialogOpen = newOpen;
	if (!newOpen) {
		linkUrl = '';
		linkError = null;
		linking = false;
	}
}

function handleLinkKeydown(e: KeyboardEvent) {
	if (e.key === 'Enter' && !linking) {
		void handleLink();
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

	{#if syncSuccess}
		<Alert>Shader synced successfully from upstream.</Alert>
	{/if}

	<!-- Upstream Sync Section -->
	{#if hasLinkedPlatform}
		<div class="rounded-lg border bg-card p-4">
			<div class="flex items-center justify-between">
				<div class="space-y-1">
					<h3 class="text-sm font-medium">Upstream Sync</h3>
					<div class="flex items-center gap-2 text-sm text-muted-foreground">
						{#if syncStatus === 'never'}
							<Badge variant="outline" class="text-warning border-warning/30">Never synced</Badge>
						{:else if syncStatus === 'very-stale'}
							<Badge variant="outline" class="text-destructive border-destructive/30">Stale</Badge>
							<span>Last synced <TimeAgo timestamp={shader.last_synced_at!} /></span>
						{:else if syncStatus === 'stale'}
							<Badge variant="outline" class="text-warning border-warning/30">Due for sync</Badge>
							<span>Last synced <TimeAgo timestamp={shader.last_synced_at!} /></span>
						{:else}
							<Badge variant="outline" class="text-green-600 border-green-600/30 dark:text-green-400 dark:border-green-400/30">Synced</Badge>
							<span>Last synced <TimeAgo timestamp={shader.last_synced_at!} /></span>
						{/if}
					</div>
					<div class="flex items-center gap-3 text-xs text-muted-foreground">
						{#if shader.modrinth_id}
							<a
								href="https://modrinth.com/shader/{shader.modrinth_id}"
								target="_blank"
								rel="noopener noreferrer"
								class="hover:text-foreground"
							>Modrinth</a>
						{/if}
						{#if shader.curseforge_id}
							<a
								href="https://www.curseforge.com/minecraft/shaders/{shader.curseforge_id}"
								target="_blank"
								rel="noopener noreferrer"
								class="hover:text-foreground"
							>CurseForge</a>
						{/if}
					</div>
				</div>
				<div class="flex items-center gap-2">
					{#if canLinkMorePlatforms}
						<Dialog.Root open={linkDialogOpen} onOpenChange={handleLinkDialogChange}>
							<Dialog.Trigger>
								{#snippet child({ props })}
									<Button variant="outline" size="sm" {...props}>
										<Link class="mr-2 h-4 w-4" />
										Link Platform
									</Button>
								{/snippet}
							</Dialog.Trigger>
							<Dialog.Content class="sm:max-w-md">
								<Dialog.Header>
									<Dialog.Title>Link Platform</Dialog.Title>
									<Dialog.Description>
										Link an additional platform ({shader.modrinth_id ? 'CurseForge' : 'Modrinth'}) to this shader.
									</Dialog.Description>
								</Dialog.Header>

								{#if linkError}
									<Alert variant="destructive">
										<CircleAlert class="h-4 w-4" />
										<div>{linkError}</div>
									</Alert>
								{/if}

								<div class="grid gap-4 py-4">
									<div class="grid gap-2">
										<Label for="link-url">Platform URL</Label>
										<Input
											id="link-url"
											placeholder={shader.modrinth_id
												? 'https://www.curseforge.com/minecraft/shaders/...'
												: 'https://modrinth.com/shader/...'}
											bind:value={linkUrl}
											onkeydown={handleLinkKeydown}
											disabled={linking}
										/>
										<p class="text-xs text-muted-foreground">
											Paste the {shader.modrinth_id ? 'CurseForge' : 'Modrinth'} URL for this shader
										</p>
									</div>
								</div>

								<Dialog.Footer>
									<Button variant="outline" onclick={() => (linkDialogOpen = false)}>Cancel</Button>
									<Button onclick={handleLink} disabled={linking || !linkUrl.trim()}>
										{#if linking}
											<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
										{:else}
											<Link class="mr-2 h-4 w-4" />
										{/if}
										Link
									</Button>
								</Dialog.Footer>
							</Dialog.Content>
						</Dialog.Root>
					{/if}
					<Button variant="outline" size="sm" onclick={handleSync} disabled={syncing}>
						{#if syncing}
							<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
						{:else}
							<RefreshCw class="mr-2 h-4 w-4" />
						{/if}
						Sync Now
					</Button>
				</div>
			</div>
		</div>
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
			{#if shader.upstream_updated_at}
				<AdminDetailField label="Upstream Updated">
					<TimeAgo timestamp={shader.upstream_updated_at} />
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
			<p class="text-sm text-foreground">No versions yet.</p>
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
		<p class="text-sm text-foreground">No captures yet.</p>
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
