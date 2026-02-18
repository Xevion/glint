<script lang="ts">
import { api } from '$lib/api';
import type {
	AuditObject,
	AuditReference,
	StorageAuditResult,
	StorageCleanupResult
} from '$lib/bindings';
import { AdminBreadcrumb } from '$lib/components/admin';
import { Alert } from '$lib/components/ui/alert';
import { Badge } from '$lib/components/ui/badge';
import { Button } from '$lib/components/ui/button';
import * as Checkbox from '$lib/components/ui/checkbox';
import * as Collapsible from '$lib/components/ui/collapsible';
import { formatBytes } from '$lib/utils/format';
import { ChevronDown, ChevronRight, Loader2, Search, Trash2 } from '@lucide/svelte';

import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();

// Audit state
let auditResult = $state<StorageAuditResult | null>(null);
let auditing = $state(false);
let auditError = $state<string | null>(null);

// Cleanup state
let cleaning = $state(false);
let cleanupResult = $state<StorageCleanupResult | null>(null);

// Selection: key -> size in bytes
let selected = $state(new Map<string, number>());

// Collapsible open state
let openSections = $state<Record<string, boolean>>({
	orphaned: true,
	stale_staging: true,
	unknown_prefix: true,
	missing: true,
	url_mismatches: true
});

// Derived
let selectedCount = $derived(selected.size);
let selectedBytes = $derived.by(() => {
	let total = 0;
	for (const bytes of selected.values()) {
		total += bytes;
	}
	return total;
});

let deletableCount = $derived.by(() => {
	if (!auditResult) return 0;
	return (
		auditResult.orphaned.length + auditResult.stale_staging.length + auditResult.unknown_prefix.length
	);
});

function toggleSelection(key: string, size: number) {
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- Map mutation triggers via reassignment
	const next = new Map(selected);
	if (next.has(key)) {
		next.delete(key);
	} else {
		next.set(key, size);
	}
	selected = next;
}

function selectAll(objects: AuditObject[]) {
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- Map mutation triggers via reassignment
	const next = new Map(selected);
	for (const obj of objects) {
		next.set(obj.key, obj.size);
	}
	selected = next;
}

function deselectAll(objects: AuditObject[]) {
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- Map mutation triggers via reassignment
	const next = new Map(selected);
	for (const obj of objects) {
		next.delete(obj.key);
	}
	selected = next;
}

function allSelected(objects: AuditObject[]): boolean {
	if (objects.length === 0) return false;
	return objects.every((obj) => selected.has(obj.key));
}

async function runAudit() {
	auditing = true;
	auditError = null;
	cleanupResult = null;
	selected = new Map();

	const result = await api.admin.storageAudit();
	result.match({
		Ok: (v) => {
			auditResult = v;
		},
		Err: (e) => {
			auditError = e.message;
		}
	});

	auditing = false;
}

async function runCleanup() {
	if (selected.size === 0) return;
	cleaning = true;
	cleanupResult = null;

	const keys = [...selected.keys()];
	const result = await api.admin.storageCleanup(keys);
	result.match({
		Ok: (v) => {
			cleanupResult = v;
		},
		Err: (e) => {
			auditError = e.message;
		}
	});

	cleaning = false;

	// Re-run audit to refresh results
	await runAudit();
}

function formatDate(epochSeconds: number): string {
	return new Date(epochSeconds * 1000).toLocaleString();
}

interface DeletableCategory {
	id: string;
	label: string;
	objects: AuditObject[];
	deletable: true;
}

interface ReadOnlyRefCategory {
	id: string;
	label: string;
	references: AuditReference[];
	deletable: false;
}

type Category = DeletableCategory | ReadOnlyRefCategory;

let categories = $derived.by((): Category[] => {
	if (!auditResult) return [];
	return [
		{ id: 'orphaned', label: 'Orphaned', objects: auditResult.orphaned, deletable: true },
		{
			id: 'stale_staging',
			label: 'Stale Staging',
			objects: auditResult.stale_staging,
			deletable: true
		},
		{
			id: 'unknown_prefix',
			label: 'Unknown Prefix',
			objects: auditResult.unknown_prefix,
			deletable: true
		},
		{ id: 'missing', label: 'Missing', references: auditResult.missing, deletable: false },
		{
			id: 'url_mismatches',
			label: 'URL Mismatches',
			references: auditResult.url_mismatches,
			deletable: false
		}
	];
});
</script>

<svelte:head><title>Storage - Glint</title></svelte:head>

