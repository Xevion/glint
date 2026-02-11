<script lang="ts">
import Meta from '$lib/components/Meta.svelte';
import SceneCard from '$lib/components/SceneCard.svelte';
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
import { Search } from '@lucide/svelte';
import { fade, fly, scale } from 'svelte/transition';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
const scenes = $derived(data.scenes);

// Filter state
let searchQuery = $state('');

const filteredScenes = $derived.by(() => {
	return scenes.filter((scene: (typeof scenes)[0]) => {
		if (searchQuery && !scene.name.toLowerCase().includes(searchQuery.toLowerCase())) return false;
		return true;
	});
});

const hasFilters = $derived(searchQuery !== '');

// OG image: first scene's representative capture
const ogImage = $derived(scenes[0]?.image_url ?? null);
</script>

<Meta
	title="Scenes"
	description="Explore Minecraft test scenes used for shader comparison. See how different lighting, biomes, and weather affect each shader."
	image={ogImage}
/>

<div class="py-8">
	<!-- Header -->
	<div in:fly={{ y: -10, duration: 400 }} class="mb-8">
		<h1 class="mb-2 text-2xl font-semibold tracking-tight">Test Scenes</h1>
	<p class="text-lg text-foreground">
		{scenes.length} standardized environments for consistent shader comparison
	</p>
	</div>

	<!-- Filters -->
	<div in:fly={{ y: 10, duration: 400, delay: 100 }} class="mb-6 space-y-4 rounded-xl bg-card p-4">
		<div class="flex flex-wrap items-center gap-4">
			<!-- Search -->
			<div class="relative min-w-0 flex-1 sm:min-w-50">
				<Search
					class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground"
					strokeWidth={2}
				/>
			<Input
				type="text"
				placeholder="Search scenes..."
				bind:value={searchQuery}
				class="pr-4 pl-10"
			/>
			</div>

			{#if hasFilters}
				<Button
					variant="ghost"
					size="sm"
					onclick={() => {
						searchQuery = '';
					}}
				>
					Clear filters
				</Button>
			{/if}
		</div>
	</div>

	<!-- Results count -->
	<div in:fade={{ duration: 300, delay: 200 }} class="mb-4 text-sm text-foreground">
		{#if filteredScenes.length === scenes.length}
			Showing all {scenes.length} scenes
		{:else}
			Showing {filteredScenes.length} of {scenes.length} scenes
		{/if}
	</div>

	<!-- Scene Grid -->
	{#if filteredScenes.length > 0}
		<div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
			{#each filteredScenes as scene, i (scene.id)}
				<div in:scale={{ duration: 350, delay: Math.min(i * 50, 400) + 150, start: 0.95 }}>
					<SceneCard {scene} />
				</div>
			{/each}
		</div>
	{:else}
		<div class="flex flex-col items-center justify-center py-16 text-center">
		<Search class="mb-4 h-16 w-16 text-foreground opacity-50" strokeWidth={1.5} />
		<h3 class="text-lg font-semibold text-foreground">No scenes found</h3>
		<p class="mt-1 text-sm text-foreground/70">Try adjusting your filters</p>
		</div>
	{/if}
</div>
