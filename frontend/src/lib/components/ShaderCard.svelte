<script lang="ts">
import { goto } from '$app/navigation';
import { resolve } from '$app/paths';
import type { ShaderListItem } from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { ImageOverlay } from '$lib/components/ui/image-overlay';
import { comparisonStore } from '$lib/stores/comparison.svelte';
import { cn } from '$lib/utils';
import { formatVersion, getCurseforgeUrl, getModrinthUrl } from '$lib/utils/display';
import { Check, ExternalLink } from '@lucide/svelte';
import BrandIcon from './icons/BrandIcon.svelte';

interface Props {
	shader: ShaderListItem;
	class?: string;
}

let { shader, class: className }: Props = $props();

let isHovered = $state(false);
const selectionMode = $derived(comparisonStore.selectionMode);
const isSelected = $derived(selectionMode && comparisonStore.isSelected(shader.id));
const hasAnySelection = $derived(selectionMode && comparisonStore.hasSelection());

const modrinthUrl = $derived(getModrinthUrl(shader.modrinth_id));
const curseforgeUrl = $derived(getCurseforgeUrl(shader.curseforge_id));

function handleCardClick(e: MouseEvent) {
	const target = e.target as HTMLElement;

	if (target.closest('[data-external-link]') || target.closest('[data-checkbox]')) {
		return;
	}

	if (target.closest('[data-clickable]')) {
		return;
	}

	if (hasAnySelection) {
		e.preventDefault();
		comparisonStore.select(shader);
		return;
	}

	void goto(resolve('/shaders/[slug]', { slug: shader.slug }), { invalidateAll: true });
}

function handleKeyDown(e: KeyboardEvent) {
	if (e.key === 'Enter' || e.key === ' ') {
		e.preventDefault();
		if (hasAnySelection) {
			comparisonStore.select(shader);
		} else {
			void goto(resolve('/shaders/[slug]', { slug: shader.slug }), { invalidateAll: true });
		}
	}
}

function handleCheckboxClick(e: MouseEvent) {
	e.stopPropagation();
	comparisonStore.select(shader);
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
		'border border-border shadow-theme-sm',
		'focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none',
		hasAnySelection
			? 'cursor-default'
			: 'cursor-pointer hover:-translate-y-1 hover:border-primary/50 hover:shadow-theme-lg',
		className
	)}
>
	<!-- Thumbnail Image -->
	<div class="relative">
		<CaptureImage
		src={shader.image_path}
		thumbhash={shader.thumbhash}
			preset="card"
			alt="{shader.name} preview"
			class="h-full w-full object-cover"
			containerClass="aspect-video w-full overflow-hidden"

		/>

		<!-- Gradient overlay -->
		<ImageOverlay />

		<!-- Compare checkbox - only visible in selection mode -->
		{#if selectionMode}
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
					<Check class="h-3 w-3 text-primary-foreground" strokeWidth={3} />
				{/if}
			</button>
		{/if}
	</div>

	<!-- Card Body -->
	<div class="flex flex-1 flex-col gap-2 px-3 pt-2.5 pb-2">
		<!-- Header -->
		<div class="flex items-baseline gap-x-2 overflow-hidden">
			<a
				href={resolve('/shaders/[slug]', { slug: shader.slug })}
				data-clickable
				onclick={(e) => {
					e.stopPropagation();
				}}
				class={cn(
					'block truncate text-base font-semibold text-card-foreground transition-colors hover:text-primary',
					hasAnySelection ? 'cursor-pointer' : ''
				)}
			>
				{shader.name}
			</a>
			<span class="shrink-0 text-sm text-muted-foreground">by <span class="font-medium text-card-foreground">{shader.authors[0]?.name ?? 'Unknown'}</span></span>
		</div>

		<!-- Description -->
		{#if shader.description}
			<p class="line-clamp-2 flex-1 text-sm text-muted-foreground">
				{shader.description}
			</p>
		{:else}
			<p class="flex-1 text-sm text-muted-foreground/50 italic">No description available</p>
		{/if}

		<!-- Tags: Style & Features -->
		{#if shader.categories.length > 0 || shader.features.length > 0}
			<div class="flex flex-wrap gap-1.5">
				{#each shader.categories as category (category.id)}
					<span
						class="inline-flex items-center rounded-md bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary capitalize ring-1 ring-primary/20"
					>
						{category.name}
					</span>
				{/each}
				{#each shader.features as feature (feature.id)}
					<span
						class="inline-flex items-center rounded-md bg-muted/50 px-1.5 py-0.5 text-[11px] text-muted-foreground ring-1 ring-border/50"
					>
						{feature.name}
					</span>
				{/each}
			</div>
		{/if}

		<!-- Footer: Version & Links -->
		<div class="mt-auto flex items-center justify-between border-t border-border pt-2">
			<div class="flex min-w-0 items-center gap-3">
				<!-- Version -->
				<span class="inline-flex shrink-0 items-center gap-1 text-xs">
					<span class="font-medium text-card-foreground">{shader.latest_version ? formatVersion(shader.latest_version) : '\u2014'}</span>
				</span>
			</div>

			<!-- External links don't need resolve() - only internal app routes -->
			<!-- eslint-disable svelte/no-navigation-without-resolve -->
			<div class="flex items-center gap-1">
				{#if modrinthUrl}
					<a
						href={modrinthUrl}
						target="_blank"
						rel="noopener noreferrer"
						data-external-link
						data-clickable
						class="group/modrinth inline-flex h-7 w-7 cursor-pointer items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent"
						title="View on Modrinth"
					>
						<BrandIcon name="modrinth" colorOnHover />
					</a>
				{/if}
				{#if curseforgeUrl}
					<a
						href={curseforgeUrl}
						target="_blank"
						rel="noopener noreferrer"
						data-external-link
						data-clickable
						class="group/curseforge inline-flex h-7 w-7 cursor-pointer items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent"
						title="View on CurseForge"
					>
						<BrandIcon name="curseforge" colorOnHover />
					</a>
				{/if}
				{#if shader.website_url}
					<a
						href={shader.website_url}
						target="_blank"
						rel="noopener noreferrer"
						data-external-link
						data-clickable
						class="inline-flex h-7 w-7 cursor-pointer items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent"
						title="Visit website"
					>
						<ExternalLink class="h-4 w-4" />
					</a>
				{/if}
			</div>
			<!-- eslint-enable svelte/no-navigation-without-resolve -->
		</div>
	</div>
</div>
