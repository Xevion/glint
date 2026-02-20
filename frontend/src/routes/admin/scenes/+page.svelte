<script lang="ts">
import type { AdminScene } from './+page';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import Breadcrumb from '$lib/components/Breadcrumb.svelte';
import { DataList } from '$lib/components/data-list';
import { ItemGrid } from '$lib/components/item-grid';
import * as Checkbox from '$lib/components/ui/checkbox';
import { MapPin } from '@lucide/svelte';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let scenes: AdminScene[] = $derived(data.scenes);
let showInactive = $state(false);
let error = $derived(data.error);

const filteredScenes = $derived(showInactive ? scenes : scenes.filter((s) => s.active));
</script>

<svelte:head><title>Scenes - Glint</title></svelte:head>

<div class="space-y-4">
	<div class="flex items-center justify-between">
		<Breadcrumb segments={[{ label: 'Scenes' }]} />
		<div class="flex items-center gap-3">
			{#if showInactive && scenes.some((s) => !s.active)}
			<span class="text-sm text-foreground">
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
		</div>
	</div>

	<DataList items={filteredScenes} {error} emptyMessage={showInactive ? 'No scenes yet.' : 'No active scenes. Enable "Show inactive" to see disabled scenes.'}>
		{#snippet content()}
			<ItemGrid items={filteredScenes} key={(s: AdminScene) => s.id} mode="row">
				{#snippet row(scene: AdminScene)}
					<a
						href="/admin/scenes/{scene.id}"
						class="group flex w-full gap-4 p-4 transition-colors hover:bg-muted/50"
					>
						<!-- Preview thumbnail -->
						<div class="hidden shrink-0 sm:block">
							{#if scene.imagePath ?? scene.thumbhash}
								<CaptureImage
									src={scene.imagePath}
									thumbhash={scene.thumbhash}
									alt="{scene.name} preview"
									preset="thumbnail"
									class="h-full w-full rounded-md object-cover"
									containerClass="h-20 w-32 overflow-hidden rounded-md"
								/>
							{:else}
								<div
									class="flex h-20 w-32 items-center justify-center rounded-md bg-muted"
								>
									<MapPin class="h-8 w-8 text-muted-foreground opacity-40" />
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

							<!-- Timestamps -->
							<div
								class="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground/70"
							>
								<span>Created <TimeAgo timestamp={scene.createdAt} /></span>
							</div>
						</div>
					</a>
				{/snippet}
			</ItemGrid>
		{/snippet}
	</DataList>
</div>
