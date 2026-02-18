<script lang="ts">
import BrowseToolbar from '$lib/components/BrowseToolbar.svelte';
import ErrorBanner from '$lib/components/ErrorBanner.svelte';
import { CompactRow, ItemGrid, ViewToggle } from '$lib/components/item-grid';
import type { GridMode } from '$lib/components/item-grid';
import Meta from '$lib/components/Meta.svelte';
import SceneCard from '$lib/components/SceneCard.svelte';
import { Badge } from '$lib/components/ui/badge';
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
import { AlertTriangle, Camera, ChevronRight, Search } from '@lucide/svelte';
import { formatTimeTicks } from '$lib/utils/display';
import { fade, fly } from 'svelte/transition';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
const scenes = $derived(data.scenes);
const loadError = $derived(data.error);

// Filter state
let searchQuery = $state('');
let viewMode = $state<GridMode>('card');

const filteredScenes = $derived.by(() => {
	return scenes.filter((scene: (typeof scenes)[0]) => {
		if (searchQuery && !scene.name.toLowerCase().includes(searchQuery.toLowerCase())) return false;
		return true;
	});
});

const hasFilters = $derived(searchQuery !== '');

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
		<p class="text-lg text-muted-foreground">
			{scenes.length} standardized environments for consistent shader comparison
		</p>
	</div>

	<!-- Toolbar -->
	<BrowseToolbar class="mb-6">
		<!-- Search -->
		<div class="relative min-w-0 flex-1 sm:min-w-50 sm:flex-initial">
			<Search
				class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground"
				strokeWidth={2}
			/>
			<Input type="text" placeholder="Search scenes..." bind:value={searchQuery} class="pr-4 pl-10 sm:w-56" />
		</div>

		{#if hasFilters}
			<Button
				variant="ghost"
				size="sm"
				onclick={() => {
					searchQuery = '';
				}}
			>
				Clear
			</Button>
		{/if}

		<!-- Spacer pushes view toggle to the right -->
		<div class="flex-1"></div>

		<ViewToggle mode={viewMode} onchange={(m: GridMode) => { viewMode = m; }} />
	</BrowseToolbar>

	<!-- Error Banner -->
	{#if loadError}
		<div class="mb-4">
			<ErrorBanner message="Failed to load scenes: {loadError}" />
		</div>
	{/if}

	<!-- Results count -->
	{#if !loadError}
		<div in:fade={{ duration: 300, delay: 200 }} class="mb-4 text-sm text-muted-foreground">
			{#if filteredScenes.length === scenes.length}
				Showing all {scenes.length} scenes
			{:else}
				Showing {filteredScenes.length} of {scenes.length} scenes
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
		<ItemGrid
			items={filteredScenes}
			key={(s: PageData['scenes'][number]) => s.slug}
			mode={viewMode}
			size="small"
			empty={hasFilters
				? { icon: Search, title: 'No scenes found', message: 'Try adjusting your filters' }
				: {
						icon: Search,
						title: 'No scenes yet',
						message: "Scenes will appear here once they've been created"
					}}
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
		</ItemGrid>
	{/if}
</div>
