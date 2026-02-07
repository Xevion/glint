<script lang="ts">
import { invalidateAll } from '$app/navigation';
import type { CaptureTargetHealth, TargetHealth } from '$lib/bindings';
import { Badge } from '$lib/components/ui/badge';
import { Button } from '$lib/components/ui/button';
import * as Tabs from '$lib/components/ui/tabs';
import * as Tooltip from '$lib/components/ui/tooltip';
import { Activity, Grid3x3, RefreshCw } from '@lucide/svelte';
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
	completed: 'bg-green-500/80 hover:bg-green-500',
	stale: 'bg-yellow-500/80 hover:bg-yellow-500',
	missing: 'bg-red-500/80 hover:bg-red-500',
	failed: 'bg-zinc-400/60 hover:bg-zinc-400'
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

function tooltipSummary(shaderSlug: string, sceneSlug: string): string {
	const targets = targetsByCell.get(`${shaderSlug}:${sceneSlug}`) ?? [];
	if (targets.length === 0) return 'No targets';
	if (targets.length === 1) {
		const t = targets[0];
		return `${STATUS_LABELS[t.status]}${t.profile ? ` (${t.profile})` : ''}`;
	}
	const counts: Record<string, number> = {};
	for (const t of targets) {
		counts[t.status] = (counts[t.status] ?? 0) + 1;
	}
	return Object.entries(counts)
		.map(([s, c]) => `${c} ${s}`)
		.join(', ');
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
	await invalidateAll();
	refreshing = false;
}
</script>

<div class="space-y-4">
	<header class="flex items-center justify-between">
		<div class="flex items-baseline gap-3">
			<h1 class="text-2xl font-semibold">Capture Health</h1>
		</div>
		<Button variant="outline" size="icon" onclick={refresh} disabled={refreshing}>
			<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
		</Button>
	</header>

	{#if health.error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
			Error: {health.error}
		</div>
	{/if}

	<!-- Summary -->
	<div class="flex flex-wrap gap-2">
		<Badge variant="default">{health.summary.completed} completed</Badge>
		<Badge variant="destructive">{health.summary.missing} missing</Badge>
		<Badge variant="secondary">{health.summary.stale} stale</Badge>
		<Badge variant="outline">{health.summary.failed} failed</Badge>
		<span class="text-sm text-muted-foreground">
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
				<p class="text-muted-foreground">
					{shaders.length === 0 ? 'No shaders found.' : 'No active scenes found.'}
				</p>
			{:else}
				<div class="flex gap-4">
					<!-- Matrix Grid -->
					<div class="min-w-0 flex-1 overflow-x-auto">
						<Tooltip.Provider>
							<table class="w-full border-collapse text-sm">
								<thead>
									<tr>
										<th class="sticky left-0 z-10 bg-background p-2 text-left text-xs font-medium text-muted-foreground">
											Shader
										</th>
										{#each scenes as scene (scene.slug)}
											<th class="p-1 text-center text-xs font-medium text-muted-foreground">
												<span class="block max-w-16 truncate" title={scene.name}>
													{scene.name}
												</span>
											</th>
										{/each}
									</tr>
								</thead>
								<tbody>
									{#each shaders as shader (shader.slug)}
										<tr class="border-t border-border/50">
											<td class="sticky left-0 z-10 bg-background p-2">
												<div class="flex flex-col">
													<span class="max-w-40 truncate font-medium" title={shader.name}>
														{shader.name}
													</span>
													<span class="text-xs text-muted-foreground">{shader.version}</span>
												</div>
											</td>
											{#each scenes as scene (scene.slug)}
												{@const targets = targetsByCell.get(`${shader.slug}:${scene.slug}`) ?? []}
												{@const status = worstStatus(targets)}
												<td class="p-1 text-center">
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
												<span class="ml-2 text-red-500">
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
				<p class="text-muted-foreground">Work queue is empty. All targets are up to date.</p>
			{:else}
				<div class="rounded-lg border">
					<table class="w-full text-sm">
						<thead>
							<tr class="border-b text-left text-xs font-medium text-muted-foreground">
								<th class="p-3">#</th>
								<th class="p-3">Shader</th>
								<th class="p-3">Version</th>
								<th class="p-3">Scene</th>
								<th class="p-3">Profile</th>
								<th class="p-3">World</th>
							</tr>
						</thead>
						<tbody>
							{#each workQueue as item, i (i)}
								<tr class="border-b border-border/50 last:border-b-0">
									<td class="p-3 text-muted-foreground">{i + 1}</td>
									<td class="p-3 font-medium">{item.shader_name}</td>
									<td class="p-3 text-muted-foreground">{item.version}</td>
									<td class="p-3">{item.scene_name}</td>
									<td class="p-3 text-muted-foreground">{item.profile ?? '—'}</td>
									<td class="p-3 text-muted-foreground">{item.world_name}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</Tabs.Content>
	</Tabs.Root>
</div>
