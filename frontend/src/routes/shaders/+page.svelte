<script lang="ts">
	import { fly, fade, scale } from 'svelte/transition';
	import { Button } from '$lib/components/ui/button';
	import ShaderCard from '$lib/components/ShaderCard.svelte';
	import type { PageData } from './$types';

	let { data } = $props<{ data: PageData }>();
	const shaders = $derived(data.shaders);

	// Filter state
	let searchQuery = $state('');
	let sortBy = $state<'name' | 'updated'>('updated');

	const filteredShaders = $derived.by(() => {
		let result = shaders.filter((shader: (typeof shaders)[0]) => {
			if (searchQuery && !shader.name.toLowerCase().includes(searchQuery.toLowerCase()))
				return false;
			return true;
		});

		// Sort
		result = [...result].sort((a, b) => {
			switch (sortBy) {
				case 'name':
					return a.name.localeCompare(b.name);
				case 'updated':
					return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime();
				default:
					return 0;
			}
		});

		return result;
	});

	const hasFilters = $derived(searchQuery !== '');
</script>

<div class="container mx-auto px-4 py-8">
	<!-- Header -->
	<div in:fly={{ y: -10, duration: 400 }} class="mb-8">
		<h1 class="mb-2 text-4xl font-bold tracking-tight">Shader Packs</h1>
		<p class="text-lg text-muted-foreground">
			Browse {shaders.length} shader packs with standardized captures and performance metrics
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
					placeholder="Search shaders..."
					bind:value={searchQuery}
					class="h-10 w-full rounded-lg border border-input bg-background pr-4 pl-10 text-sm placeholder:text-muted-foreground focus:ring-2 focus:ring-ring focus:outline-none"
				/>
			</div>

			<!-- Sort -->
			<select
				bind:value={sortBy}
				class="h-10 rounded-lg border border-input bg-background px-3 text-sm focus:ring-2 focus:ring-ring focus:outline-none"
			>
				<option value="updated">Recently Updated</option>
				<option value="name">Name A-Z</option>
			</select>

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
		{#if filteredShaders.length === shaders.length}
			Showing all {shaders.length} shaders
		{:else}
			Showing {filteredShaders.length} of {shaders.length} shaders
		{/if}
	</div>

	<!-- Shader Grid -->
	{#if filteredShaders.length > 0}
		<div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
			{#each filteredShaders as shader, i (shader.id)}
				<div in:scale={{ duration: 350, delay: Math.min(i * 50, 400) + 150, start: 0.95 }}>
					<ShaderCard {shader} />
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
			<h3 class="text-lg font-semibold text-muted-foreground">No shaders found</h3>
			<p class="mt-1 text-sm text-muted-foreground/70">Try adjusting your filters</p>
		</div>
	{/if}
</div>
