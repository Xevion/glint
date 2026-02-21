<script lang="ts">
import { goto, invalidate } from '$app/navigation';
import CaptureCard from '$lib/components/CaptureCard.svelte';
import Breadcrumb from '$lib/components/Breadcrumb.svelte';
import DetailField from '$lib/components/DetailField.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { createAction } from '$lib/components/action-form.svelte';
import {
	Counter,
	DataView,
	Grid,
	Pagination,
	Table,
	Toolbar,
	ViewToggle,
	createCursorList
} from '$lib/components/data-view';
import {
	DateRangeFilter,
	FilterCombobox,
	NumericRangeFilter,
	PopoverMultiSelect
} from '$lib/components/filters';
import {
	AdminCapturesQuery,
	type AdminCaptureNode,
	captureFilterConfig,
	captureFreshnessOptions,
	captureStatusOptions,
	type CaptureFilters
} from '../../captures/queries';
import { columns as captureColumns } from '../../captures/columns';
import * as FolderCard from '$lib/components/folder-card';
import { Alert } from '$lib/components/ui/alert';
import { Badge } from '$lib/components/ui/badge';
import { Button } from '$lib/components/ui/button';
import * as Collapsible from '$lib/components/ui/collapsible';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import * as Dialog from '$lib/components/ui/dialog';
import * as Form from '$lib/components/ui/form';
import { Input } from '$lib/components/ui/input';
import * as Select from '$lib/components/ui/select';
import { Switch } from '$lib/components/ui/switch';
import { Textarea } from '$lib/components/ui/textarea';
import { type ResultOf, createGraphQLClient, graphql, mutation } from '$lib/graphql';
import { ageMs, formatCount } from '$lib/utils/format';
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
	RotateCcw,
	SkipForward,
	Trash2
} from '@lucide/svelte';
import { untrack } from 'svelte';
import { toast } from 'svelte-sonner';
import { defaults, superForm } from 'sveltekit-superforms';
import { zod4Client } from 'sveltekit-superforms/adapters';
import type { PageData } from './$types';
import type {
	ShaderDetailData,
	ShaderMetadata,
	ShaderProfile,
	ShaderVersionDetail
} from './queries';
import { shaderFormSchema } from './schema.js';
import { createVersionColumns } from './version-columns.js';

const UpdateShaderMutation = graphql(`
	mutation UpdateShader($id: ID!, $input: UpdateShaderInput!) {
		updateShader(id: $id, input: $input) {
			id
		}
	}
`);

const DeleteShaderMutation = graphql(`
	mutation DeleteShader($id: ID!) {
		deleteShader(id: $id)
	}
`);

const SyncShaderMutation = graphql(`
	mutation SyncShader($id: ID!) {
		syncShader(id: $id) {
			id
		}
	}
`);

const RestoreShaderMutation = graphql(`
	mutation RestoreShader($id: ID!) {
		restoreShader(id: $id) {
			id
		}
	}
`);

const PurgeShaderMutation = graphql(`
	mutation PurgeShader($id: ID!) {
		purgeShader(id: $id)
	}
`);

const LinkShaderPlatformMutation = graphql(`
	mutation LinkShaderPlatform($id: ID!, $url: String!) {
		linkShaderPlatform(id: $id, url: $url) {
			id
		}
	}
`);

const PLATFORM_URL_PATTERN =
	/^https?:\/\/(www\.)?(modrinth\.com\/shader\/|curseforge\.com\/minecraft\/shaders\/)/i;

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let shader: ShaderDetailData = $derived(data.shader);
let versions: ShaderVersionDetail[] = $derived(data.shader.versions.edges.map((e) => e.node));
let profiles: ShaderProfile[] = $derived(data.shader.profiles.edges.map((e) => e.node));
let metadata: ShaderMetadata | null = $derived(data.shader.metadata);

// Captures tab: cursor-paginated with forced shader filter
const captureList = createCursorList<AdminCaptureNode, CaptureFilters>({
	initial: () => data.captures,
	query: AdminCapturesQuery,
	extract: (d: ResultOf<typeof AdminCapturesQuery>) => d.adminCaptures,
	pageSize: 24,
	filters: captureFilterConfig,
	filterVariable: 'filters',
	fixedFilters: { shaderSlug: untrack(() => data.shader.slug) },
	viewMode: 'table'
});

