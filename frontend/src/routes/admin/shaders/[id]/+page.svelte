<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import { api } from '$lib/api';
import type {
	CaptureWithContext,
	ShaderVersionDetail,
	ShaderVersionMetadata,
	ShaderVersionProfile,
	ShaderWithCaptures
} from '$lib/bindings';
import { createDataTable, DataTable } from '$lib/components/data-table';
import { ItemGrid } from '$lib/components/item-grid';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import {
	AdminBreadcrumb,
	AdminCaptureCard,
	AdminDetailField,
	createAdminAction
} from '$lib/components/admin';
import { Alert } from '$lib/components/ui/alert';
import { Badge } from '$lib/components/ui/badge';
import { Button } from '$lib/components/ui/button';
import * as FolderCard from '$lib/components/folder-card';
import * as Collapsible from '$lib/components/ui/collapsible';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import * as Dialog from '$lib/components/ui/dialog';
import * as Form from '$lib/components/ui/form';
import { Input } from '$lib/components/ui/input';
import { Textarea } from '$lib/components/ui/textarea';
import { freshnessColors } from '$lib/utils/status';
import {
	AlertTriangle,
	ChevronDown,
	CircleAlert,
	Clock,
	FlaskConical,
	Link,
	LoaderCircle,
	PackageCheck,
	PackageX,
	Pin,
	PinOff,
	RefreshCw,
	SkipForward,
	Trash2
} from '@lucide/svelte';
import * as Select from '$lib/components/ui/select';
import { Switch } from '$lib/components/ui/switch';
import { superForm } from 'sveltekit-superforms';
import { zod4Client } from 'sveltekit-superforms/adapters';
import { toast } from 'svelte-sonner';
import { shaderFormSchema } from './schema.js';
import type { PageData } from './$types';
import { createVersionColumns } from './version-columns.js';

const PLATFORM_URL_PATTERN =
	/^https?:\/\/(www\.)?(modrinth\.com\/shader\/|curseforge\.com\/minecraft\/shaders\/)/i;

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let shader: ShaderWithCaptures = $derived(data.shader);
let versions: ShaderVersionDetail[] = $derived(data.shader.versions);
let captures: CaptureWithContext[] = $derived(data.shader.captures);
let profiles: ShaderVersionProfile[] = $derived(data.shader.profiles);
let metadata: ShaderVersionMetadata | undefined = $derived(data.shader.metadata);

/** The effective (latest) version — matches what the backend returns profiles/metadata for */
let effectiveVersion = $derived(versions.length > 0 ? versions[0] : null);

let versionColumns = $derived(createVersionColumns(effectiveVersion?.id));
const versionTable = createDataTable<ShaderVersionDetail>({
	get data() {
		return versions;
	},
	get columns() {
		return versionColumns;
	},
	pageSize: false,
	selection: false
});

/** Humanize a camelCase or snake_case key into words */
function humanize(key: string): string {
	return key
		.replace(/([a-z])([A-Z])/g, '$1 $2')
		.replace(/_/g, ' ')
		.replace(/\b\w/g, (c) => c.toUpperCase());
}

// ── Superforms ──────────────────────────────────────────────
// Superforms manages its own reactivity after initialization; the initial
// data.form reference is intentional — updates come from server actions.
// svelte-ignore state_referenced_locally
const superform = superForm(data.form, {
	validators: zod4Client(shaderFormSchema),
	invalidateAll: 'force',
	resetForm: false,
	onUpdated({ form }) {
		if (form.message) {
			const msg = String(form.message);
			if (msg.toLowerCase().includes('success')) {
				toast.success(msg);
			} else {
				toast.error(msg);
			}
		}
	}
});
const { form: formData, enhance, tainted, submitting } = superform;

let isDirty = $derived($tainted != null && Object.values($tainted).some(Boolean));

// ── Non-form actions (delete, sync, link) ───────────────────
const deleteAction = createAdminAction({
	action: () => api.admin.deleteShader(shader.id),
	onSuccess: () => void goto('/admin/shaders'),
	setError: (msg) => toast.error(msg)
});

let syncing = $state(false);
let showDeleteConfirm = $state(false);
let linkDialogOpen = $state(false);
let linkUrl = $state('');
let linkError = $state<string | null>(null);
let linking = $state(false);

