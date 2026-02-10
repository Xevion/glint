<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import type { SceneWithWorld } from '$lib/bindings';
import { AdminPageHeader } from '$lib/components/admin';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import * as Checkbox from '$lib/components/ui/checkbox';
import { Alert } from '$lib/components/ui/alert';
import { Camera, CloudSun, Globe, MapPin } from '@lucide/svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let scenes: SceneWithWorld[] = $derived(data.scenes);
let refreshing = $state(false);
let showInactive = $state(false);
let error = $derived(data.error);

const filteredScenes = $derived(showInactive ? scenes : scenes.filter((s) => s.active));

async function refresh() {
	refreshing = true;
	await Promise.all([invalidateAll(), new Promise((r) => setTimeout(r, 300))]);
	refreshing = false;
}

function formatDimension(dim: string): string {
	return dim.split(':').pop() ?? dim;
}
</script>

<svelte:head><title>Scenes - Glint</title></svelte:head>

<div class="space-y-4">
	<AdminPageHeader title="Scenes" count={filteredScenes.length} {refreshing} onrefresh={refresh}>
		{#snippet actions()}
			{#if showInactive && scenes.some((s) => !s.active)}
				<span class="text-sm text-muted-foreground">
					({scenes.filter((s) => !s.active).length} inactive)
				</span>
			{/if}
			<label class="flex items-center gap-2 text-sm">
				<Checkbox.Root
					checked={showInactive}
					onCheckedChange={(v) => (showInactive = v === true)}
				/>
				<span>Show inactive</span>
			</label>
		{/snippet}
	</AdminPageHeader>

	{#if error}
		<Alert variant="destructive">Error: {error}</Alert>
	{:else if filteredScenes.length === 0}
		<p class="text-muted-foreground">
			{showInactive
				? 'No scenes yet.'
				: 'No active scenes. Enable "Show inactive" to see disabled scenes.'}
		</p>
	{:else}
		<div class="flex flex-col overflow-hidden rounded-lg border border-border">
			{#each filteredScenes as scene (scene.id)}
				<button
					type="button"
					class="group flex w-full cursor-pointer gap-4 border-b border-border bg-card p-4 text-left transition-colors last:border-b-0 hover:bg-muted/50"
					onclick={() => goto(`/admin/scenes/${scene.id}`)}
				>
					<!-- Preview thumbnail -->
					<div class="hidden shrink-0 sm:block">
						{#if scene.image_url ?? scene.thumbhash}
							<CaptureImage
								src={scene.image_url}
								thumbhash={scene.thumbhash}
								alt="{scene.name} preview"
								preset="card"
								class="h-full w-full rounded-md object-cover"
								containerClass="h-20 w-32 overflow-hidden rounded-md"
							/>
						{:else}
							<div
								class="flex h-20 w-32 items-center justify-center rounded-md bg-muted"
							>
								<MapPin class="h-8 w-8 text-muted-foreground/40" />
							</div>
						{/if}
					</div>

					<!-- Content -->
					<div class="flex min-w-0 flex-1 flex-col gap-1.5">
						<!-- Title row -->
						<div class="flex items-center gap-2">
							<span class="truncate text-sm font-semibold">{scene.name}</span>
							<code
								class="shrink-0 rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground"
								>{scene.slug}</code
							>
							{#if !scene.active}
								<span
									class="shrink-0 rounded bg-destructive/10 px-1.5 py-0.5 text-xs font-medium text-destructive"
									>Inactive</span
								>
							{/if}
						</div>

						<!-- Description -->
						{#if scene.description}
							<p class="line-clamp-1 text-xs text-muted-foreground">
								{scene.description}
							</p>
						{/if}

						<!-- Stats row -->
						<div
							class="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground"
						>
							{#if scene.world_name}
								<span class="inline-flex items-center gap-1" title="World">
									<Globe class="h-3 w-3" />
									{scene.world_name}
								</span>
							{/if}
							<span class="inline-flex items-center gap-1" title="Dimension">
								<MapPin class="h-3 w-3" />
								{formatDimension(scene.dimension)}
							</span>
						<span class="inline-flex items-center gap-1" title="Weather">
							<CloudSun class="h-3 w-3" />
							<span class="capitalize">{scene.version.weather}</span>
						</span>
							<span class="inline-flex items-center gap-1" title="Captures">
								<Camera class="h-3 w-3" />
								{scene.capture_count} capture{scene.capture_count !== 1 ? 's' : ''}
							</span>
						</div>

						<!-- Timestamps -->
						<div
							class="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground/70"
						>
							<span>Created <TimeAgo timestamp={scene.created_at} /></span>
						</div>
					</div>
				</button>
			{/each}
		</div>
	{/if}
</div>
