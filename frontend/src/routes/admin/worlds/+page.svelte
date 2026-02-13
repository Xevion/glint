<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import type { WorldListItem } from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import WorldUploadDialog from '$lib/components/WorldUploadDialog.svelte';
import { AdminPageHeader } from '$lib/components/admin';
import { Alert } from '$lib/components/ui/alert';
import { formatBytes } from '$lib/utils/format';
import { Camera, Globe, HardDrive, Layers, MapPin } from '@lucide/svelte';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let worlds = $derived(data.worlds);
let refreshing = $state(false);
let error = $derived(data.error);

async function refresh() {
	refreshing = true;
	await Promise.all([invalidateAll(), new Promise((r) => setTimeout(r, 300))]);
	refreshing = false;
}

function sizeBytes(world: WorldListItem): number | null {
	return world.latest_version?.size_bytes ?? null;
}

function lastUploadAt(world: WorldListItem): string | null {
	return world.latest_version?.created_at ?? null;
}
</script>

<svelte:head><title>Worlds - Glint</title></svelte:head>

<div class="space-y-4">
	<AdminPageHeader title="Worlds" count={worlds.length} {refreshing} onrefresh={refresh}>
		{#snippet actions()}
			<WorldUploadDialog onWorldCreated={refresh} />
		{/snippet}
	</AdminPageHeader>

	{#if error}
		<Alert variant="destructive">Error: {error}</Alert>
	{:else if worlds.length === 0}
		<p class="text-muted-foreground">No worlds uploaded yet.</p>
	{:else}
		<div class="flex flex-col overflow-hidden rounded-lg border border-border">
			{#each worlds as world (world.id)}
				<button
					type="button"
					class="group flex w-full cursor-pointer gap-4 border-b border-border bg-card p-4 text-left transition-colors last:border-b-0 hover:bg-muted/50"
					onclick={() => goto(`/admin/worlds/${world.id}`)}
				>
					<!-- Preview thumbnail -->
					<div class="hidden shrink-0 sm:block">
						{#if world.preview?.image_url ?? world.preview?.thumbhash}
							<CaptureImage
								src={world.preview?.image_url}
								thumbhash={world.preview?.thumbhash}
								alt="{world.name} preview"
								preset="thumbnail"
								class="h-full w-full rounded-md object-cover"
								containerClass="h-20 w-32 overflow-hidden rounded-md"
							/>
						{:else}
							<div
								class="flex h-20 w-32 items-center justify-center rounded-md bg-muted"
							>
								<Globe class="h-8 w-8 text-muted-foreground opacity-40" />
							</div>
						{/if}
					</div>

					<!-- Content -->
					<div class="flex min-w-0 flex-1 flex-col gap-1.5">
						<!-- Title row -->
						<div class="flex items-center gap-2">
							<span class="truncate text-sm font-semibold">{world.name}</span>
							<code
								class="shrink-0 rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground"
								>{world.slug}</code
							>
							<code
								class="shrink-0 rounded bg-primary/10 px-1.5 py-0.5 text-xs font-medium text-primary"
								>{world.minecraft_version}</code
							>
						</div>

						<!-- Description -->
						{#if world.description}
							<p class="line-clamp-1 text-xs text-muted-foreground">
								{world.description}
							</p>
						{/if}

						<!-- Stats row -->
						<div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground">
							<span class="inline-flex items-center gap-1" title="Scenes">
								<MapPin class="h-3 w-3" />
								{world.scene_count} scene{world.scene_count !== 1 ? 's' : ''}
							</span>
							<span class="inline-flex items-center gap-1" title="Versions">
								<Layers class="h-3 w-3" />
								{world.version_count} version{world.version_count !== 1 ? 's' : ''}
							</span>
							<span class="inline-flex items-center gap-1" title="Captures">
								<Camera class="h-3 w-3" />
								{world.capture_count} capture{world.capture_count !== 1 ? 's' : ''}
							</span>
							{#if sizeBytes(world)}
								<span class="inline-flex items-center gap-1" title="Latest version size">
									<HardDrive class="h-3 w-3" />
									{formatBytes(sizeBytes(world)!)}
								</span>
							{/if}
						</div>

						<!-- Timestamps -->
						<div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground/70">
							<span>Updated <TimeAgo timestamp={world.updated_at} /></span>
							{#if lastUploadAt(world)}
								<span>Last upload <TimeAgo timestamp={lastUploadAt(world)!} /></span>
							{/if}
						</div>
					</div>
				</button>
			{/each}
		</div>
	{/if}
</div>
