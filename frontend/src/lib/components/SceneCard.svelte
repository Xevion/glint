<script lang="ts">
	import type { Scene } from '$lib/bindings';
	import {
		getBiomeDisplayName,
		getDimensionDisplayName,
		getWeatherDisplayName,
		hashStringToNumber
	} from '$lib/utils/display';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { comparisonStore } from '$lib/stores/comparison.svelte';
	import { cn } from '$lib/utils';

	interface Props {
		scene: Scene;
		class?: string;
	}

	let { scene, class: className }: Props = $props();

	// Parse tags from JSON string (with error handling)
	function parseTags(tagsJson: string | null): string[] {
		if (!tagsJson) return [];
		try {
			const parsed: unknown = JSON.parse(tagsJson);
			if (!Array.isArray(parsed)) {
				console.warn('SceneCard: tags is not an array', typeof parsed);
				return [];
			}
			return parsed as string[];
		} catch (e) {
			console.warn('SceneCard: failed to parse tags JSON', e);
			return [];
		}
	}

	const tags = $derived<string[]>(parseTags(scene.tags));

	// Determine time of day from ticks (0-24000, where 0=6am, 6000=noon, 18000=midnight)
	function getTimeOfDay(ticks: number): string {
		const normalizedTicks = ticks % 24000;
		if (normalizedTicks < 1000) return 'dawn';
		if (normalizedTicks < 11000) return 'day';
		if (normalizedTicks < 13000) return 'dusk';
		return 'night';
	}

	const timeOfDay = $derived(getTimeOfDay(scene.time_of_day_ticks));

	const wallpaperIndex = $derived(hashStringToNumber(scene.id) % 50);

	let isHovered = $state(false);
	const isSelected = $derived(comparisonStore.isSceneSelected(scene.id));
	const hasAnySelection = $derived(comparisonStore.hasSceneSelection);

	function handleCardClick(e: MouseEvent) {
		const target = e.target as HTMLElement;

		if (target.closest('[data-checkbox]') || target.closest('[data-clickable]')) {
			return;
		}

		if (hasAnySelection) {
			e.preventDefault();
			comparisonStore.toggleScene(scene.id);
			return;
		}

		void goto(resolve(`/scenes/${scene.slug}`), { invalidateAll: true });
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			if (hasAnySelection) {
				comparisonStore.toggleScene(scene.id);
			} else {
				void goto(resolve(`/scenes/${scene.slug}`), { invalidateAll: true });
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
		'border border-border shadow-sm',
		'focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none',
		hasAnySelection
			? 'cursor-default'
			: 'cursor-pointer hover:-translate-y-1 hover:border-primary/50 hover:shadow-lg',
		className
	)}
>
	<!-- Thumbnail Image -->
	<div class="relative aspect-video w-full overflow-hidden">
		<img
			src="/wallpapers/{wallpaperIndex}.jpg"
			alt="{scene.name} scene preview"
			class="h-full w-full object-cover transition-transform duration-500 ease-out group-hover:scale-105"
			style="will-change: transform; backface-visibility: hidden;"
			loading="lazy"
		/>

		<!-- Gradient overlay -->
		<div
			class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent"
		></div>

		<!-- Time badge - top left -->
		<div class="absolute top-3 left-3 flex">
			<span
				class="inline-flex items-center gap-1.5 rounded-md bg-blue-500/80 px-2 py-1 text-xs font-medium text-white capitalize shadow-sm backdrop-blur-sm"
			>
				<svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
					/>
				</svg>
				{timeOfDay}
			</span>
		</div>

		<!-- Compare checkbox - top right -->
		<button
			type="button"
			data-checkbox
			onclick={handleCheckboxClick}
			class={cn(
				'absolute top-3 right-3 flex h-5 w-5 cursor-pointer items-center justify-center rounded border-2 transition-all duration-200',
				isSelected
					? 'border-primary bg-primary'
					: 'border-gray-400 bg-gray-900/70 backdrop-blur-sm hover:border-gray-300',
				isSelected || isHovered || hasAnySelection ? 'opacity-100' : 'opacity-0',
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

	<!-- Card Body -->
	<div class="flex flex-1 flex-col gap-3 p-4">
		<!-- Header -->
		<div class="space-y-1.5">
			<a
				href={resolve(`/scenes/${scene.slug}`)}
				data-clickable
				onclick={(e) => {
					e.stopPropagation();
				}}
				class={cn(
					'line-clamp-1 block text-lg font-semibold text-card-foreground transition-colors hover:text-primary',
					hasAnySelection ? 'cursor-pointer' : ''
				)}
			>
				{scene.name}
			</a>
			<div class="flex items-center gap-2">
				{#if scene.biome}
					<span
						class="inline-flex items-center rounded-md bg-green-500/10 px-2 py-0.5 text-xs font-medium text-green-700 ring-1 ring-green-500/20 dark:text-green-400"
					>
						{getBiomeDisplayName(scene.biome)}
					</span>
				{/if}
				<span
					class="inline-flex items-center rounded-md bg-muted px-2 py-0.5 text-xs font-medium text-muted-foreground"
				>
					{getDimensionDisplayName(scene.dimension)}
				</span>
			</div>
		</div>

		<!-- Description -->
		{#if scene.description}
			<p class="line-clamp-2 flex-1 text-sm text-muted-foreground">
				{scene.description}
			</p>
		{:else}
			<p class="flex-1 text-sm text-muted-foreground/50 italic">No description available</p>
		{/if}

		<!-- Feature Tags -->
		{#if tags.length > 0}
			<div class="flex flex-wrap gap-1.5">
				{#each tags.slice(0, 4) as tag (tag)}
					<span
						class="inline-flex items-center rounded-md bg-muted/50 px-1.5 py-0.5 text-[11px] text-muted-foreground ring-1 ring-border/50"
					>
						{tag}
					</span>
				{/each}
				{#if tags.length > 4}
					<span
						class="inline-flex items-center rounded-md bg-muted/50 px-1.5 py-0.5 text-[11px] text-muted-foreground ring-1 ring-border/50"
					>
						+{tags.length - 4} more
					</span>
				{/if}
			</div>
		{/if}

		<!-- Footer -->
		<div class="mt-auto flex items-center justify-between border-t border-border pt-3">
			<div class="flex items-center gap-2 text-xs text-muted-foreground">
				<span>{getWeatherDisplayName(scene.weather)}</span>
				<span>•</span>
				<span class="capitalize">Medium scene</span>
			</div>
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
