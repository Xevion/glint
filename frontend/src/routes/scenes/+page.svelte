<script lang="ts">
	import SceneCard from '$lib/components/SceneCard.svelte';
	import { getAllScenes, type TimeOfDay, type Weather, type Dimension } from '$lib/data/mock';
	import Button from '$lib/components/ui/button/button.svelte';

	const scenes = getAllScenes();

	// Filter state
	let selectedTime = $state<TimeOfDay | null>(null);
	let selectedWeather = $state<Weather | null>(null);
	let selectedDimension = $state<Dimension | null>(null);
	let searchQuery = $state('');

	const times: TimeOfDay[] = [
		'dawn',
		'morning',
		'noon',
		'afternoon',
		'sunset',
		'dusk',
		'night',
		'midnight'
	];
	const weathers: Weather[] = ['clear', 'cloudy', 'rain', 'storm', 'snow', 'fog'];
	const dimensions: Dimension[] = ['overworld', 'nether', 'end'];

	const filteredScenes = $derived(() => {
		return scenes.filter((scene) => {
			if (selectedTime && scene.defaultTime !== selectedTime) return false;
			if (selectedWeather && scene.defaultWeather !== selectedWeather) return false;
			if (selectedDimension && scene.dimension !== selectedDimension) return false;
			if (searchQuery && !scene.name.toLowerCase().includes(searchQuery.toLowerCase()))
				return false;
			return true;
		});
	});

	const hasFilters = $derived(
		selectedTime !== null ||
			selectedWeather !== null ||
			selectedDimension !== null ||
			searchQuery !== ''
	);
</script>

<div class="container mx-auto px-4 py-8">
	<!-- Header -->
	<div class="animate-fade-in-down mb-8">
		<h1 class="mb-2 text-4xl font-bold tracking-tight">Test Scenes</h1>
		<p class="text-lg text-foreground/70 dark:text-muted-foreground">
			{scenes.length} standardized environments for consistent shader comparison
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
					placeholder="Search scenes..."
					bind:value={searchQuery}
					class="h-10 w-full rounded-lg border border-input bg-background pr-4 pl-10 text-sm placeholder:text-muted-foreground focus:ring-2 focus:ring-ring focus:outline-none"
				/>
			</div>

			<!-- Dimension filter -->
			<div class="flex gap-1">
				{#each dimensions as dimension}
					<button
						onclick={() => (selectedDimension = selectedDimension === dimension ? null : dimension)}
						class="rounded-lg px-3 py-2 text-xs font-medium capitalize transition-colors
							{selectedDimension === dimension
							? 'bg-primary text-primary-foreground'
							: 'bg-muted text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
					>
						{dimension}
					</button>
				{/each}
			</div>
		</div>

		<div class="flex flex-wrap items-center gap-4">
			<!-- Time of Day -->
			<div class="flex items-center gap-2">
				<span class="text-sm font-medium text-muted-foreground">Time:</span>
				<div class="flex flex-wrap gap-1">
					{#each times as time}
						<button
							onclick={() => (selectedTime = selectedTime === time ? null : time)}
							class="rounded-lg px-2.5 py-1.5 text-xs font-medium capitalize transition-colors
								{selectedTime === time
								? 'bg-primary text-primary-foreground'
								: 'bg-muted text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
						>
							{time}
						</button>
					{/each}
				</div>
			</div>

			<div class="hidden h-6 w-px bg-border md:block"></div>

			<!-- Weather -->
			<div class="flex items-center gap-2">
				<span class="text-sm font-medium text-muted-foreground">Weather:</span>
				<div class="flex flex-wrap gap-1">
					{#each weathers as weather}
						<button
							onclick={() => (selectedWeather = selectedWeather === weather ? null : weather)}
							class="rounded-lg px-2.5 py-1.5 text-xs font-medium capitalize transition-colors
								{selectedWeather === weather
								? 'bg-primary text-primary-foreground'
								: 'bg-muted text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
						>
							{weather}
						</button>
					{/each}
				</div>
			</div>

			{#if hasFilters}
				<Button
					variant="ghost"
					size="sm"
					onclick={() => {
						selectedTime = null;
						selectedWeather = null;
						selectedDimension = null;
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
		{#if filteredScenes().length === scenes.length}
			Showing all {scenes.length} scenes
		{:else}
			Showing {filteredScenes().length} of {scenes.length} scenes
		{/if}
	</div>

	<!-- Scene Grid -->
	{#if filteredScenes().length > 0}
		<div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
			{#each filteredScenes() as scene, i (scene.id)}
				<div class="animate-fade-in-scale" style="animation-delay: {Math.min(i * 50, 400) + 150}ms">
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