<div class="space-y-6">
	<AdminBreadcrumb segments={[{ label: 'Storage' }]} />

	<!-- Stats cards -->
	{#if data.stats}
		<div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
			<div class="rounded-lg border bg-card p-4">
				<p class="text-sm text-muted-foreground">Total Storage</p>
				<p class="text-2xl font-semibold">{formatBytes(data.stats.total_bytes)}</p>
			</div>
			<div class="rounded-lg border bg-card p-4">
				<p class="text-sm text-muted-foreground">Captures</p>
				<p class="text-2xl font-semibold">{data.stats.capture_count.toLocaleString()}</p>
			</div>
			<div class="rounded-lg border bg-card p-4">
				<p class="text-sm text-muted-foreground">Avg Size</p>
				<p class="text-2xl font-semibold">{formatBytes(data.stats.avg_bytes)}</p>
			</div>
			<div class="rounded-lg border bg-card p-4">
				<p class="text-sm text-muted-foreground">Missing</p>
				<p class="text-2xl font-semibold">{data.stats.missing_count.toLocaleString()}</p>
			</div>
		</div>
	{/if}

	<!-- Audit section -->
	<div class="rounded-lg border bg-card p-6">
		<div class="mb-4 flex items-center justify-between">
			<h2 class="text-lg font-semibold">Storage Audit</h2>
			<div class="flex gap-2">
				{#if selectedCount > 0}
					<Button variant="destructive" onclick={runCleanup} disabled={cleaning}>
						{#if cleaning}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							Deleting...
						{:else}
							<Trash2 class="mr-2 h-4 w-4" />
							Delete Selected ({selectedCount} / {formatBytes(selectedBytes)})
						{/if}
					</Button>
				{/if}
				<Button onclick={runAudit} disabled={auditing}>
					{#if auditing}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						Auditing...
					{:else}
						<Search class="mr-2 h-4 w-4" />
						Run Audit
					{/if}
				</Button>
			</div>
		</div>

		{#if cleanupResult}
			<Alert variant="success" class="mb-4">
				Cleanup complete: {cleanupResult.deleted_count} deleted, {cleanupResult.skipped_count} skipped, {cleanupResult.failed_count} failed.
				Freed {formatBytes(cleanupResult.freed_bytes)}.
			</Alert>
		{/if}

		{#if auditError}
			<Alert variant="destructive" class="mb-4">{auditError}</Alert>
		{/if}

		{#if auditing && !auditResult}
			<div class="flex items-center justify-center py-12">
				<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
			</div>
		{/if}

		{#if auditResult}
			<p class="mb-4 text-sm text-muted-foreground">
				Found {auditResult.summary.total_r2_objects.toLocaleString()} objects
				({formatBytes(auditResult.summary.total_r2_bytes)}) in R2,
				{auditResult.summary.total_db_references.toLocaleString()} DB references.
				{#if deletableCount > 0}
					{deletableCount.toLocaleString()} deletable objects found.
				{:else}
					No issues found.
				{/if}
			</p>

			<div class="space-y-2">
				{#each categories as category (category.id)}
					{@const count = category.deletable ? category.objects.length : category.references.length}
					{#if count > 0}
						<Collapsible.Root bind:open={openSections[category.id]}>
							<Collapsible.Trigger class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm font-medium hover:bg-muted">
								{#if openSections[category.id]}
									<ChevronDown class="h-4 w-4" />
								{:else}
									<ChevronRight class="h-4 w-4" />
								{/if}
								{category.label}
								<Badge variant="secondary" class="ml-auto">{count}</Badge>
							</Collapsible.Trigger>
							<Collapsible.Content>
								<div class="mt-1 overflow-x-auto">
									{#if category.deletable}
										{@const objects = category.objects}
										<table class="w-full text-sm">
											<thead>
												<tr class="border-b text-left text-xs text-muted-foreground">
													<th class="px-3 py-2 font-medium">
														<Checkbox.Root
															checked={allSelected(objects)}
															onCheckedChange={(v) => {
																if (v) selectAll(objects);
																else deselectAll(objects);
															}}
														/>
													</th>
													<th class="px-3 py-2 font-medium">Key</th>
													<th class="px-3 py-2 font-medium">Size</th>
													<th class="px-3 py-2 font-medium">Last Modified</th>
												</tr>
											</thead>
											<tbody>
												{#each objects as obj (obj.key)}
													<tr class="border-b border-border/50 last:border-b-0">
														<td class="px-3 py-2">
															<Checkbox.Root
																checked={selected.has(obj.key)}
																onCheckedChange={() => toggleSelection(obj.key, obj.size)}
															/>
														</td>
														<td class="max-w-md truncate px-3 py-2 font-mono text-xs">{obj.key}</td>
														<td class="whitespace-nowrap px-3 py-2">{formatBytes(obj.size)}</td>
														<td class="whitespace-nowrap px-3 py-2">{formatDate(obj.last_modified)}</td>
													</tr>
												{/each}
											</tbody>
										</table>
									{:else}
										{@const references = (category as ReadOnlyRefCategory).references}
										<table class="w-full text-sm">
											<thead>
												<tr class="border-b text-left text-xs text-muted-foreground">
													<th class="px-3 py-2 font-medium">Table</th>
													<th class="px-3 py-2 font-medium">Row ID</th>
													<th class="px-3 py-2 font-medium">Stored URL</th>
													{#if category.id === 'url_mismatches'}
														<th class="px-3 py-2 font-medium">Expected URL</th>
													{/if}
												</tr>
											</thead>
											<tbody>
												{#each references as ref (ref.row_id + ref.stored_url)}
													<tr class="border-b border-border/50 last:border-b-0">
														<td class="whitespace-nowrap px-3 py-2">{ref.table}</td>
														<td class="px-3 py-2 font-mono text-xs">{ref.row_id}</td>
														<td class="max-w-xs truncate px-3 py-2 font-mono text-xs">{ref.stored_url}</td>
														{#if category.id === 'url_mismatches' && ref.expected_url}
															<td class="max-w-xs truncate px-3 py-2 font-mono text-xs">{ref.expected_url}</td>
														{/if}
													</tr>
												{/each}
											</tbody>
										</table>
									{/if}
								</div>
							</Collapsible.Content>
						</Collapsible.Root>
					{/if}
				{/each}
			</div>
		{/if}
	</div>
</div>
