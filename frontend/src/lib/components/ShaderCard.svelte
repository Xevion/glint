<script lang="ts">
	import { cn } from '$lib/utils';
	import {
		type Shader,
		formatNumber,
		formatDate,
		getTierColor,
		getTierLabel,
		getStyleColor
	} from '$lib/data/mock';
	import { goto } from '$app/navigation';
	import { comparisonStore } from '$lib/stores/comparison.svelte';
	import TierIcon from './TierIcon.svelte';
	import BrandIcon from './icons/BrandIcon.svelte';

	interface Props {
		shader: Shader;
		class?: string;
	}

	let { shader, class: className }: Props = $props();

	let isHovered = $state(false);
	const isSelected = $derived(comparisonStore.isShaderSelected(shader.id));
	const hasAnySelection = $derived(comparisonStore.hasShaderSelection);

	function handleCardClick(e: MouseEvent) {
		const target = e.target as HTMLElement;

		// Always allow external links and checkbox interactions
		if (target.closest('[data-external-link]') || target.closest('[data-checkbox]')) {
			return;
		}

		// Always allow clickable elements (title, author link)
		if (target.closest('[data-clickable]')) {
			return;
		}

		// If any card is selected, clicking toggles this card's selection
		if (hasAnySelection) {
			e.preventDefault();
			comparisonStore.toggleShader(shader.id);
			return;
		}

		// Default: navigate to shader page
		goto(`/shaders/${shader.id}`);
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			if (hasAnySelection) {
				comparisonStore.toggleShader(shader.id);
			} else {
				goto(`/shaders/${shader.id}`);
			}
		}
	}

	function handleCheckboxClick(e: MouseEvent) {
		e.stopPropagation();
		comparisonStore.toggleShader(shader.id);
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
		'card-glow',
		'focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none',
		hasAnySelection
			? 'cursor-default'
			: 'cursor-pointer hover:-translate-y-1 hover:ring-2 hover:ring-primary/50',
		className
	)}
>
	<!-- Inset Image Container -->
	<div class="relative aspect-video w-full overflow-hidden">
		<img
			src={shader.thumbnail}
			alt="{shader.name} preview"
			class="h-full w-full object-cover transition-transform duration-500 ease-out group-hover:scale-105"
			style="will-change: transform; backface-visibility: hidden;"
			loading="lazy"
		/>

		<!-- Gradient overlay for text readability -->
		<div
			class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent"
		></div>

		<!-- Performance badge - top left -->
		<div class="absolute top-3 left-3">
			<span
				class={cn(
					'inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium shadow-sm backdrop-blur-sm',
					getTierColor(shader.tier)
				)}
			>
				<TierIcon tier={shader.tier} size={12} />
				{getTierLabel(shader.tier)}
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

		<!-- Quick stats overlay - bottom of image -->
		<div class="absolute right-3 bottom-3 left-3 flex items-center justify-between">
			<div class="flex items-center gap-3">
				<span class="inline-flex items-center gap-1 text-sm font-medium text-white drop-shadow-md">
					<svg
						class="h-4 w-4"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
						/>
					</svg>
					{formatNumber(shader.downloadCount)}
				</span>
				<span class="inline-flex items-center gap-1 text-sm font-medium text-white drop-shadow-md">
					<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
						<path
							d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"
						/>
					</svg>
					{formatNumber(shader.likes)}
				</span>
			</div>
		</div>
	</div>

	<!-- Card Body with padding -->
	<div class="flex flex-1 flex-col gap-3 p-4">
		<!-- Header: Title and Author -->
		<div class="space-y-1">
			<a
				href="/shaders/{shader.id}"
				data-clickable
				onclick={(e) => e.stopPropagation()}
				class={cn(
					'line-clamp-1 block text-lg font-semibold text-card-foreground transition-colors hover:text-primary',
					hasAnySelection ? 'cursor-pointer' : ''
				)}
			>
				{shader.name}
			</a>
			<div class="flex items-center gap-2 text-sm text-muted-foreground">
				<span>by</span>
				<a
					href={shader.authorUrl}
					target="_blank"
					rel="noopener noreferrer"
					data-external-link
					data-clickable
					class="inline-flex cursor-pointer items-center gap-1 font-medium text-card-foreground underline decoration-transparent underline-offset-2 transition-colors hover:text-primary hover:decoration-primary"
				>
					{shader.author}
					<svg
						class="h-3 w-3 opacity-50"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M13.5 6H5.25A2.25 2.25 0 003 8.25v10.5A2.25 2.25 0 005.25 21h10.5A2.25 2.25 0 0018 18.75V10.5m-10.5 6L21 3m0 0h-5.25M21 3v5.25"
						/>
					</svg>
				</a>
				<span class="text-muted-foreground/50">·</span>
				<span>{formatDate(shader.lastUpdated)}</span>
			</div>
		</div>

		<!-- Description -->
		<p class="line-clamp-2 flex-1 text-sm text-muted-foreground">
			{shader.description}
		</p>

		<!-- Style Tag -->
		<div class="flex flex-wrap gap-1.5">
			<span
				class={cn(
					'inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium capitalize',
					getStyleColor(shader.style)
				)}
			>
				{shader.style}
			</span>
		</div>

		<!-- Features -->
		<div class="flex flex-wrap gap-1">
			{#each shader.features.slice(0, 4) as feature}
				<span
					class="inline-flex items-center rounded-md bg-muted/50 px-1.5 py-0.5 text-[11px] text-muted-foreground ring-1 ring-border/50"
				>
					{feature.name}
				</span>
			{/each}
			{#if shader.features.length > 4}
				<span
					class="inline-flex items-center rounded-md bg-muted/50 px-1.5 py-0.5 text-[11px] text-muted-foreground ring-1 ring-border/50"
				>
					+{shader.features.length - 4} more
				</span>
			{/if}
		</div>

		<!-- Footer: Version & Links -->
		<div class="mt-auto flex items-center justify-between border-t border-border pt-3">
			<div class="flex items-center gap-3">
				<!-- Shader version with label -->
				<span class="inline-flex items-center gap-1 text-xs">
					<span class="text-muted-foreground/70">v</span>
					<span class="font-medium text-card-foreground">{shader.version}</span>
				</span>

				<!-- Visual separator -->
				<span class="h-4 w-px bg-border"></span>

				<!-- MC versions with icon -->
				<span class="inline-flex items-center gap-1.5 text-xs text-muted-foreground">
					<svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="currentColor">
						<path
							d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"
							stroke="currentColor"
							stroke-width="2"
							fill="none"
							stroke-linecap="round"
							stroke-linejoin="round"
						/>
					</svg>
					{shader.mcVersions.join(', ')}
				</span>
			</div>

			<div class="flex items-center gap-1">
				{#if shader.modrinthUrl}
					<a
						href={shader.modrinthUrl}
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
				{#if shader.curseforgeUrl}
					<a
						href={shader.curseforgeUrl}
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
			</div>
		</div>
	</div>
</div>