let hasLinkedPlatform = $derived(!!shader.modrinth_id || !!shader.curseforge_id);
let canLinkMorePlatforms = $derived(!shader.modrinth_id || !shader.curseforge_id);

/** Compute sync staleness: how old the last sync is */
let syncAge = $derived.by(() => {
	if (!shader.last_synced_at) return null;
	const ms = Date.now() - new Date(shader.last_synced_at).getTime();
	return {
		hours: ms / (1000 * 60 * 60),
		days: ms / (1000 * 60 * 60 * 24)
	};
});

let syncStatus = $derived.by<'never' | 'fresh' | 'stale' | 'very-stale'>(() => {
	if (!hasLinkedPlatform) return 'never';
	if (!syncAge) return 'never';
	if (syncAge.days > 7) return 'very-stale';
	if (syncAge.days > 1) return 'stale';
	return 'fresh';
});

async function handleSync() {
	syncing = true;

	try {
		const result = await api.admin.syncShader(shader.id);
		result.match({
			Ok: () => {
				toast.success('Shader synced successfully from upstream.');
				void invalidateAll();
			},
			Err: (err) => toast.error(err.message)
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
</script>

<svelte:head><title>{shader.name} - Glint</title></svelte:head>

<div class="space-y-6">
    <!-- Header -->
    <AdminBreadcrumb
        segments={[{ label: 'Shaders', href: '/admin/shaders' }, { label: shader.name }]}
    >
        {#snippet trailing()}
            {#if shader.icon_url}
                <img
                    src={shader.icon_url}
                    alt="{shader.name} icon"
                    class="ml-2 h-5 w-5 rounded"
                />
            {/if}
        {/snippet}
    </AdminBreadcrumb>

    <FolderCard.Root value="sync">
        {#snippet tabs()}
            <FolderCard.Tab value="sync">Sync & Editing</FolderCard.Tab>
            <FolderCard.Tab value="details">Details & Versions</FolderCard.Tab>
            <FolderCard.Tab value="captures"
                >Captures ({captures.length})</FolderCard.Tab
            >
        {/snippet}

        {#snippet trailing()}
            <Button
                variant="destructive"
                size="sm"
                onclick={() => (showDeleteConfirm = true)}
                disabled={deleteAction.loading}
            >
                <Trash2 class="mr-2 h-4 w-4" />
                Delete
            </Button>
        {/snippet}

        <!-- Tab 1: Sync & Editing -->
        <FolderCard.Content value="sync">
            <div class="space-y-6">
                <!-- Upstream Sync (not part of the form) -->
                {#if hasLinkedPlatform}
                    <div class="space-y-1">
                        <div class="flex items-center justify-between">
                            <div class="space-y-1">
                                <h3 class="text-sm font-medium">
                                    Upstream Sync
                                </h3>
                                <div
                                    class="flex items-center gap-2 text-sm text-muted-foreground"
                                >
                                    {#if syncStatus === "never"}
                                        <Badge
                                            variant="outline"
                                            class="text-warning border-warning/30"
                                            >Never synced</Badge
                                        >
                                    {:else if syncStatus === "very-stale"}
                                        <Badge
                                            variant="outline"
                                            class="text-destructive border-destructive/30"
                                            >Stale</Badge
                                        >
                                        <span
                                            >Last synced <TimeAgo
                                                timestamp={shader.last_synced_at!}
                                            /></span
                                        >
                                    {:else if syncStatus === "stale"}
                                        <Badge
                                            variant="outline"
                                            class="text-warning border-warning/30"
                                            >Due for sync</Badge
                                        >
                                        <span
                                            >Last synced <TimeAgo
                                                timestamp={shader.last_synced_at!}
                                            /></span
                                        >
                                    {:else}
                                        <Badge
                                            variant="outline"
                                            class="text-success border-success/30"
                                            >Synced</Badge
                                        >
                                        <span
                                            >Last synced <TimeAgo
                                                timestamp={shader.last_synced_at!}
                                            /></span
                                        >
                                    {/if}
                                </div>
                                <div
                                    class="flex items-center gap-3 text-xs text-muted-foreground"
                                >
                                    {#if shader.modrinth_id}
                                        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
                                        <a
                                            href="https://modrinth.com/shader/{shader.modrinth_id}"
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            class="hover:text-foreground"
                                            >Modrinth</a
                                        >
                                    {/if}
                                    {#if shader.curseforge_id}
                                        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
                                        <a
                                            href="https://www.curseforge.com/minecraft/shaders/{shader.curseforge_id}"
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            class="hover:text-foreground"
                                            >CurseForge</a
                                        >
                                    {/if}
                                </div>
                            </div>
                            <div class="flex items-center gap-2">
                                {#if canLinkMorePlatforms}
                                    <Dialog.Root
                                        open={linkDialogOpen}
                                        onOpenChange={handleLinkDialogChange}
                                    >
                                        <Dialog.Trigger>
                                            {#snippet child({ props })}
                                                <Button
                                                    variant="outline"
                                                    size="sm"
                                                    {...props}
                                                >
                                                    <Link
                                                        class="mr-2 h-4 w-4"
                                                    />
                                                    Link Platform
                                                </Button>
                                            {/snippet}
                                        </Dialog.Trigger>
                                        <Dialog.Content class="sm:max-w-md">
                                            <Dialog.Header>
                                                <Dialog.Title
                                                    >Link Platform</Dialog.Title
                                                >
                                                <Dialog.Description>
                                                    Link an additional platform
                                                    ({shader.modrinth_id
                                                        ? "CurseForge"
                                                        : "Modrinth"}) to this
                                                    shader.
                                                </Dialog.Description>
                                            </Dialog.Header>

                                            {#if linkError}
                                                <Alert variant="destructive">
                                                    <CircleAlert
                                                        class="h-4 w-4"
                                                    />
                                                    <div>{linkError}</div>
                                                </Alert>
                                            {/if}

                                            <div class="grid gap-4 py-4">
                                                <div class="grid gap-2">
                                                    <label for="link-url" class="text-sm font-medium">Platform URL</label>
                                                    <Input
                                                        id="link-url"
                                                        placeholder={shader.modrinth_id
                                                            ? "https://www.curseforge.com/minecraft/shaders/..."
                                                            : "https://modrinth.com/shader/..."}
                                                        bind:value={linkUrl}
                                                        onkeydown={handleLinkKeydown}
                                                        disabled={linking}
                                                    />
                                                    <p
                                                        class="text-xs text-muted-foreground"
                                                    >
                                                        Paste the {shader.modrinth_id
                                                            ? "CurseForge"
                                                            : "Modrinth"} URL for
                                                        this shader
                                                    </p>
                                                </div>
                                            </div>

                                            <Dialog.Footer>
                                                <Button
                                                    variant="outline"
                                                    onclick={() =>
                                                        (linkDialogOpen = false)}
                                                    >Cancel</Button
                                                >
                                                <Button
                                                    onclick={handleLink}
                                                    disabled={linking ||
                                                        !linkUrl.trim()}
                                                >
                                                    {#if linking}
                                                        <LoaderCircle
                                                            class="mr-2 h-4 w-4 animate-spin"
                                                        />
                                                    {:else}
                                                        <Link
                                                            class="mr-2 h-4 w-4"
                                                        />
                                                    {/if}
                                                    Link
                                                </Button>
                                            </Dialog.Footer>
                                        </Dialog.Content>
                                    </Dialog.Root>
                                {/if}
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onclick={handleSync}
                                    disabled={syncing}
                                >
                                    {#if syncing}
                                        <LoaderCircle
                                            class="mr-2 h-4 w-4 animate-spin"
                                        />
                                    {:else}
                                        <RefreshCw class="mr-2 h-4 w-4" />
                                    {/if}
                                    Sync Now
                                </Button>
                            </div>
                        </div>
                    </div>
                {/if}

                <!-- Edit Form -->
                <form method="POST" use:enhance class="space-y-6">
                    <!-- Capture Controls -->
                    <div class="space-y-4">
                        <h3 class="text-sm font-medium">Capture Controls</h3>

                        <Form.Field form={superform} name="capture_enabled">
                            <Form.Control>
                                {#snippet children({ props })}
                                    <div class="flex items-center justify-between rounded-lg border px-4 py-3">
                                        <div class="space-y-0.5">
                                            <Form.Label>Capture Enabled</Form.Label>
                                            <Form.Description>
                                                When disabled, no new captures will be scheduled for this shader.
                                            </Form.Description>
                                        </div>
                                        <Switch
                                            {...props}
                                            bind:checked={$formData.capture_enabled}
                                        />
                                    </div>
                                {/snippet}
                            </Form.Control>
                        </Form.Field>

                        <Form.Field form={superform} name="preferred_version_id">
                            <Form.Control>
                                {#snippet children({ props })}
                                    <div class="space-y-2 rounded-lg border px-4 py-3">
                                        <div class="space-y-0.5">
                                            <Form.Label>Preferred Version</Form.Label>
                                            <Form.Description>
                                                Pin a specific version for captures and as the default on the public page. When unset, the latest version is used.
                                            </Form.Description>
                                        </div>
                                        <!-- eslint-disable @typescript-eslint/no-unsafe-member-access -- formsnap control props -->
                                        <Select.Root
                                            type="single"
                                            value={$formData.preferred_version_id ?? ''}
                                            onValueChange={(v: string) => {
                                                $formData.preferred_version_id = v === '' ? null : v;
                                            }}
                                            name={props.name as string}
                                        >
                                        <!-- eslint-enable @typescript-eslint/no-unsafe-member-access -->
                                            <Select.Trigger size="sm" class="w-full">
                                                {#if $formData.preferred_version_id}
                                                    {@const pinned = versions.find((v) => v.id === $formData.preferred_version_id)}
                                                    <Pin class="mr-1.5 h-3.5 w-3.5 text-info" />
                                                    {pinned ? pinned.version : $formData.preferred_version_id}
                                                {:else}
                                                    <span class="text-muted-foreground">Auto (latest version)</span>
                                                {/if}
                                            </Select.Trigger>
                                            <Select.Content>
                                                <Select.Item value="">
                                                    <span class="flex items-center gap-2">
                                                        <PinOff class="h-3.5 w-3.5 text-muted-foreground" />
                                                        Auto (latest version)
                                                    </span>
                                                </Select.Item>
                                                {#each versions as version, i (version.id)}
                                                    <Select.Item value={version.id}>
                                                        <span class="flex items-center gap-2">
                                                            {version.version}
                                                            {#if i === 0}
                                                                <span class="rounded bg-info/15 px-1.5 py-0.5 text-[10px] font-semibold leading-none text-info">
                                                                    Latest
                                                                </span>
                                                            {/if}
                                                            <span class="ml-auto text-xs text-muted-foreground">
                                                                {version.capture_count} captures
                                                            </span>
                                                        </span>
                                                    </Select.Item>
                                                {/each}
                                            </Select.Content>
                                        </Select.Root>
                                    </div>
                                {/snippet}
                            </Form.Control>
                        </Form.Field>
                    </div>

                    <!-- Edit Fields -->
                    <div class="space-y-4">
                        <Form.Field form={superform} name="name">
                            <Form.Control>
                                {#snippet children({ props })}
                                    <Form.Label>Name</Form.Label>
                                    <Input {...props} bind:value={$formData.name} />
                                {/snippet}
                            </Form.Control>
                            <Form.FieldErrors />
                        </Form.Field>

                        <Form.Field form={superform} name="description">
                            <Form.Control>
                                {#snippet children({ props })}
                                    <Form.Label>Description</Form.Label>
                                    <Textarea
                                        {...props}
                                        bind:value={$formData.description}
                                        rows={3}
                                    />
                                {/snippet}
                            </Form.Control>
                            <Form.FieldErrors />
                        </Form.Field>

                        <Form.Field form={superform} name="modrinth_id">
                            <Form.Control>
                                {#snippet children({ props })}
                                    <Form.Label>Modrinth ID</Form.Label>
                                    <Input
                                        {...props}
                                        bind:value={$formData.modrinth_id}
                                        placeholder="e.g., abc123"
                                    />
                                {/snippet}
                            </Form.Control>
                            <Form.FieldErrors />
                        </Form.Field>

                        <Form.Field form={superform} name="curseforge_id">
                            <Form.Control>
                                {#snippet children({ props })}
                                    <Form.Label>CurseForge ID</Form.Label>
                                    <Input
                                        {...props}
                                        bind:value={$formData.curseforge_id}
                                        placeholder="e.g., 123456"
                                    />
                                {/snippet}
                            </Form.Control>
                            <Form.FieldErrors />
                        </Form.Field>

                        <Form.Field form={superform} name="website_url">
                            <Form.Control>
                                {#snippet children({ props })}
                                    <Form.Label>Website URL</Form.Label>
                                    <Input
                                        {...props}
                                        bind:value={$formData.website_url}
                                        placeholder="e.g., https://example.com"
                                    />
                                {/snippet}
                            </Form.Control>
                            <Form.FieldErrors />
                        </Form.Field>
                    </div>

                    <!-- Save / Reset -->
                    <div class="flex justify-end gap-2">
                        <Button
                            type="button"
                            variant="outline"
                            disabled={!isDirty || $submitting}
                            onclick={() => superform.reset()}
                        >
                            Reset
                        </Button>
                        <Form.Button disabled={!isDirty || $submitting}>
                            {#if $submitting}
                                <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
                            {/if}
                            {$submitting ? 'Saving...' : 'Save Changes'}
                        </Form.Button>
                    </div>
                </form>
            </div>
        </FolderCard.Content>

        <!-- Tab 2: Details & Versions -->
        <FolderCard.Content value="details">
            <div class="space-y-6">
                <!-- Metadata -->
                <dl class="flex flex-wrap gap-x-16 gap-y-4 text-sm">
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

                <!-- Versions -->
                <div class="space-y-3">
                    <h3 class="text-sm font-medium">
                        Versions ({versions.length})
                    </h3>
                    {#if versions.length === 0}
                        <p class="text-sm text-muted-foreground">
                            No versions yet.
                        </p>
                    {:else}
                        <DataTable table={versionTable} />
                    {/if}
                </div>

                <!-- Extraction Data -->
                {#if effectiveVersion}
                    {@const estatus = effectiveVersion.extraction_status}
                    {#if estatus === "completed" && (profiles.length > 0 || metadata)}
                        <div class="space-y-4">
                            <div class="flex items-center gap-3">
                                <FlaskConical
                                    class="h-5 w-5 text-muted-foreground"
                                />
                                <h3 class="text-sm font-medium">
                                    Extraction Data
                                </h3>
                                <Badge variant="outline" class="text-xs"
                                    >{effectiveVersion.version}</Badge
                                >
                            </div>

                            <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
                                <!-- Profiles -->
                                {#if profiles.length > 0}
                                    <div class="rounded-lg border">
                                        <div
                                            class="flex items-center justify-between border-b px-4 py-3"
                                        >
                                            <h4 class="text-sm font-medium">
                                                Profiles
                                            </h4>
                                            <Badge
                                                variant="secondary"
                                                class="text-xs"
                                                >{profiles.length} profile{profiles.length !==
                                                1
                                                    ? "s"
                                                    : ""}</Badge
                                            >
                                        </div>
                                        <div class="divide-y">
                                            {#each profiles as profile (profile.id)}
                                                {@const optionEntries =
                                                    Object.entries(
                                                        profile.options ?? {},
                                                    )}
                                                <Collapsible.Root class="group">
                                                    <Collapsible.Trigger
                                                        class="flex w-full items-center gap-3 px-4 py-2.5 text-left hover:bg-muted/50"
                                                    >
                                                        <span
                                                            class="shrink-0 text-xs text-muted-foreground tabular-nums"
                                                            >#{profile.sort_order +
                                                                1}</span
                                                        >
                                                        <div
                                                            class="min-w-0 flex-1"
                                                        >
                                                            <span
                                                                class="font-medium text-sm"
                                                                >{profile.display_name}</span
                                                            >
                                                        </div>
                                                        <Badge
                                                            variant="outline"
                                                            class="shrink-0 text-[10px]"
                                                            >{optionEntries.length}
                                                            opt{optionEntries.length !==
                                                            1
                                                                ? "s"
                                                                : ""}</Badge
                                                        >
                                                        <ChevronDown
                                                            class="h-4 w-4 shrink-0 text-muted-foreground transition-transform group-data-[state=open]:rotate-180"
                                                        />
                                                    </Collapsible.Trigger>
                                                    <Collapsible.Content>
                                                        <div
                                                            class="border-t px-4 py-3"
                                                        >
                                                            {#if profile.description}
                                                                <p
                                                                    class="mb-3 text-sm text-muted-foreground"
                                                                >
                                                                    {profile.description}
                                                                </p>
                                                            {/if}
                                                            <dl class="mb-3 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-xs">
                                                                <dt class="text-muted-foreground">Name</dt>
                                                                <dd class="font-mono">{profile.name}</dd>
                                                                {#if profile.label}
                                                                    <dt class="text-muted-foreground">Label</dt>
                                                                    <dd class="font-mono">{profile.label}</dd>
                                                                {/if}
                                                                <dt class="text-muted-foreground">Display Name</dt>
                                                                <dd>{profile.display_name}</dd>
                                                            </dl>
                                                            {#if optionEntries.length > 0}
                                                                <div
                                                                    class="rounded border"
                                                                >
                                                                    {#each optionEntries.slice(0, 20) as [key, val], i (key)}
                                                                        <div
                                                                            class="flex items-center justify-between gap-4 px-3 py-1.5 text-xs {i %
                                                                                2 ===
                                                                            0
                                                                                ? ''
                                                                                : 'bg-muted/50'}"
                                                                        >
                                                                            <span
                                                                                class="min-w-0 truncate font-mono text-muted-foreground"
                                                                                >{key}</span
                                                                            >
                                                                            <span
                                                                                class="shrink-0 font-mono"
                                                                                >{val}</span
                                                                            >
                                                                        </div>
                                                                    {/each}
                                                                    {#if optionEntries.length > 20}
                                                                        <details
                                                                            class="border-t"
                                                                        >
                                                                            <summary
                                                                                class="cursor-pointer px-3 py-1.5 text-xs text-muted-foreground hover:text-foreground"
                                                                            >
                                                                                Show
                                                                                all
                                                                                {optionEntries.length}
                                                                                options
                                                                            </summary>
                                                                            {#each optionEntries.slice(20) as [key, val], i (key)}
                                                                                <div
                                                                                    class="flex items-center justify-between gap-4 px-3 py-1.5 text-xs {(i +
                                                                                        20) %
                                                                                        2 ===
                                                                                    0
                                                                                        ? ''
                                                                                        : 'bg-muted/50'}"
                                                                                >
                                                                                    <span
                                                                                        class="min-w-0 truncate font-mono text-muted-foreground"
                                                                                        >{key}</span
                                                                                    >
                                                                                    <span
                                                                                        class="shrink-0 font-mono"
                                                                                        >{val}</span
                                                                                    >
                                                                                </div>
                                                                            {/each}
                                                                        </details>
                                                                    {/if}
                                                                </div>
                                                            {:else}
                                                                <p
                                                                    class="text-xs text-muted-foreground"
                                                                >
                                                                    No option
                                                                    overrides
                                                                    (inherits
                                                                    defaults)
                                                                </p>
                                                            {/if}
                                                        </div>
                                                    </Collapsible.Content>
                                                </Collapsible.Root>
                                            {/each}
                                        </div>
                                    </div>
                                {/if}

                                <!-- Metadata -->
                                {#if metadata}
                                    <div class="rounded-lg border">
                                        <div
                                            class="flex items-center justify-between border-b px-4 py-3"
                                        >
                                            <h4 class="text-sm font-medium">
                                                Metadata
                                            </h4>
                                            <span
                                                class="text-xs text-muted-foreground"
                                                ><TimeAgo
                                                    timestamp={metadata.extracted_at}
                                                /></span
                                            >
                                        </div>
                                        <div class="space-y-4 px-4 py-3">
                                            {#if metadata.pipeline_features && Object.keys(metadata.pipeline_features).length > 0}
                                                <div>
                                                    <div
                                                        class="mb-1.5 text-xs font-medium uppercase tracking-wide text-muted-foreground"
                                                    >
                                                        Pipeline Features
                                                    </div>
                                                    <div
                                                        class="flex flex-wrap gap-1.5"
                                                    >
                                                        {#each Object.entries(metadata.pipeline_features) as [key, val] (key)}
                                                            {#if val === true}
                                                                <Badge
                                                                    variant="default"
                                                                    class="text-[11px] bg-success hover:bg-success"
                                                                    >{humanize(
                                                                        key,
                                                                    )}</Badge
                                                                >
                                                            {:else if val === false}
                                                                <Badge
                                                                    variant="outline"
                                                                    class="text-[11px] line-through opacity-50"
                                                                    >{humanize(
                                                                        key,
                                                                    )}</Badge
                                                                >
                                                            {:else}
                                                                <Badge
                                                                    variant="secondary"
                                                                    class="text-[11px]"
                                                                    >{humanize(
                                                                        key,
                                                                    )}: {val}</Badge
                                                                >
                                                            {/if}
                                                        {/each}
                                                    </div>
                                                </div>
                                            {/if}

                                            {#if (metadata.iris_features_required?.length ?? 0) > 0 || (metadata.iris_features_optional?.length ?? 0) > 0}
                                                <div>
                                                    <div
                                                        class="mb-1.5 text-xs font-medium uppercase tracking-wide text-muted-foreground"
                                                    >
                                                        Iris Features
                                                    </div>
                                                    {#if metadata.iris_features_required && metadata.iris_features_required.length > 0}
                                                        <div class="mb-1">
                                                            <span
                                                                class="mr-1.5 text-[10px] text-muted-foreground"
                                                                >Required:</span
                                                            >
                                                            {#each metadata.iris_features_required as feat (feat)}
                                                                <Badge
                                                                    variant="default"
                                                                    class="mr-1 mb-1 text-[11px]"
                                                                    >{feat}</Badge
                                                                >
                                                            {/each}
                                                        </div>
                                                    {/if}
                                                    {#if metadata.iris_features_optional && metadata.iris_features_optional.length > 0}
                                                        <div>
                                                            <span
                                                                class="mr-1.5 text-[10px] text-muted-foreground"
                                                                >Optional:</span
                                                            >
                                                            {#each metadata.iris_features_optional as feat (feat)}
                                                                <Badge
                                                                    variant="outline"
                                                                    class="mr-1 mb-1 text-[11px]"
                                                                    >{feat}</Badge
                                                                >
                                                            {/each}
                                                        </div>
                                                    {/if}
                                                </div>
                                            {/if}

                                            {#if metadata.dimension_support && metadata.dimension_support.length > 0}
                                                <div>
                                                    <div
                                                        class="mb-1.5 text-xs font-medium uppercase tracking-wide text-muted-foreground"
                                                    >
                                                        Dimension Support
                                                    </div>
                                                    <div
                                                        class="flex flex-wrap gap-1.5"
                                                    >
                                                        {#each metadata.dimension_support as dim (dim)}
                                                            <Badge
                                                                variant="secondary"
                                                                class="text-[11px]"
                                                                >{humanize(
                                                                    dim.replace(
                                                                        "the_",
                                                                        "",
                                                                    ),
                                                                )}</Badge
                                                            >
                                                        {/each}
                                                    </div>
                                                </div>
                                            {/if}

                                            {#if metadata.has_custom_textures != null}
                                                <div>
                                                    <div
                                                        class="mb-1.5 text-xs font-medium uppercase tracking-wide text-muted-foreground"
                                                    >
                                                        Custom Textures
                                                    </div>
                                                    <div
                                                        class="flex items-center gap-1.5 text-sm"
                                                    >
                                                        {#if metadata.has_custom_textures}
                                                            <PackageCheck
                                                                class="h-4 w-4 text-success"
                                                            />
                                                            <span
                                                                >Uses custom
                                                                textures</span
                                                            >
                                                        {:else}
                                                            <PackageX
                                                                class="h-4 w-4 text-muted-foreground"
                                                            />
                                                            <span
                                                                class="text-muted-foreground"
                                                                >No custom
                                                                textures</span
                                                            >
                                                        {/if}
                                                    </div>
                                                </div>
                                            {/if}

                                            {#if metadata.file_paths && metadata.file_paths.length > 0}
                                                <Collapsible.Root>
                                                    <Collapsible.Trigger
                                                        class="flex w-full items-center gap-2 text-left"
                                                    >
                                                        <div
                                                            class="text-xs font-medium uppercase tracking-wide text-muted-foreground"
                                                        >
                                                            File Paths
                                                        </div>
                                                        <Badge
                                                            variant="secondary"
                                                            class="text-[10px]"
                                                            >{metadata
                                                                .file_paths
                                                                .length} files</Badge
                                                        >
                                                        <ChevronDown
                                                            class="ml-auto h-3.5 w-3.5 text-muted-foreground transition-transform [[data-state=open]>&]:rotate-180"
                                                        />
                                                    </Collapsible.Trigger>
                                                    <Collapsible.Content>
                                                        <div
                                                            class="mt-2 max-h-64 overflow-y-auto rounded border bg-muted/30 p-2"
                                                        >
                                                            {#each metadata.file_paths as path, i (path)}
                                                                <div
                                                                    class="px-1 py-0.5 font-mono text-[11px] {i %
                                                                        2 ===
                                                                    0
                                                                        ? ''
                                                                        : 'bg-muted/50'}"
                                                                >
                                                                    {path}
                                                                </div>
                                                            {/each}
                                                        </div>
                                                    </Collapsible.Content>
                                                </Collapsible.Root>
                                            {/if}

                                            {#if metadata.settings_screen && metadata.settings_screen.length > 0}
                                                <Collapsible.Root>
                                                    <Collapsible.Trigger
                                                        class="flex w-full items-center gap-2 text-left"
                                                    >
                                                        <div
                                                            class="text-xs font-medium uppercase tracking-wide text-muted-foreground"
                                                        >
                                                            Settings Screen
                                                            Layout
                                                        </div>
                                                        <Badge
                                                            variant="secondary"
                                                            class="text-[10px]"
                                                            >{metadata
                                                                .settings_screen
                                                                .length} entries</Badge
                                                        >
                                                        <ChevronDown
                                                            class="ml-auto h-3.5 w-3.5 text-muted-foreground transition-transform [[data-state=open]>&]:rotate-180"
                                                        />
                                                    </Collapsible.Trigger>
                                                    <Collapsible.Content>
                                                        <pre
                                                            class="mt-2 max-h-80 overflow-y-auto rounded-md bg-muted p-3 font-mono text-xs">{JSON.stringify(
                                                                metadata.settings_screen,
                                                                null,
                                                                2,
                                                            )}</pre>
                                                    </Collapsible.Content>
                                                </Collapsible.Root>
                                            {/if}
                                        </div>
                                    </div>
                                {/if}
                            </div>
                        </div>
                    {:else if estatus === "failed"}
                        <Alert variant="destructive">
                            <AlertTriangle class="h-4 w-4" />
                            <div>
                                Extraction failed for v{effectiveVersion.version}{effectiveVersion.extraction_error
                                    ? `: ${effectiveVersion.extraction_error}`
                                    : ""}
                            </div>
                        </Alert>
                    {:else if estatus === "pending"}
                        <div
                            class="flex items-center gap-2 rounded-lg border bg-muted/30 px-4 py-3 text-sm text-muted-foreground"
                        >
                            <Clock class="h-4 w-4" />
                            Extraction pending for v{effectiveVersion.version}
                        </div>
                    {:else if estatus === "skipped"}
                        <div
                            class="flex items-center gap-2 rounded-lg border bg-muted/30 px-4 py-3 text-sm text-muted-foreground"
                        >
                            <SkipForward class="h-4 w-4" />
                            Extraction skipped for v{effectiveVersion.version}
                        </div>
                    {/if}
                {/if}
            </div>
        </FolderCard.Content>

        <!-- Tab 3: Captures -->
        <FolderCard.Content value="captures">
            {#if captures.length === 0}
                <p class="text-sm text-muted-foreground">No captures yet.</p>
            {:else}
                <div class="space-y-3">
                    <div class="flex justify-end">
                        <a
                            href="/admin/captures?shader={shader.slug}"
                            class="text-sm text-primary hover:underline"
                        >
                            View all
                        </a>
                    </div>
                    <ItemGrid items={captures} key={(c: CaptureWithContext) => c.id} size="small">
                        {#snippet card(capture: CaptureWithContext)}
                            <AdminCaptureCard {capture} alt={capture.scene_name ?? capture.scene_id}>
                                <div class="p-2">
                                    <div class="flex items-center justify-between">
                                        <div class="text-sm font-medium">
                                            {capture.scene_name ?? capture.scene_id}
                                        </div>
                                        {#if capture.freshness !== "fresh"}
                                            <span
                                                class="rounded-full px-1.5 py-0.5 text-[10px] font-medium {freshnessColors[
                                                    capture.freshness
                                                ]}"
                                            >
                                                {capture.freshness}
                                            </span>
                                        {/if}
                                    </div>
                                    <div class="text-xs text-muted-foreground">
                                        {capture.shader_version}
                                        {#if capture.profile_display_name}
                                            &middot; {capture.profile_display_name}
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
    bind:open={showDeleteConfirm}
    title="Delete Shader"
    description={`Delete shader "${shader.name}"? This cannot be undone.`}
    confirmLabel="Delete"
    onConfirm={deleteAction.execute}
/>
