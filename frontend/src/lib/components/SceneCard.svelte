<script lang="ts">
import { goto } from '$app/navigation';
import { resolve } from '$app/paths';
import type { SceneListItem } from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { cn } from '$lib/utils';
import { ImageOverlay } from '$lib/components/ui/image-overlay';
import {
	getBiomeDisplayName,
	getDimensionDisplayName,
	getWeatherDisplayName
} from '$lib/utils/display';
import { StatusBadge } from '$lib/components/ui/status-badge';
import { ArrowRight, Sun } from '@lucide/svelte';

interface Props {
	scene: SceneListItem;
	class?: string;
}

let { scene, class: className }: Props = $props();

// Determine time of day from ticks (0-24000, where 0=6am, 6000=noon, 18000=midnight)
function getTimeOfDay(ticks: number): string {
	const normalizedTicks = ticks % 24000;
	if (normalizedTicks < 1000) return 'dawn';
	if (normalizedTicks < 11000) return 'day';
	if (normalizedTicks < 13000) return 'dusk';
	return 'night';
}

const timeOfDay = $derived(getTimeOfDay(scene.time_of_day_ticks));

function handleCardClick() {
	void goto(resolve('/scenes/[id]', { id: scene.slug }), { invalidateAll: true });
}

function handleKeyDown(e: KeyboardEvent) {
	if (e.key === 'Enter' || e.key === ' ') {
		e.preventDefault();
		void goto(resolve('/scenes/[id]', { id: scene.slug }), { invalidateAll: true });
	}
}
</script>

<div
	role="button"
	tabindex="0"
	onclick={handleCardClick}
	onkeydown={handleKeyDown}
	class={cn(
		'group relative flex flex-col overflow-hidden rounded-xl bg-card transition-all duration-300',
		'border border-border shadow-theme-sm',
		'focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none',
		'cursor-pointer hover:-translate-y-1 hover:border-primary/50 hover:shadow-theme-lg',
		className
	)}
>
	<!-- Thumbnail Image -->
	<div class="relative">
		<CaptureImage
			src={scene.image_url}
			thumbhash={scene.thumbhash}
			preset="card"
			alt="{scene.name} scene preview"
			class="h-full w-full object-cover transition-all duration-500 ease-out group-hover:scale-105"
			containerClass="aspect-video w-full overflow-hidden"
			style="will-change: transform; backface-visibility: hidden;"
		/>

		<!-- Gradient overlay -->
		<ImageOverlay />

		<!-- Time badge - top left -->
		<div class="absolute top-3 left-3 flex">
			<span
				class="inline-flex items-center gap-1.5 rounded-md bg-blue-500/80 px-2 py-1 text-xs font-medium text-white capitalize shadow-theme-sm backdrop-blur-sm"
			>
				<Sun class="h-3 w-3" strokeWidth={2} />
				{timeOfDay}
			</span>
		</div>
	</div>

	<!-- Card Body -->
	<div class="flex flex-1 flex-col gap-3 p-4">
		<!-- Header -->
		<div class="space-y-1.5">
			<a
				href={resolve('/scenes/[id]', { id: scene.slug })}
				data-clickable
				onclick={(e) => {
					e.stopPropagation();
				}}
				class="line-clamp-1 block text-lg font-semibold text-card-foreground transition-colors hover:text-primary"
			>
				{scene.name}
			</a>
			<div class="flex items-center gap-2">
			{#if scene.biome}
				<StatusBadge status="active">
					{getBiomeDisplayName(scene.biome)}
				</StatusBadge>
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

		<!-- Footer -->
		<div class="mt-auto flex items-center justify-between border-t border-border pt-3">
			<div class="flex items-center gap-2 text-xs text-muted-foreground">
				<span>{getWeatherDisplayName(scene.weather)}</span>
				<span>•</span>
				<span>{scene.capture_count} capture{scene.capture_count !== 1 ? 's' : ''}</span>
			</div>
			<span
				class="inline-flex items-center gap-1 text-xs font-medium text-primary transition-transform group-hover:translate-x-1"
			>
				View
				<ArrowRight class="h-3 w-3" strokeWidth={2} />
			</span>
		</div>
	</div>
</div>
