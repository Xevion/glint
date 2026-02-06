<script lang="ts">
import type { ShaderListItem } from '$lib/bindings';
import {
	formatDate,
	getModrinthUrl,
	getCurseforgeUrl,
	hashStringToNumber
} from '$lib/utils/display';
import { goto } from '$app/navigation';
import { resolve } from '$app/paths';
import { comparisonStore } from '$lib/stores/comparison.svelte';
import { cn } from '$lib/utils';
import BrandIcon from './icons/BrandIcon.svelte';
import { Check, Layers, ExternalLink } from '@lucide/svelte';

interface Props {
	shader: ShaderListItem;
	class?: string;
}

let { shader, class: className }: Props = $props();

const wallpaperIndex = $derived(hashStringToNumber(shader.id) % 50);

let isHovered = $state(false);
const isSelected = $derived(comparisonStore.isSelected(shader.id));
const hasAnySelection = $derived(comparisonStore.hasSelection());

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

	void goto(resolve('/shaders/[id]', { id: shader.slug }), { invalidateAll: true });
}

function handleKeyDown(e: KeyboardEvent) {
	if (e.key === 'Enter' || e.key === ' ') {
		e.preventDefault();
		if (hasAnySelection) {
			comparisonStore.select(shader);
		} else {
			void goto(resolve('/shaders/[id]', { id: shader.slug }), { invalidateAll: true });
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
			src={shader.thumbnail_url ?? shader.icon_url ?? `/wallpapers/${wallpaperIndex}.jpg`}
			alt="{shader.name} preview"
			class="h-full w-full object-cover transition-transform duration-500 ease-out group-hover:scale-105"
			style="will-change: transform; backface-visibility: hidden;"
			loading="lazy"
		/>

		<!-- Gradient overlay -->
		<div
			class="absolute inset-0 bg-linear-to-t from-black/60 via-transparent to-transparent"
		></div>

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
				<Check class="h-3 w-3 text-primary-foreground" strokeWidth={3} />
			{/if}
		</button>
	</div>

	<!-- Card Body -->
	<div class="flex flex-1 flex-col gap-3 p-4">
		<!-- Header -->
		<div class="space-y-1">
			<a
				href={resolve('/shaders/[id]', { id: shader.slug })}
				data-clickable
				onclick={(e) => {
					e.stopPropagation();
				}}
				class={cn(
					'line-clamp-1 block text-lg font-semibold text-card-foreground transition-colors hover:text-primary',
					hasAnySelection ? 'cursor-pointer' : ''
				)}
			>
				{shader.name}
			</a>
			<div class="flex items-center gap-2 text-sm text-muted-foreground">
				<span>by</span>
				<span class="font-medium text-card-foreground">{shader.authors[0]?.name ?? 'Unknown Author'}</span>
				<span class="text-muted-foreground/50">·</span>
				<span>{formatDate(shader.updated_at)}</span>
			</div>
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
		<div class="mt-auto flex items-center justify-between border-t border-border pt-3">
			<div class="flex items-center gap-3">
				<!-- Version -->
				<span class="inline-flex items-center gap-1 text-xs">
					<span class="text-muted-foreground/70">v</span>
					<span class="font-medium text-card-foreground">{shader.latest_version ?? '\u2014'}</span>
				</span>

				<!-- Separator -->
				<span class="h-4 w-px bg-border"></span>

				<!-- MC Version -->
				<span class="inline-flex items-center gap-1.5 text-xs text-muted-foreground">
					<Layers class="h-3.5 w-3.5" />
					{shader.game_versions ?? '\u2014'}
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
						class="group/modrinth inline-flex h-8 w-8 cursor-pointer items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent"
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
						class="group/curseforge inline-flex h-8 w-8 cursor-pointer items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent"
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
						class="inline-flex h-8 w-8 cursor-pointer items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent"
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
