<script lang="ts">
	import ShaderCard from '$lib/components/ShaderCard.svelte';
	import { getAllShaders, type PerformanceTier, type ShaderStyle } from '$lib/data/mock';
	import Button from '$lib/components/ui/button/button.svelte';

	const shaders = getAllShaders();

	// Filter state
	let selectedTier = $state<PerformanceTier | null>(null);
	let selectedStyle = $state<ShaderStyle | null>(null);
	let searchQuery = $state('');
	let sortBy = $state<'name' | 'downloads' | 'updated'>('downloads');

	const tiers: PerformanceTier[] = ['potato', 'low', 'medium', 'high', 'ultra'];
	const styles: ShaderStyle[] = [
		'realistic',
		'fantasy',
		'vibrant',
		'minimal',
		'retro',
		'cinematic'
	];

	const filteredShaders = $derived(() => {
		let result = shaders.filter((shader) => {
			if (selectedTier && shader.tier !== selectedTier) return false;
			if (selectedStyle && shader.style !== selectedStyle) return false;
			if (searchQuery && !shader.name.toLowerCase().includes(searchQuery.toLowerCase()))
				return false;
			return true;
		});

		// Sort
		result = [...result].sort((a, b) => {
			switch (sortBy) {
				case 'name':
					return a.name.localeCompare(b.name);
				case 'downloads':
					return b.downloadCount - a.downloadCount;
				case 'updated':
					return new Date(b.lastUpdated).getTime() - new Date(a.lastUpdated).getTime();
				default:
					return 0;
			}
		});

		return result;
	});

	const hasFilters = $derived(
		selectedTier !== null || selectedStyle !== null || searchQuery !== ''
	);
</script>

<div class="container mx-auto px-4 py-8">
	<!-- Header -->
	<div class="animate-fade-in-down mb-8">
		<h1 class="mb-2 text-4xl font-bold tracking-tight">Shader Packs</h1>
		<p class="text-lg text-muted-foreground">
			Browse {shaders.length} shader packs with standardized captures and performance metrics
		</p>
	</div>

	<!-- Filters -->
	<div class="animate-fade-in-up animation-delay-100 mb-6 space-y-4 rounded-xl bg-card p-4">
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
				<option value="downloads">Most Downloads</option>
				<option value="updated">Recently Updated</option>
				<option value="name">Name A-Z</option>
			</select>
		</div>

		<div class="flex flex-wrap items-center gap-4">
			<!-- Performance Tier -->
			<div class="flex items-center gap-2">
				<span class="text-sm font-medium text-muted-foreground">Performance:</span>
				<div class="flex flex-wrap gap-1">
					{#each tiers as tier}
						<button
							onclick={() => (selectedTier = selectedTier === tier ? null : tier)}
							class="rounded-lg px-3 py-1.5 text-xs font-medium capitalize transition-colors
								{selectedTier === tier
								? 'bg-primary text-primary-foreground'
								: 'bg-muted text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
						>
							{tier}
						</button>
					{/each}
				</div>
			</div>

			<div class="hidden h-6 w-px bg-border sm:block"></div>

			<!-- Style -->
			<div class="flex items-center gap-2">
				<span class="text-sm font-medium text-muted-foreground">Style:</span>
				<div class="flex flex-wrap gap-1">
					{#each styles as style}
						<button
							onclick={() => (selectedStyle = selectedStyle === style ? null : style)}
							class="rounded-lg px-3 py-1.5 text-xs font-medium capitalize transition-colors
								{selectedStyle === style
								? 'bg-primary text-primary-foreground'
								: 'bg-muted text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
						>
							{style}
						</button>
					{/each}
				</div>
			</div>

			{#if hasFilters}
				<Button
					variant="ghost"
					size="sm"
					onclick={() => {
						selectedTier = null;
						selectedStyle = null;
						searchQuery = '';
					}}
				>
					Clear filters
				</Button>
			{/if}
		</div>
	</div>

	<!-- Results count -->
	<div class="animate-fade-in animation-delay-200 mb-4 text-sm text-muted-foreground">
		{#if filteredShaders().length === shaders.length}
			Showing all {shaders.length} shaders
		{:else}
			Showing {filteredShaders().length} of {shaders.length} shaders
		{/if}
	</div>

	<!-- Shader Grid -->
	{#if filteredShaders().length > 0}
		<div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
			{#each filteredShaders() as shader, i (shader.id)}
				<div class="animate-fade-in-scale" style="animation-delay: {Math.min(i * 50, 400) + 150}ms">
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
