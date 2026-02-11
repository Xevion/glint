<script lang="ts">
import { invalidateAll } from '$app/navigation';
import type { CaptureTargetHealth, StaleReason, TargetHealth } from '$lib/bindings';
import RefreshButton from '$lib/components/RefreshButton.svelte';
import { Alert } from '$lib/components/ui/alert';
import { Badge } from '$lib/components/ui/badge';
import * as Table from '$lib/components/ui/table';
import * as Tabs from '$lib/components/ui/tabs';
import * as Tooltip from '$lib/components/ui/tooltip';
import { Activity, Grid3x3 } from '@lucide/svelte';
import { SvelteMap } from 'svelte/reactivity';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();

let health = $derived(data.health);
let workQueue = $derived(data.workQueue);
let refreshing = $state(false);
let selectedCell = $state<{ shaderSlug: string; sceneSlug: string } | null>(null);

// Derive unique shaders and scenes for grid axes
let shaders = $derived.by(() => {
	const seen = new SvelteMap<string, { id: string; name: string; slug: string; version: string }>();
	for (const t of health.targets) {
		if (!seen.has(t.shader_slug)) {
			seen.set(t.shader_slug, {
				id: t.shader_id,
				name: t.shader_name,
				slug: t.shader_slug,
				version: t.version
			});
		}
	}
	return [...seen.values()].sort((a, b) => a.name.localeCompare(b.name));
});

let scenes = $derived.by(() => {
	const seen = new SvelteMap<string, { id: string; name: string; slug: string }>();
	for (const t of health.targets) {
		if (!seen.has(t.scene_slug)) {
			seen.set(t.scene_slug, {
				id: t.scene_id,
				name: t.scene_name,
				slug: t.scene_slug
			});
		}
	}
	return [...seen.values()].sort((a, b) => a.name.localeCompare(b.name));
});

// Build lookup: (shaderSlug, sceneSlug) -> CaptureTargetHealth[]
let targetsByCell = $derived.by(() => {
	const map = new SvelteMap<string, CaptureTargetHealth[]>();
	for (const t of health.targets) {
		const key = `${t.shader_slug}:${t.scene_slug}`;
		const list = map.get(key) ?? [];
		list.push(t);
		map.set(key, list);
	}
	return map;
});

// Status priority for worst-of aggregation
const STATUS_PRIORITY: Record<TargetHealth, number> = {
	failed: 0,
	missing: 1,
	stale: 2,
	completed: 3
};

function worstStatus(targets: CaptureTargetHealth[]): TargetHealth {
	let worst: TargetHealth = 'completed';
	for (const t of targets) {
		if (STATUS_PRIORITY[t.status] < STATUS_PRIORITY[worst]) {
			worst = t.status;
		}
	}
	return worst;
}

const CELL_COLORS: Record<TargetHealth, string> = {
	completed: 'bg-success/80 hover:bg-success',
	stale: 'bg-warning/80 hover:bg-warning',
	missing: 'bg-destructive/80 hover:bg-destructive',
	failed: 'bg-muted-foreground/40 hover:bg-muted-foreground/60'
};

const STATUS_LABELS: Record<TargetHealth, string> = {
	completed: 'Completed',
	stale: 'Stale',
	missing: 'Missing',
	failed: 'Failed'
};

const BADGE_VARIANTS: Record<TargetHealth, 'default' | 'secondary' | 'destructive' | 'outline'> = {
	completed: 'default',
	stale: 'secondary',
	missing: 'destructive',
	failed: 'outline'
};

const STALE_REASON_LABELS: Record<StaleReason, string> = {
	world_updated: 'World updated',
	scene_updated: 'Scene updated',
	both_updated: 'World + scene updated'
};

function relativeTime(iso: string): string {
	const ms = Date.now() - new Date(iso).getTime();
	const seconds = Math.floor(ms / 1000);
	if (seconds < 60) return 'just now';
	const minutes = Math.floor(seconds / 60);
	if (minutes < 60) return `${minutes}m ago`;
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return `${hours}h ago`;
	const days = Math.floor(hours / 24);
	if (days < 30) return `${days}d ago`;
	const months = Math.floor(days / 30);
	return `${months}mo ago`;
}