const sceneOptions = $derived(data.sceneOptions);

/** The effective (latest) version — matches what the backend returns profiles/metadata for */
let effectiveVersion = $derived(versions.length > 0 ? versions[0] : null);

let versionColumns = $derived(createVersionColumns(effectiveVersion?.version.id));

/** Humanize a camelCase or snake_case key into words */
function humanize(key: string): string {
	return key
		.replace(/([a-z])([A-Z])/g, '$1 $2')
		.replace(/_/g, ' ')
		.replace(/\b\w/g, (c) => c.toUpperCase());
}

const client = createGraphQLClient();

const initialShader = untrack(() => shader);
const superform = superForm(
	defaults(
		{
			name: initialShader.name,
			description: initialShader.description ?? '',
			modrinth_id: initialShader.modrinthId ?? '',
			curseforge_id: initialShader.curseforgeId ?? '',
			website_url: initialShader.websiteUrl ?? '',
			capture_enabled: initialShader.captureEnabled,
			preferred_version_id: initialShader.preferredVersionId ?? null
		},
		zod4Client(shaderFormSchema)
	),
	{
		SPA: true,
		validators: zod4Client(shaderFormSchema),
		resetForm: false,
		onUpdate({ form }) {
			if (!form.valid) return;
			void (async () => {
				const result = await mutation(client, UpdateShaderMutation, {
					id: shader.id,
					input: {
						name: form.data.name,
						description: form.data.description || null,
						modrinthId: form.data.modrinth_id || null,
						curseforgeId: form.data.curseforge_id || null,
						websiteUrl: form.data.website_url || null,
						captureEnabled: form.data.capture_enabled,
						preferredVersionId: form.data.preferred_version_id
					}
				});
				result.match({
					Ok: () => {
						toast.success('Shader updated');
						void Promise.all([
							invalidate(`glint:admin:shader:${shader.id}`),
							invalidate('glint:shaders')
						]);
					},
					Err: (err) => toast.error(err.message)
				});
			})();
		}
	}
);
const { form: formData, tainted, submitting, enhance } = superform;

let isDirty = $derived($tainted != null && Object.values($tainted).some(Boolean));

const deleteAction = createAction({
	action: () => mutation(client, DeleteShaderMutation, { id: shader.id }),
	onSuccess: () => {
		toast.success('Shader moved to trash');
		void Promise.all([invalidate(`glint:admin:shader:${shader.id}`), invalidate('glint:shaders')]);
	},
	setError: (msg: string) => toast.error(msg)
});

let syncing = $state(false);
let showDeleteConfirm = $state(false);
let showPurgeConfirm = $state(false);
let restoring = $state(false);
let linkDialogOpen = $state(false);
let linkUrl = $state('');
let linkError = $state<string | null>(null);
let linking = $state(false);

let hasLinkedPlatform = $derived(!!shader.modrinthId || !!shader.curseforgeId);
let canLinkMorePlatforms = $derived(!shader.modrinthId || !shader.curseforgeId);

let syncAgeMs = $derived(shader.lastSyncedAt ? ageMs(shader.lastSyncedAt) : null);

let syncStatus = $derived.by<'never' | 'fresh' | 'stale' | 'very-stale'>(() => {
	if (!hasLinkedPlatform) return 'never';
	if (syncAgeMs == null) return 'never';
	const days = syncAgeMs / (1000 * 60 * 60 * 24);
	if (days > 7) return 'very-stale';
	if (days > 1) return 'stale';
	return 'fresh';
});

async function handleSync() {
	syncing = true;

	try {
		const result = await mutation(client, SyncShaderMutation, { id: shader.id });
		result.match({
			Ok: () => {
				toast.success('Shader synced successfully from upstream.');
				void Promise.all([invalidate(`glint:admin:shader:${shader.id}`), invalidate('glint:shaders')]);
			},
			Err: (err) => toast.error(err.message)
		});
	} finally {
		syncing = false;
	}
}

