<script lang="ts">
import { invalidate } from '$app/navigation';
import ErrorBanner from '$lib/components/ErrorBanner.svelte';
import Meta from '$lib/components/Meta.svelte';
import SceneCard from '$lib/components/SceneCard.svelte';
import {
	DataView,
	Grid,
	Search,
	Toolbar,
	ViewToggle,
	createClientList
} from '$lib/components/data-view';
import CompactRow from '$lib/components/CompactRow.svelte';
import { Badge } from '$lib/components/ui/badge';
import { formatTimeTicks } from '$lib/utils/format';
import { AlertTriangle, Camera, ChevronRight, Search as SearchIcon } from '@lucide/svelte';
import { fade, fly } from 'svelte/transition';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
const scenes = $derived(data.scenes);
const loadError = $derived(data.error);

const list = createClientList({
	items: () => scenes,
	search: {
		clientFilter: (scene, q) => scene.name.toLowerCase().includes(q.toLowerCase())
	},
	syncUrl: true
});

// OG image: first scene's representative capture
const ogImage: string | null = $derived(scenes[0]?.imagePath ?? null);
</script>

<Meta
	title="Scenes"
	description="Explore Minecraft test scenes used for shader comparison. See how different lighting, biomes, and weather affect each shader."
	image={ogImage}
	ogImagePath="/og/scenes/og.png"
/>

<div class="py-8">
	<!-- Header -->
	<div in:fly={{ y: -10, duration: 400 }} class="mb-8">
		<h1 class="mb-2 text-2xl font-semibold tracking-tight">Test Scenes</h1>
		<p class="text-lg text-foreground">
			{scenes.length} standardized environments for consistent shader comparison
		</p>
	</div>

	<!-- Toolbar -->
	<Toolbar class="mb-6">
		<Search bind:value={list.search} placeholder="Search scenes..." />

		<!-- Spacer pushes view toggle to the right -->
		<div class="flex-1"></div>

		<ViewToggle modes={['grid', 'row']} bind:mode={list.viewMode} />
	</Toolbar>

	<!-- Error Banner -->
	{#if loadError}
		<div class="mb-4">
			<ErrorBanner message="Failed to load scenes: {loadError}" onRetry={() => invalidate('glint:scenes')} />
		</div>
	{/if}

	<!-- Results count -->
	{#if !loadError}
		<div in:fade={{ duration: 300, delay: 200 }} class="mb-4 text-sm text-foreground">
			{#if list.items.length === scenes.length}
				Showing all {scenes.length} scenes
			{:else}
				Showing {list.items.length} of {scenes.length} scenes
			{/if}
		</div>
	{/if}

	<!-- Scene Grid -->
	{#if loadError && scenes.length === 0}
		<div class="flex flex-col items-center justify-center py-16 text-center">
			<AlertTriangle class="mb-4 h-16 w-16 text-destructive opacity-50" strokeWidth={1.5} />
			<h3 class="text-lg font-semibold text-foreground">Failed to load scenes</h3>
			<p class="mt-1 text-sm text-foreground/70">Check your connection and try again</p>
		</div>
	{:else}
		<DataView {list}>
			{#snippet empty()}
				<div class="flex flex-col items-center justify-center py-16 text-center">
					<SearchIcon class="mb-4 h-16 w-16 text-muted-foreground opacity-50" strokeWidth={1.5} />
					{#if list.search}
						<h3 class="text-lg font-semibold text-foreground">No scenes found</h3>
						<p class="mt-1 text-sm text-foreground/70">Try adjusting your filters</p>
					{:else}
						<h3 class="text-lg font-semibold text-foreground">No scenes yet</h3>
						<p class="mt-1 text-sm text-foreground/70">Scenes will appear here once they've been created</p>
					{/if}
				</div>
			{/snippet}

			<Grid
				mode={list.viewMode}
				size="small"
				key={(s: PageData['scenes'][number]) => s.slug}
			>
				{#snippet card(scene: PageData['scenes'][number])}
					<SceneCard {scene} />
				{/snippet}
				{#snippet row(scene: PageData['scenes'][number])}
					<CompactRow
						name={scene.name}
						subtitle={scene.dimension}
						image={scene.imagePath}
						thumbhash={scene.thumbhash}
						href={`/scenes/${scene.slug}`}
					>
						{#snippet metadata()}
							<div class="flex items-center gap-2">
								{#if scene.version?.biome}
									<Badge variant="secondary" class="text-[11px]">
										{scene.version.biome.replace('minecraft:', '')}
									</Badge>
								{/if}
								{#if scene.version}
									<span class="text-xs text-muted-foreground">
										{formatTimeTicks(scene.version.timeOfDayTicks)}
									</span>
								{/if}
								{#if scene.captureCount > 0}
									<span class="flex items-center gap-1 text-xs text-muted-foreground">
										<Camera class="h-3 w-3" strokeWidth={2} />
										{scene.captureCount}
									</span>
								{/if}
							</div>
						{/snippet}
						{#snippet trailing()}
							<ChevronRight class="h-4 w-4 text-muted-foreground" />
						{/snippet}
					</CompactRow>
				{/snippet}
			</Grid>
		</DataView>
	{/if}
</div>