function targetDetail(t: CaptureTargetHealth): string {
	const age = t.last_capture_at ? relativeTime(t.last_capture_at) : null;
	switch (t.status) {
		case 'stale': {
			const reason = t.stale_reason ? STALE_REASON_LABELS[t.stale_reason] : 'Stale';
			return age ? `${reason} — captured ${age}` : reason;
		}
		case 'missing':
			return 'Missing — no capture exists';
		case 'failed':
			return `Failed — ${t.failure_count} consecutive failures`;
		case 'completed':
			return age ? `Completed — captured ${age}` : 'Completed';
	}
}

function tooltipSummary(shaderSlug: string, sceneSlug: string): string {
	const targets = targetsByCell.get(`${shaderSlug}:${sceneSlug}`) ?? [];
	if (targets.length === 0) return 'No targets';
	if (targets.length === 1) {
		const t = targets[0];
		const detail = targetDetail(t);
		return t.profile ? `${detail} (${t.profile})` : detail;
	}
	return targets
		.map((t) => {
			const label = t.profile ?? 'Default';
			return `${label}: ${targetDetail(t)}`;
		})
		.join('\n');
}

let selectedTargets = $derived.by(() => {
	if (!selectedCell) return [];
	return targetsByCell.get(`${selectedCell.shaderSlug}:${selectedCell.sceneSlug}`) ?? [];
});

function formatTime(iso: string | null): string {
	if (!iso) return 'Never';
	const d = new Date(iso);
	return d.toLocaleDateString('en-US', {
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit'
	});
}

async function refresh() {
	refreshing = true;
	await Promise.all([invalidateAll(), new Promise((r) => setTimeout(r, 300))]);
	refreshing = false;
}
</script>

<svelte:head><title>Health - Glint</title></svelte:head>

