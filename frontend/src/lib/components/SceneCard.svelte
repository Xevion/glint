<script lang="ts">
	import { cn } from '$lib/utils';
	import {
		type Scene,
		type Weather,
		getTimeColor,
		getWeatherColor,
		getBiomeColor,
		getTimeIconPath,
		getWeatherIconPath
	} from '$lib/data/mock';
	import { goto } from '$app/navigation';
	import { comparisonStore } from '$lib/stores/comparison.svelte';

	interface Props {
		scene: Scene;
		class?: string;
	}

	let { scene, class: className }: Props = $props();

	// Only show weather badge for weather types that significantly affect the scene
	const significantWeathers: Weather[] = ['cloudy', 'rain', 'storm', 'snow', 'fog'];
	const showWeather = significantWeathers.includes(scene.defaultWeather);

	let isHovered = $state(false);
	const isSelected = $derived(comparisonStore.isSceneSelected(scene.id));
	const hasAnySelection = $derived(comparisonStore.hasSceneSelection);

	function handleCardClick(e: MouseEvent) {
		const target = e.target as HTMLElement;

		// Always allow checkbox interactions
		if (target.closest('[data-checkbox]')) {
			return;
		}

		// Always allow clickable elements (title)
		if (target.closest('[data-clickable]')) {
			return;
		}

		// If any card is selected, clicking toggles this card's selection
		if (hasAnySelection) {
			e.preventDefault();
			comparisonStore.toggleScene(scene.id);
			return;
		}

		// Default: navigate to scene page
		goto(`/scenes/${scene.id}`);
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			if (hasAnySelection) {
				comparisonStore.toggleScene(scene.id);
			} else {
				goto(`/scenes/${scene.id}`);
			}
		}
	}

	function handleCheckboxClick(e: MouseEvent) {
		e.stopPropagation();
		comparisonStore.toggleScene(scene.id);
	}
</script>

<div
	role="button"
	tabindex="0"
	onclick={handleCardClick}
	onkeydown={handleKeyDown}
	onmouseenter={() => (isHovered = true)}
	onmouseleave={() => (isHovered = false)}
	class={cn(
		'group relative flex flex-col overflow-hidden rounded-xl bg-card transition-all duration-300',
		'hover:shadow-xl hover:shadow-primary/10',
		'focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none',
		hasAnySelection
			? 'cursor-default'
			: 'cursor-pointer hover:-translate-y-1 hover:ring-2 hover:ring-primary/50',
		className
	)}
>
	<!-- Inset Image Container -->
	<div class="relative aspect-video w-full overflow-hidden">
		<!-- Anti-aliasing fix for smooth zoom transforms -->
		<img
			src={scene.thumbnail}
			alt="{scene.name} scene preview"
			class="h-full w-full object-cover transition-transform duration-500 ease-out group-hover:scale-105"
			style="will-change: transform; backface-visibility: hidden;"
			loading="lazy"
		/>

		<!-- Gradient overlay -->
		<div
			class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent"
		></div>

		<!-- Time and Weather badges - top left (grouped together when both present) -->
		<div class="absolute top-3 left-3 flex">
			<!-- Time badge -->
			<span
				class={cn(
					'inline-flex items-center gap-1.5 px-2 py-1 text-xs font-medium capitalize shadow-sm backdrop-blur-sm',
					getTimeColor(scene.defaultTime),
					// Rounded on left, flat on right when weather is shown
					showWeather ? 'rounded-l-md' : 'rounded-md'
				)}
			>
				<svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
					<path stroke-linecap="round" stroke-linejoin="round" d={getTimeIconPath(scene.defaultTime)} />
				</svg>
				{scene.defaultTime}
			</span>
			<!-- Weather badge (only for significant weather) -->
			{#if showWeather}
				<span
					class={cn(
						'inline-flex items-center gap-1.5 rounded-r-md px-2 py-1 text-xs font-medium capitalize shadow-sm backdrop-blur-sm',
						getWeatherColor(scene.defaultWeather)
					)}
				>
					<svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d={getWeatherIconPath(scene.defaultWeather)}
						/>
					</svg>
					{scene.defaultWeather}
				</span>
			{/if}
		</div>

		<!-- Compare checkbox - top right -->
		<div class="absolute top-3 right-3">
			<button
				type="button"
				data-checkbox
				onclick={handleCheckboxClick}
				class={cn(
					'flex h-5 w-5 cursor-pointer items-center justify-center rounded border-2 transition-all duration-200',
					isSelected
						? 'border-primary bg-primary'
						: 'border-gray-400 bg-gray-900/70 backdrop-blur-sm hover:border-gray-300',
					// Visibility: show if selected, hovered, or any selection exists
					isSelected || isHovered || hasAnySelection
						? 'opacity-100'
						: 'pointer-events-none opacity-0',
					// Dim unchecked checkboxes when selection exists but not hovered
					!isSelected && hasAnySelection && !isHovered && 'opacity-60'
				)}
			>
				{#if isSelected}
					<svg
						class="h-3 w-3 text-primary-foreground"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="3"
					>
						<path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
					</svg>
				{/if}
			</button>
		</div>
	</div>

	<!-- Card Body -->
	<div class="flex flex-1 flex-col gap-3 p-4">
		<!-- Header -->
		<div class="space-y-1.5">
			<a
				href="/scenes/{scene.id}"
				data-clickable
				onclick={(e) => e.stopPropagation()}
				class={cn(
					'line-clamp-1 block text-lg font-semibold text-card-foreground transition-colors hover:text-primary',
					hasAnySelection ? 'cursor-pointer' : ''
				)}
			>
				{scene.name}
			</a>
			<div class="flex items-center gap-2">
				<span
					class={cn(
						'inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium',
						getBiomeColor(scene.biome)
					)}
				>
					{scene.biome}
				</span>
			</div>
		</div>

		<!-- Description -->
		<p class="line-clamp-2 flex-1 text-sm text-muted-foreground">
			{scene.description}
		</p>

		<!-- Features -->
		<div class="flex flex-wrap gap-1.5">
			{#each scene.features as feature}
				<span
					class="inline-flex items-center rounded-md bg-muted/50 px-1.5 py-0.5 text-[11px] text-muted-foreground ring-1 ring-border/50"
				>
					{feature}
				</span>
			{/each}
		</div>

		<!-- Footer -->
		<div class="mt-auto flex items-center justify-between border-t border-border pt-3">
			<span class="text-xs text-muted-foreground capitalize">
				{scene.complexity} scene
			</span>
			<span
				class="inline-flex items-center gap-1 text-xs font-medium text-primary transition-transform group-hover:translate-x-1"
			>
				View
				<svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3"
					/>
				</svg>
			</span>
		</div>
	</div>
</div>
