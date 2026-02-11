<script lang="ts">
import ShaderCard from '$lib/components/ShaderCard.svelte';
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
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

<svelte:head><title>Shaders - Glint</title></svelte:head>

<div class="py-6">
	<!-- Minimal inline header -->
	<div in:fly={{ y: -10, duration: 400 }} class="mb-6 flex flex-wrap items-center gap-4">
		<!-- Title with count -->
		<h1 class="text-2xl font-semibold tracking-tight">
			Shaders
			<span class="ml-1 text-lg font-normal text-foreground">({filteredShaders.length})</span>
		</h1>

		<div class="flex w-full flex-wrap items-center gap-3 sm:ml-auto sm:w-auto">
			<!-- Search -->
			<div class="relative">
			<Search
				class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-foreground"
				strokeWidth={2}
			/>
			<Input
				type="text"
				placeholder="Search..."
				bind:value={searchQuery}
				class="w-full bg-background/50 pr-3 pl-9 backdrop-blur-sm transition-all focus:w-full sm:w-48 sm:focus:w-64"
			/>
			</div>

			<!-- Sort -->
		<select
			bind:value={sortBy}
			class="h-9 rounded-md border border-input bg-background/50 px-3 text-sm shadow-xs backdrop-blur-sm transition-[color,box-shadow] outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:bg-input/30"
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
		<div class="grid grid-cols-1 gap-6 sm:[grid-template-columns:repeat(auto-fit,minmax(300px,1fr))]">
			{#each filteredShaders as shader, i (shader.id)}
				<div in:scale={{ duration: 350, delay: Math.min(i * 50, 400) + 150, start: 0.95 }}>
					<ShaderCard {shader} />
				</div>
			{/each}
		</div>
	{:else}
		<div class="flex flex-col items-center justify-center py-16 text-center">
		<Search class="mb-4 h-16 w-16 text-foreground opacity-50" strokeWidth={1.5} />
		<h3 class="text-lg font-semibold text-foreground">No shaders found</h3>
		<p class="mt-1 text-sm text-foreground/70">Try adjusting your filters</p>
		</div>
	{/if}
</div>
