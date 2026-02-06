<script lang="ts">
import ShaderCard from '$lib/components/ShaderCard.svelte';
import { Button } from '$lib/components/ui/button';
import { Search } from '@lucide/svelte';
import { fly, scale } from 'svelte/transition';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
const shaders = $derived(data.shaders);

// Filter state
let searchQuery = $state('');
let sortBy = $state<'name' | 'updated'>('updated');

const filteredShaders = $derived.by(() => {
	let result = shaders.filter((shader: (typeof shaders)[0]) => {
		if (searchQuery && !shader.name.toLowerCase().includes(searchQuery.toLowerCase())) return false;
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

<div class="py-6">
	<!-- Minimal inline header -->
	<div in:fly={{ y: -10, duration: 400 }} class="mb-6 flex flex-wrap items-center gap-4">
		<!-- Title with count -->
		<h1 class="text-2xl font-semibold tracking-tight">
			Shaders
			<span class="ml-1 text-lg font-normal text-muted-foreground">({filteredShaders.length})</span>
		</h1>

		<div class="flex w-full flex-wrap items-center gap-3 sm:ml-auto sm:w-auto">
			<!-- Search -->
			<div class="relative">
				<Search
					class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground"
					strokeWidth={2}
				/>
				<input
					type="text"
					placeholder="Search..."
					bind:value={searchQuery}
					class="h-9 w-full rounded-lg border border-input bg-background/50 pr-3 pl-9 text-sm backdrop-blur-sm transition-all placeholder:text-muted-foreground focus:w-full focus:outline-none focus:ring-2 focus:ring-ring sm:w-48 sm:focus:w-64"
				/>
			</div>

			<!-- Sort -->
			<select
				bind:value={sortBy}
				class="h-9 rounded-lg border border-input bg-background/50 backdrop-blur-sm px-3 text-sm focus:ring-2 focus:ring-ring focus:outline-none"
			>
				<option value="updated">Recent</option>
				<option value="name">A-Z</option>
			</select>

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
		</div>
	</div>

	<!-- Shader Grid -->
	{#if filteredShaders.length > 0}
		<div class="grid grid-cols-1 gap-5 sm:[grid-template-columns:repeat(auto-fit,minmax(300px,1fr))]">
			{#each filteredShaders as shader, i (shader.id)}
				<div in:scale={{ duration: 350, delay: Math.min(i * 50, 400) + 150, start: 0.95 }}>
					<ShaderCard {shader} />
				</div>
			{/each}
		</div>
	{:else}
		<div class="flex flex-col items-center justify-center py-16 text-center">
			<Search class="mb-4 h-16 w-16 text-muted-foreground opacity-50" strokeWidth={1.5} />
			<h3 class="text-lg font-semibold text-muted-foreground">No shaders found</h3>
			<p class="mt-1 text-sm text-muted-foreground/70">Try adjusting your filters</p>
		</div>
	{/if}
</div>
