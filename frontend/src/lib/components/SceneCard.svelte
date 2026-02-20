<script lang="ts" module>
import { graphql, type ResultOf } from '$lib/graphql';

/** Shared fragment for scene card fields — used by browse and home queries. */
export const SceneCardFragment = graphql(`
	fragment SceneCardFields on SceneNode @_unmask {
		slug
		name
		description
		dimension
		imagePath
		thumbhash
		captureCount
		version {
			timeOfDayTicks
			weather
			biome
		}
	}
`);

/** Type derived from the SceneCardFields fragment via gql.tada. */
export type SceneCardScene = ResultOf<typeof SceneCardFragment>;
</script>

<script lang="ts">
import { resolve } from '$app/paths';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { ImageOverlay } from '$lib/components/ui/image-overlay';
import { StatusBadge } from '$lib/components/ui/status-badge';
import { cn } from '$lib/utils';
import {
	formatTimeTicks,
	getBiomeDisplayName,
	getDimensionDisplayName,
	getWeatherDisplayName
} from '$lib/utils/format';
import { ArrowRight, Sun } from '@lucide/svelte';

interface Props {
	scene: SceneCardScene;
	class?: string;
}

let { scene, class: className }: Props = $props();

const timeOfDay = $derived(scene.version ? formatTimeTicks(scene.version.timeOfDayTicks) : null);
</script>

<a
	href={resolve('/scenes/[slug]', { slug: scene.slug })}
	class={cn(
		'group relative flex flex-col overflow-hidden rounded-xl bg-card transition-all duration-300',
		'border border-border shadow-theme-sm',
		'focus-visible:outline-2 focus-visible:outline-primary focus-visible:outline-offset-2',
		'hover:-translate-y-1 hover:border-primary/50 hover:shadow-theme-lg',
		className
	)}
>
	<!-- Thumbnail Image -->
	<div class="relative">
	<CaptureImage
	src={scene.imagePath}
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
		{#if timeOfDay}
			<div class="absolute top-3 left-3 flex">
				<span
					class="inline-flex items-center gap-1.5 rounded-md bg-info/80 px-2 py-1 text-xs font-medium text-white capitalize shadow-theme-sm backdrop-blur-sm"
				>
					<Sun class="h-3 w-3" strokeWidth={2} />
					{timeOfDay}
				</span>
			</div>
		{/if}
	</div>

	<!-- Card Body -->
	<div class="flex flex-1 flex-col gap-3 p-4">
		<!-- Header -->
		<div class="space-y-1.5">
		<span class="line-clamp-1 block text-lg font-semibold text-card-foreground">
			{scene.name}
		</span>
			<div class="flex items-center gap-2">
		{#if scene.version?.biome}
			<StatusBadge status="active">
				{getBiomeDisplayName(scene.version.biome)}
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
			{#if scene.version}<span>{getWeatherDisplayName(scene.version.weather)}</span>{/if}
			<span>•</span>
				<span>{scene.captureCount} capture{scene.captureCount !== 1 ? 's' : ''}</span>
			</div>
			<span
				class="inline-flex items-center gap-1 text-xs font-medium text-primary transition-transform group-hover:translate-x-1"
			>
				View
				<ArrowRight class="h-3 w-3" strokeWidth={2} />
			</span>
		</div>
	</div>
</a>
