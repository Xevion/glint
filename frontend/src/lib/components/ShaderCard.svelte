<script lang="ts">
	import type { Shader } from '$lib/data/types';
	import { formatDate, getModrinthUrl, getCurseforgeUrl } from '$lib/utils/display';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { comparisonStore } from '$lib/stores/comparison.svelte';
	import { cn } from '$lib/utils';
	import BrandIcon from './icons/BrandIcon.svelte';

	interface Props {
		shader: Shader;
		class?: string;
	}

	let { shader, class: className }: Props = $props();

	// Deterministic wallpaper selection based on shader ID (avoids hydration mismatch)
	function hashStringToNumber(str: string): number {
		let hash = 0;
		for (let i = 0; i < str.length; i++) {
			hash = (hash << 5) - hash + str.charCodeAt(i);
			hash = hash & hash; // Convert to 32-bit integer
		}
		return Math.abs(hash);
	}

	const wallpaperIndex = $derived(hashStringToNumber(shader.id) % 50);

	let isHovered = $state(false);
	const isSelected = $derived(comparisonStore.isShaderSelected(shader.id));
	const hasAnySelection = $derived(comparisonStore.hasShaderSelection);

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
			comparisonStore.toggleShader(shader.id);
			return;
		}

		void goto(resolve(`/shaders/${shader.slug}`), { invalidateAll: true });
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			if (hasAnySelection) {
				comparisonStore.toggleShader(shader.id);
			} else {
				void goto(resolve(`/shaders/${shader.slug}`), { invalidateAll: true });
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
			alt="{shader.name} preview"
			class="h-full w-full object-cover transition-transform duration-500 ease-out group-hover:scale-105"
			style="will-change: transform; backface-visibility: hidden;"
			loading="lazy"
		/>

		<!-- Gradient overlay -->
		<div
			class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent"
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
		<div class="space-y-1">
			<a
				href={resolve(`/shaders/${shader.slug}`)}
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
				<span class="font-medium text-card-foreground">Unknown Author</span>
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
		<div class="flex flex-wrap gap-1.5">
			<span
				class="inline-flex items-center rounded-md bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary capitalize ring-1 ring-primary/20"
			>
				Realistic
			</span>
			<span
				class="inline-flex items-center rounded-md bg-muted/50 px-1.5 py-0.5 text-[11px] text-muted-foreground ring-1 ring-border/50"
			>
				Ray Tracing
			</span>
			<span
				class="inline-flex items-center rounded-md bg-muted/50 px-1.5 py-0.5 text-[11px] text-muted-foreground ring-1 ring-border/50"
			>
				Shadows
			</span>
		</div>

		<!-- Footer: Version & Links -->
		<div class="mt-auto flex items-center justify-between border-t border-border pt-3">
			<div class="flex items-center gap-3">
				<!-- Version -->
				<span class="inline-flex items-center gap-1 text-xs">
					<span class="text-muted-foreground/70">v</span>
					<span class="font-medium text-card-foreground">1.0.0</span>
				</span>

				<!-- Separator -->
				<span class="h-4 w-px bg-border"></span>

				<!-- MC Version -->
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
					1.21.4
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
						<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M13.5 6H5.25A2.25 2.25 0 003 8.25v10.5A2.25 2.25 0 005.25 21h10.5A2.25 2.25 0 0018 18.75V10.5m-10.5 6L21 3m0 0h-5.25M21 3v5.25"
							/>
						</svg>
					</a>
				{/if}
			</div>
			<!-- eslint-enable svelte/no-navigation-without-resolve -->
		</div>
	</div>
</div>
