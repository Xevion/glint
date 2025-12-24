<script lang="ts">
	import { fly, fade, scale } from 'svelte/transition';
	import { Button } from '$lib/components/ui/button';
	import SceneCard from '$lib/components/SceneCard.svelte';
	import type { PageData } from './$types';

	let { data } = $props<{ data: PageData }>();
	const scenes = $derived(data.scenes);

	// Filter state
	let searchQuery = $state('');

	const filteredScenes = $derived.by(() => {
		return scenes.filter((scene: (typeof scenes)[0]) => {
			if (searchQuery && !scene.name.toLowerCase().includes(searchQuery.toLowerCase()))
				return false;
			return true;
		});
	});

	const hasFilters = $derived(searchQuery !== '');
</script>

<div class="container mx-auto px-4 py-8">
	<!-- Header -->
	<div in:fly={{ y: -10, duration: 400 }} class="mb-8">
		<h1 class="mb-2 text-4xl font-bold tracking-tight">Test Scenes</h1>
		<p class="text-lg text-foreground/70 dark:text-muted-foreground">
			{scenes.length} standardized environments for consistent shader comparison
		</p>
	</div>

	<!-- Filters -->
	<div in:fly={{ y: 10, duration: 400, delay: 100 }} class="mb-6 space-y-4 rounded-xl bg-card p-4">
		<div class="flex flex-wrap items-center gap-4">
			<!-- Search -->
			<div class="relative min-w-[200px] flex-1">
				<svg
					class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
					stroke-width="2"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z"
					/>
				</svg>
				<input
					type="text"
					placeholder="Search scenes..."
					bind:value={searchQuery}
					class="h-10 w-full rounded-lg border border-input bg-background pr-4 pl-10 text-sm placeholder:text-muted-foreground focus:ring-2 focus:ring-ring focus:outline-none"
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
	<div in:fade={{ duration: 300, delay: 200 }} class="mb-4 text-sm text-muted-foreground">
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
			<svg
				class="mb-4 h-16 w-16 text-muted-foreground/50"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				stroke-width="1.5"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z"
				/>
			</svg>
			<h3 class="text-lg font-semibold text-muted-foreground">No scenes found</h3>
			<p class="mt-1 text-sm text-muted-foreground/70">Try adjusting your filters</p>
		</div>
	{/if}
</div>