async function handleRestore() {
	restoring = true;
	try {
		const result = await mutation(client, RestoreShaderMutation, { id: shader.id });
		result.match({
			Ok: () => {
				toast.success('Shader restored');
				void Promise.all([invalidate(`glint:admin:shader:${shader.id}`), invalidate('glint:shaders')]);
			},
			Err: (err) => toast.error(err.message)
		});
	} finally {
		restoring = false;
	}
}

const purgeAction = createAction({
	action: () => mutation(client, PurgeShaderMutation, { id: shader.id }),
	onSuccess: () => {
		toast.success('Shader permanently deleted');
		void goto('/admin/shaders');
	},
	setError: (msg: string) => toast.error(msg)
});

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
		const result = await mutation(client, LinkShaderPlatformMutation, {
			id: shader.id,
			url: trimmed
		});
		result.match({
			Ok: () => {
				linkDialogOpen = false;
				linkUrl = '';
				linkError = null;
				void invalidate(`glint:admin:shader:${shader.id}`);
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
    <Breadcrumb
        segments={[{ label: 'Shaders', href: '/admin/shaders' }, { label: shader.name }]}
    >
        {#snippet trailing()}
            {#if shader.iconUrl}
                <img
                    src={shader.iconUrl}
                    alt="{shader.name} icon"
                    class="ml-2 h-5 w-5 rounded"
                />
            {/if}
        {/snippet}
    </Breadcrumb>

    {#if shader.deletedAt}
        <Alert variant="destructive">
            <Trash2 class="h-4 w-4" />
            <div class="flex items-center justify-between">
                <span>This shader was deleted <TimeAgo timestamp={shader.deletedAt} />.</span>
                <div class="flex gap-2">
                    <Button
                        variant="outline"
                        size="sm"
                        onclick={handleRestore}
                        disabled={restoring}
                    >
                        {#if restoring}
                            <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
                        {:else}
                            <RotateCcw class="mr-2 h-4 w-4" />
                        {/if}
                        Restore
                    </Button>
                    <Button
                        variant="destructive"
                        size="sm"
                        onclick={() => (showPurgeConfirm = true)}
                    >
                        <Trash2 class="mr-2 h-4 w-4" />
                        Permanently Delete
                    </Button>
                </div>
            </div>
        </Alert>
    {/if}

    <FolderCard.Root value="sync">
        {#snippet tabs()}
            <FolderCard.Tab value="sync">Sync & Editing</FolderCard.Tab>
            <FolderCard.Tab value="details">Details & Versions</FolderCard.Tab>
            <FolderCard.Tab value="captures"
                >Captures{captureList.totalCount != null ? ` (${captureList.totalCount})` : ''}</FolderCard.Tab
            >
        {/snippet}

        {#snippet trailing()}
            {#if !shader.deletedAt}
                <Button
                    variant="destructive"
                    size="sm"
                    onclick={() => (showDeleteConfirm = true)}
                    disabled={deleteAction.loading}
                >
                    <Trash2 class="mr-2 h-4 w-4" />
                    Delete
                </Button>
            {/if}
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
                                                timestamp={shader.lastSyncedAt!}
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
                                                timestamp={shader.lastSyncedAt!}
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
                                                timestamp={shader.lastSyncedAt!}
                                            /></span
                                        >
                                    {/if}
                                </div>
                                <div
                                    class="flex items-center gap-3 text-xs text-muted-foreground"
                                >
                                    {#if shader.modrinthId}
                                        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
                                        <a
                                            href="https://modrinth.com/shader/{shader.modrinthId}"
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            class="hover:text-foreground"
                                            >Modrinth</a
                                        >
                                    {/if}
                                    {#if shader.curseforgeId}
                                        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
                                        <a
                                            href="https://www.curseforge.com/minecraft/shaders/{shader.curseforgeId}"
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
                                                    ({shader.modrinthId
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
                                                        placeholder={shader.modrinthId
                                                            ? "https://www.curseforge.com/minecraft/shaders/..."
                                                            : "https://modrinth.com/shader/..."}
                                                        bind:value={linkUrl}
                                                        onkeydown={handleLinkKeydown}
                                                        disabled={linking}
                                                    />
                                                    <p
                                                        class="text-xs text-muted-foreground"
                                                    >
                                                        Paste the {shader.modrinthId
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
                <form use:enhance class="space-y-6">
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
                                                    {@const pinned = versions.find((v) => v.version.id === $formData.preferred_version_id)}
                                                    <Pin class="mr-1.5 h-3.5 w-3.5 text-info" />
                                                    {pinned ? pinned.version.version : $formData.preferred_version_id}
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
                                                {#each versions as version, i (version.version.id)}
                                                    <Select.Item value={version.version.id}>
                                                        <span class="flex items-center gap-2">
                                                            {version.version.version}
                                                            {#if i === 0}
                                                                <span class="rounded bg-info/15 px-1.5 py-0.5 text-[10px] font-semibold leading-none text-info">
                                                                    Latest
                                                                </span>
                                                            {/if}
                                                            <span class="ml-auto text-xs text-muted-foreground">
                                                                {version.captureCount} captures
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
                    <DetailField label="ID">
                        <code class="text-xs">{shader.id}</code>
                    </DetailField>
                    <DetailField label="Slug">
                        {shader.slug}
                    </DetailField>
                    {#if shader.licenseId}
                        <DetailField label="License">
                            {shader.licenseId}
                        </DetailField>
                    {/if}
                    {#if shader.upstreamDownloads}
                        <DetailField label="Upstream Downloads">
                            {formatCount(shader.upstreamDownloads)}
                        </DetailField>
                    {/if}
                    {#if shader.upstreamUpdatedAt}
                        <DetailField label="Upstream Updated">
                            <TimeAgo timestamp={shader.upstreamUpdatedAt} />
                        </DetailField>
                    {/if}
                    <DetailField label="Created">
                        <TimeAgo timestamp={shader.createdAt} />
                    </DetailField>
                    <DetailField label="Updated">
                        <TimeAgo timestamp={shader.updatedAt} />
                    </DetailField>
                </dl>

                <!-- Authors -->
                {#if shader.authors.length > 0}
                    <div class="space-y-3">
                        <h3 class="text-sm font-medium">Authors ({shader.authors.length})</h3>
                        <div class="divide-y rounded-lg border">
                            {#each shader.authors as author (author.id)}
                                <div class="flex items-center justify-between px-4 py-2.5 text-sm">
                                    <div class="flex items-center gap-2">
                                        <span class="font-medium">{author.name}</span>
                                        <Badge variant="outline" class="text-xs">{author.platform}</Badge>
                                    </div>
                                    {#if author.url}
                                        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
                                        <a href={author.url} target="_blank" rel="noopener noreferrer"
                                           class="text-xs text-muted-foreground hover:text-foreground">
                                            {author.url}
                                        </a>
                                    {/if}
                                </div>
                            {/each}
                        </div>
                    </div>
                {/if}

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
                        <DataView items={versions}>
                            <Table columns={versionColumns} />
                        </DataView>
                    {/if}
                </div>

                <!-- Extraction Data -->
                {#if effectiveVersion}
                    {@const estatus = effectiveVersion.version.extractionStatus}
                    {#if estatus === "COMPLETED" && (profiles.length > 0 || metadata)}
                        <div class="space-y-4">
                            <div class="flex items-center gap-3">
                                <FlaskConical
                                    class="h-5 w-5 text-muted-foreground"
                                />
                                <h3 class="text-sm font-medium">
                                    Extraction Data
                                </h3>
                                <Badge variant="outline" class="text-xs"
                                    >{effectiveVersion.version.version}</Badge
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
                                                        (profile.options ?? {}) as Record<string, unknown>,
                                                    )}
                                                <Collapsible.Root class="group">
                                                    <Collapsible.Trigger
                                                        class="flex w-full items-center gap-3 px-4 py-2.5 text-left hover:bg-muted/50"
                                                    >
                                                        <span
                                                            class="shrink-0 text-xs text-muted-foreground tabular-nums"
                                                            >#{profile.sortOrder +
                                                                1}</span
                                                        >
                                                        <div
                                                            class="min-w-0 flex-1"
                                                        >
                                                            <span
                                                                class="font-medium text-sm"
                                                                >{profile.displayName}</span
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
                                                                <dd>{profile.displayName}</dd>
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
                                    {@const pipelineFeatures = (metadata.pipelineFeatures ?? {}) as Record<string, unknown>}
                                    {@const irisFeaturesRequired = metadata.irisFeaturesRequired ?? []}
                                    {@const irisFeaturesOptional = metadata.irisFeaturesOptional ?? []}
                                    {@const dimensionSupport = metadata.dimensionSupport ?? []}
                                    {@const filePaths = metadata.filePaths ?? []}
                                    {@const settingsScreen = (metadata.settingsScreen ?? []) as unknown[]}
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
                                                    timestamp={metadata.extractedAt}
                                                /></span
                                            >
                                        </div>
                                        <div class="space-y-4 px-4 py-3">
                                            {#if Object.keys(pipelineFeatures).length > 0}
                                                <div>
                                                    <div
                                                        class="mb-1.5 text-xs font-medium uppercase tracking-wide text-muted-foreground"
                                                    >
                                                        Pipeline Features
                                                    </div>
                                                    <div
                                                        class="flex flex-wrap gap-1.5"
                                                    >
                                                        {#each Object.entries(pipelineFeatures) as [key, val] (key)}
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

                                            {#if irisFeaturesRequired.length > 0 || irisFeaturesOptional.length > 0}
                                                <div>
                                                    <div
                                                        class="mb-1.5 text-xs font-medium uppercase tracking-wide text-muted-foreground"
                                                    >
                                                        Iris Features
                                                    </div>
                                                    {#if irisFeaturesRequired.length > 0}
                                                        <div class="mb-1">
                                                            <span
                                                                class="mr-1.5 text-[10px] text-muted-foreground"
                                                                >Required:</span
                                                            >
                                                            {#each irisFeaturesRequired as feat (feat)}
                                                                <Badge
                                                                    variant="default"
                                                                    class="mr-1 mb-1 text-[11px]"
                                                                    >{feat}</Badge
                                                                >
                                                            {/each}
                                                        </div>
                                                    {/if}
                                                    {#if irisFeaturesOptional.length > 0}
                                                        <div>
                                                            <span
                                                                class="mr-1.5 text-[10px] text-muted-foreground"
                                                                >Optional:</span
                                                            >
                                                            {#each irisFeaturesOptional as feat (feat)}
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

                                            {#if dimensionSupport.length > 0}
                                                <div>
                                                    <div
                                                        class="mb-1.5 text-xs font-medium uppercase tracking-wide text-muted-foreground"
                                                    >
                                                        Dimension Support
                                                    </div>
                                                    <div
                                                        class="flex flex-wrap gap-1.5"
                                                    >
                                                        {#each dimensionSupport as dim (dim)}
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

                                            {#if metadata.hasCustomTextures != null}
                                                <div>
                                                    <div
                                                        class="mb-1.5 text-xs font-medium uppercase tracking-wide text-muted-foreground"
                                                    >
                                                        Custom Textures
                                                    </div>
                                                    <div
                                                        class="flex items-center gap-1.5 text-sm"
                                                    >
                                                        {#if metadata.hasCustomTextures}
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

                                            {#if filePaths.length > 0}
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
                                                            >{filePaths.length} files</Badge
                                                        >
                                                        <ChevronDown
                                                            class="ml-auto h-3.5 w-3.5 text-muted-foreground transition-transform [[data-state=open]>&]:rotate-180"
                                                        />
                                                    </Collapsible.Trigger>
                                                    <Collapsible.Content>
                                                        <div
                                                            class="mt-2 max-h-64 overflow-y-auto rounded border bg-muted/30 p-2"
                                                        >
                                                            {#each filePaths as path, i (path)}
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

                                            {#if settingsScreen.length > 0}
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
                                                            >{settingsScreen.length} entries</Badge
                                                        >
                                                        <ChevronDown
                                                            class="ml-auto h-3.5 w-3.5 text-muted-foreground transition-transform [[data-state=open]>&]:rotate-180"
                                                        />
                                                    </Collapsible.Trigger>
                                                    <Collapsible.Content>
                                                        <pre
                                                            class="mt-2 max-h-80 overflow-y-auto rounded-md bg-muted p-3 font-mono text-xs">{JSON.stringify(
                                                                settingsScreen,
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
                    {:else if estatus === "FAILED"}
                        <Alert variant="destructive">
                            <AlertTriangle class="h-4 w-4" />
                            <div>
                                Extraction failed for v{effectiveVersion.version.version}{effectiveVersion.version.extractionError
                                    ? `: ${effectiveVersion.version.extractionError}`
                                    : ""}
                            </div>
                        </Alert>
                    {:else if estatus === "PENDING"}
                        <div
                            class="flex items-center gap-2 rounded-lg border bg-muted/30 px-4 py-3 text-sm text-muted-foreground"
                        >
                            <Clock class="h-4 w-4" />
                            Extraction pending for v{effectiveVersion.version.version}
                        </div>
                    {:else if estatus === "SKIPPED"}
                        <div
                            class="flex items-center gap-2 rounded-lg border bg-muted/30 px-4 py-3 text-sm text-muted-foreground"
                        >
                            <SkipForward class="h-4 w-4" />
                            Extraction skipped for v{effectiveVersion.version.version}
                        </div>
                    {/if}
                {/if}
            </div>
        </FolderCard.Content>

        <!-- Tab 3: Captures -->
        <FolderCard.Content value="captures">
            <div class="space-y-3">
                <Toolbar>
                    <FilterCombobox
                        placeholder="Scene"
                        options={sceneOptions}
                        bind:value={captureList.filters.sceneId}
                    />
                    <PopoverMultiSelect
                        label="Status"
                        options={captureStatusOptions}
                        bind:value={captureList.filters.statuses}
                    />
                    <PopoverMultiSelect
                        label="Freshness"
                        options={captureFreshnessOptions}
                        bind:value={captureList.filters.freshness}
                    />
                    <DateRangeFilter
                        label="Captured"
                        bind:after={captureList.filters.capturedAfter}
                        bind:before={captureList.filters.capturedBefore}
                    />
                    <NumericRangeFilter
                        label="File Size"
                        unit="MB"
                        scale={1048576}
                        step={0.1}
                        bind:min={captureList.filters.minFileSize}
                        bind:max={captureList.filters.maxFileSize}
                    />
                    <div class="flex-1"></div>
                    <ViewToggle modes={['table', 'tile']} bind:mode={captureList.viewMode} />
                </Toolbar>

                <DataView list={captureList}>
                    {#snippet empty()}
                        <p class="py-8 text-center text-sm text-muted-foreground">No captures yet.</p>
                    {/snippet}

                    <Table
                        columns={captureColumns}
                        getRowHref={(capture: AdminCaptureNode) => `/admin/captures/${capture.id}`}
                    >
                        {#snippet card(capture: AdminCaptureNode)}
                            <CaptureCard layout="row"
                                {capture}
                                title={capture.sceneName ?? 'Unknown scene'}
                                subtitle={[capture.shaderVersion, capture.profileDisplayName].filter(Boolean).join(' \u00b7 ')}
                            />
                        {/snippet}
                    </Table>
                    <Grid href={(capture: AdminCaptureNode) => `/admin/captures/${capture.id}`} key={(capture: AdminCaptureNode) => capture.id}>
                        {#snippet card(capture: AdminCaptureNode)}
                            <CaptureCard layout="tile"
                                {capture}
                                title={capture.sceneName ?? 'Unknown scene'}
                                subtitle={[capture.shaderVersion, capture.profileDisplayName].filter(Boolean).join(' \u00b7 ')}
                            />
                        {/snippet}
                    </Grid>

                    <Counter noun="capture" />

                    <Pagination style="load-more" />
                </DataView>
            </div>
        </FolderCard.Content>
    </FolderCard.Root>
</div>

<ConfirmDialog
    bind:open={showDeleteConfirm}
    title="Delete Shader"
    description={`Delete shader "${shader.name}"? It will be moved to trash and can be restored later.`}
    confirmLabel="Delete"
    onConfirm={deleteAction.execute}
/>

<ConfirmDialog
    bind:open={showPurgeConfirm}
    title="Permanently Delete Shader"
    description={`Permanently delete "${shader.name}"? This cannot be undone. All versions, captures, and metadata will be destroyed.`}
    confirmLabel="Permanently Delete"
    onConfirm={purgeAction.execute}
/>
