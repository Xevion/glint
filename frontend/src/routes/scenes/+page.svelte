<script lang="ts">
import ErrorBanner from '$lib/components/ErrorBanner.svelte';
import { ItemGrid } from '$lib/components/item-grid';
import Meta from '$lib/components/Meta.svelte';
import SceneCard from '$lib/components/SceneCard.svelte';
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
import { AlertTriangle, Search } from '@lucide/svelte';
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
		<!-- eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-return -->
		<ItemGrid
			items={filteredScenes}
			key={(s) => s.id}
			size="small"
			empty={hasFilters
				? { icon: Search, title: 'No scenes found', message: 'Try adjusting your filters' }
				: { icon: Search, title: 'No scenes yet', message: 'Scenes will appear here once they\'ve been created' }}
		>
			{#snippet card(scene)}
				<SceneCard {scene} />
			{/snippet}
		</ItemGrid>
		<!-- eslint-enable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-return -->
	{/if}
</div>