<div class="space-y-4">
	<header class="flex items-center justify-between">
		<div class="flex items-baseline gap-3">
			<h1 class="text-2xl font-semibold">Capture Health</h1>
		</div>
		<RefreshButton {refreshing} onclick={refresh} />
	</header>

	{#if health.error}
		<Alert variant="destructive">Error: {health.error}</Alert>
	{/if}

	<!-- Summary -->
	<div class="flex flex-wrap gap-2">
		<Badge variant="default">{health.summary.completed} completed</Badge>
		<Badge variant="destructive">{health.summary.missing} missing</Badge>
		<Badge variant="secondary">{health.summary.stale} stale</Badge>
		<Badge variant="outline">{health.summary.failed} failed</Badge>
	<span class="text-sm text-foreground">
		/ {health.summary.total_targets} total targets
	</span>
	</div>

	<Tabs.Root value="matrix">
		<Tabs.List>
			<Tabs.Trigger value="matrix">
				<Grid3x3 class="mr-1.5 h-4 w-4" />
				Capture Matrix
			</Tabs.Trigger>
			<Tabs.Trigger value="queue">
				<Activity class="mr-1.5 h-4 w-4" />
				Work Queue ({workQueue.length})
			</Tabs.Trigger>
		</Tabs.List>

		<Tabs.Content value="matrix" class="mt-4">
		{#if shaders.length === 0 || scenes.length === 0}
			<p class="text-foreground">
				{shaders.length === 0 ? 'No shaders found.' : 'No active scenes found.'}
			</p>
			{:else}
				<div class="flex gap-4">
					<!-- Matrix Grid -->
					<div class="min-w-0 flex-1 overflow-x-auto">
						<Tooltip.Provider delayDuration={0} disableHoverableContent>
							<table class="border-collapse text-sm">
								<thead>
									<tr>
										<th class="sticky left-0 z-10 p-2 text-left text-xs font-medium text-muted-foreground">
										</th>
										{#each scenes as scene (scene.slug)}
											<th class="h-28 w-9 p-0 align-bottom">
												<div class="relative h-full w-full">
													<span
														class="absolute bottom-1 left-1/2 origin-bottom-left -rotate-55 whitespace-nowrap text-xs font-medium text-muted-foreground"
														title={scene.name}
													>
														{scene.name}
													</span>
												</div>
											</th>
										{/each}
									</tr>
								</thead>
								<tbody>
									{#each shaders as shader (shader.slug)}
										<tr class="h-9 border-t border-border/50">
											<td class="sticky left-0 z-10 max-w-40 bg-background px-2 py-0">
												<div class="flex h-9 flex-col justify-center overflow-hidden">
													<span class="truncate text-xs font-medium" title={shader.name}>
														{shader.name}
													</span>
													<span class="truncate text-[10px] leading-tight text-muted-foreground">{shader.version}</span>
												</div>
											</td>
											{#each scenes as scene (scene.slug)}
												{@const targets = targetsByCell.get(`${shader.slug}:${scene.slug}`) ?? []}
												{@const status = worstStatus(targets)}
												<td class="bg-background p-1 text-center">
													<Tooltip.Root>
														<Tooltip.Trigger
															class="h-7 w-7 rounded transition-colors {CELL_COLORS[status]} {selectedCell?.shaderSlug === shader.slug && selectedCell?.sceneSlug === scene.slug ? 'ring-2 ring-primary ring-offset-1 ring-offset-background' : ''}"
															aria-label="{shader.name} / {scene.name}"
															onclick={() =>
																(selectedCell =
																	selectedCell?.shaderSlug === shader.slug &&
																	selectedCell?.sceneSlug === scene.slug
																		? null
																		: {
																				shaderSlug: shader.slug,
																				sceneSlug: scene.slug
																			})}
														/>
														<Tooltip.Content>
															<p class="text-xs">
																{shader.name} / {scene.name}
															</p>
															<p class="text-xs text-muted-foreground">
																{tooltipSummary(shader.slug, scene.slug)}
															</p>
														</Tooltip.Content>
													</Tooltip.Root>
												</td>
											{/each}
										</tr>
									{/each}
								</tbody>
							</table>
						</Tooltip.Provider>
					</div>

					<!-- Detail Panel -->
					{#if selectedCell && selectedTargets.length > 0}
						<div class="w-72 shrink-0 rounded-lg border bg-card p-4">
							<h3 class="mb-3 font-medium">
								{selectedTargets[0].shader_name} / {selectedTargets[0].scene_name}
							</h3>
							<div class="space-y-2">
								{#each selectedTargets as target (target.profile ?? '__default')}
									<div class="rounded border p-2 text-sm">
										<div class="flex items-center justify-between">
											<span class="text-muted-foreground">
												{target.profile ?? 'Default'}
											</span>
											<Badge variant={BADGE_VARIANTS[target.status]} class="text-xs">
												{STATUS_LABELS[target.status]}
											</Badge>
										</div>
										<div class="mt-1 text-xs text-muted-foreground">
											{#if target.last_capture_at}
												Last: {formatTime(target.last_capture_at)}
											{:else}
												No captures
											{/if}
											{#if target.failure_count > 0}
											<span class="ml-2 text-destructive">
												{target.failure_count} failures
											</span>
											{/if}
										</div>
									</div>
								{/each}
							</div>
						</div>
					{/if}
				</div>
			{/if}
		</Tabs.Content>

		<Tabs.Content value="queue" class="mt-4">
			{#if workQueue.length === 0}
				<p class="text-foreground">Work queue is empty. All targets are up to date.</p>
			{:else}
			<Table.Root class="border">
				<Table.Header>
					<Table.Row>
						<Table.Head class="p-3">#</Table.Head>
						<Table.Head class="p-3">Shader</Table.Head>
						<Table.Head class="p-3">Version</Table.Head>
						<Table.Head class="p-3">Scene</Table.Head>
						<Table.Head class="p-3">Profile</Table.Head>
						<Table.Head class="p-3">World</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each workQueue as item, i (i)}
						<Table.Row class="last:border-b-0">
							<Table.Cell class="p-3 text-muted-foreground">{i + 1}</Table.Cell>
							<Table.Cell class="p-3 font-medium">
								<a href="/admin/shaders/{item.shader_slug}" class="hover:underline">{item.shader_name}</a>
							</Table.Cell>
							<Table.Cell class="p-3 text-muted-foreground">{item.version}</Table.Cell>
							<Table.Cell class="p-3">
								<a href="/admin/scenes/{item.scene_id}" class="hover:underline">{item.scene_name}</a>
							</Table.Cell>
							<Table.Cell class="p-3 text-muted-foreground">{item.profile ?? '—'}</Table.Cell>
							<Table.Cell class="p-3 text-muted-foreground">
								<a href="/admin/worlds/{item.world_id}" class="hover:underline">{item.world_name}</a>
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
			{/if}
		</Tabs.Content>
	</Tabs.Root>
</div>
