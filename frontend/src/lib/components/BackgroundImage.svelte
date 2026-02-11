<script lang="ts">
import type { Background } from '$lib/bindings';
import { cfImageUrl } from '$lib/utils/image';
import { decodeThumbhash } from '$lib/utils/thumbhash';
import { onMount } from 'svelte';
import type { Snippet } from 'svelte';

interface Props {
	backgrounds: Background[];
	blur?: number;
	overlay?: boolean;
	overlayOpacity?: number;
	children?: Snippet;
	/** Scale factor for wallpapers (1.75 = 175% zoom) */
	scale?: number;
	/** Height of the gradient blend zone between wallpapers in pixels */
	blendHeight?: number;
	/** Brightness adjustment for light mode wallpapers (1.0 = normal, 1.1 = 10% brighter) */
	lightBrightness?: number;
}

const {
	backgrounds,
	blur = 0,
	overlay = true,
	overlayOpacity = 0.5,
	children,
	scale = 1.75,
	blendHeight = 150,
	lightBrightness = 1.0
}: Props = $props();

// Default wallpaper dimensions (used when background has no width/height)
const DEFAULT_WIDTH = 3840;
const DEFAULT_HEIGHT = 2160;

let containerHeight = $state(2160);
let viewportWidth = $state(1920);
let containerEl: HTMLDivElement | undefined = $state();
let mounted = $state(false);
let initialAnimationDone = $state(false);

// Split backgrounds by theme mode
const lightBackgrounds = $derived(
	backgrounds.filter((b) => b.theme_mode === 'light' || b.theme_mode === 'both')
);
const darkBackgrounds = $derived(
	backgrounds.filter((b) => b.theme_mode === 'dark' || b.theme_mode === 'both')
);

// Calculate the displayed height of each wallpaper section based on scale
// Use first background's dimensions or defaults
const sectionHeight = $derived.by(() => {
	const ref = backgrounds[0];
	const w = ref?.width ?? DEFAULT_WIDTH;
	const h = ref?.height ?? DEFAULT_HEIGHT;
	const displayedHeight = (viewportWidth * h) / w / scale;
	return displayedHeight;
});

// Calculate how many wallpaper sections we need
const sectionsNeeded = $derived.by(() => {
	if (sectionHeight <= 0) return 3;
	return Math.max(3, Math.ceil(containerHeight / sectionHeight) + 1);
});

function getImageUrl(bg: Background): string {
	// Use Cloudflare Image Transforms to serve optimized version (480px wide, webp)
	return cfImageUrl(bg.image_url, { width: 480, quality: 50, format: 'webp' }) ?? bg.image_url;
}

function getThumbhashBackground(bg: Background): string | null {
	return decodeThumbhash(bg.thumbhash);
}

// Shuffle array using Fisher-Yates algorithm
function shuffleArray<T>(array: T[]): T[] {
	const shuffled = [...array];
	for (let i = shuffled.length - 1; i > 0; i--) {
		const j = Math.floor(Math.random() * (i + 1));
		[shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
	}
	return shuffled;
}

// Cache for shuffled arrays (initialized in onMount)
let lightShuffled = $state<Background[]>([]);
let darkShuffled = $state<Background[]>([]);

// Get backgrounds for the required sections (with looping)
function getBackgroundsForSections(bgs: Background[], count: number): Background[] {
	if (bgs.length === 0) return [];
	const result: Background[] = [];
	for (let i = 0; i < count; i++) {
		result.push(bgs[i % bgs.length]);
	}
	return result;
}

const lightSections = $derived(getBackgroundsForSections(lightShuffled, sectionsNeeded));
const darkSections = $derived(getBackgroundsForSections(darkShuffled, sectionsNeeded));

let imagesLoaded = $state(false);

const backgroundSize = $derived(`${scale * 100}%`);

onMount(() => {
	lightShuffled = shuffleArray(lightBackgrounds);
	darkShuffled = shuffleArray(darkBackgrounds);

	let debounceTimer: ReturnType<typeof setTimeout> | undefined;
	let previousHeight = 0;

	const updateHeight = () => {
		const newHeight = Math.max(
			document.documentElement.scrollHeight,
			document.documentElement.clientHeight,
			window.innerHeight
		);

		if (Math.abs(newHeight - previousHeight) > 50) {
			containerHeight = newHeight;
			previousHeight = newHeight;
		}
	};

	const updateDimensions = () => {
		viewportWidth = window.innerWidth;
		updateHeight();
	};

	updateDimensions();
	mounted = true;

	const debouncedHeightUpdate = () => {
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(updateHeight, 150);
	};

	// Preload all background images before showing them
	const preloadImages = async () => {
		const allBgs = [...new Set([...lightBackgrounds, ...darkBackgrounds])];
		if (allBgs.length === 0) {
			imagesLoaded = true;
			return;
		}

		const promises = allBgs.map((bg) => {
			return new Promise<void>((resolve, reject) => {
				const img = new Image();
				img.onload = () => resolve();
				img.onerror = () => reject(new Error(`Failed to load background ${bg.id}`));
				img.src = getImageUrl(bg);
			});
		});

		try {
			await Promise.all(promises);
			imagesLoaded = true;
		} catch (error) {
			console.error('Failed to preload background images:', error);
			imagesLoaded = true;
		}
	};

	void preloadImages().then(() => {
		setTimeout(() => {
			initialAnimationDone = true;
		}, 800);
	});

	const observer = new MutationObserver(debouncedHeightUpdate);

	observer.observe(document.body, {
		childList: true,
		subtree: true,
		attributes: true,
		attributeFilter: ['style', 'class']
	});

	window.addEventListener('resize', updateDimensions);

	return () => {
		if (debounceTimer) clearTimeout(debounceTimer);
		observer.disconnect();
		window.removeEventListener('resize', updateDimensions);
	};
});

const lightFilterStyle = $derived.by(() => {
	const filters: string[] = [];
	if (blur > 0) filters.push(`blur(${blur}px)`);
	if (lightBrightness !== 1.0) filters.push(`brightness(${lightBrightness})`);
	return filters.length > 0 ? filters.join(' ') : undefined;
});

const darkFilterStyle = $derived(blur > 0 ? `blur(${blur}px)` : undefined);
</script>

<div class="background-container" bind:this={containerEl}>
	<!-- Light theme thumbhash placeholders (shown immediately while images load) -->
	{#if mounted && !imagesLoaded}
		{#each lightSections as bg, i (i)}
			{@const top = i * sectionHeight}
			{@const isFirst = i === 0}
			{@const isLast = i === lightSections.length - 1}
			{@const thumbhashUrl = getThumbhashBackground(bg)}
			{#if thumbhashUrl}
				<div
					class="wallpaper-section wallpaper-light"
					style:top="{top}px"
					style:height="{sectionHeight + (isLast ? 0 : blendHeight)}px"
					style:background-image="url({thumbhashUrl})"
					style:background-size="cover"
					style:filter={lightFilterStyle ?? undefined}
					style:--blend-height="{blendHeight}px"
					class:has-fade-top={!isFirst}
					class:has-fade-bottom={!isLast}
				></div>
			{/if}
		{/each}
		{#each darkSections as bg, i (i)}
			{@const top = i * sectionHeight}
			{@const isFirst = i === 0}
			{@const isLast = i === darkSections.length - 1}
			{@const thumbhashUrl = getThumbhashBackground(bg)}
			{#if thumbhashUrl}
				<div
					class="wallpaper-section wallpaper-dark"
					style:top="{top}px"
					style:height="{sectionHeight + (isLast ? 0 : blendHeight)}px"
					style:background-image="url({thumbhashUrl})"
					style:background-size="cover"
					style:filter={darkFilterStyle ?? undefined}
					style:--blend-height="{blendHeight}px"
					class:has-fade-top={!isFirst}
					class:has-fade-bottom={!isLast}
				></div>
			{/if}
		{/each}
	{/if}

	<!-- Light theme wallpapers -->
	{#if mounted && imagesLoaded}
		{#each lightSections as bg, i (i)}
			{@const top = i * sectionHeight}
			{@const isFirst = i === 0}
			{@const isLast = i === lightSections.length - 1}
			<div
				class="wallpaper-section wallpaper-light"
				class:animate-in={!initialAnimationDone}
				style:top="{top}px"
				style:height="{sectionHeight + (isLast ? 0 : blendHeight)}px"
				style:background-image="url({getImageUrl(bg)})"
				style:background-size={backgroundSize}
				style:filter={lightFilterStyle ?? undefined}
				style:--blend-height="{blendHeight}px"
				class:has-fade-top={!isFirst}
				class:has-fade-bottom={!isLast}
			></div>
		{/each}
	{/if}

	<!-- Dark theme wallpapers -->
	{#if mounted && imagesLoaded}
		{#each darkSections as bg, i (i)}
			{@const top = i * sectionHeight}
			{@const isFirst = i === 0}
			{@const isLast = i === darkSections.length - 1}
			<div
				class="wallpaper-section wallpaper-dark"
				class:animate-in={!initialAnimationDone}
				style:top="{top}px"
				style:height="{sectionHeight + (isLast ? 0 : blendHeight)}px"
				style:background-image="url({getImageUrl(bg)})"
				style:background-size={backgroundSize}
				style:filter={darkFilterStyle ?? undefined}
				style:--blend-height="{blendHeight}px"
				class:has-fade-top={!isFirst}
				class:has-fade-bottom={!isLast}
			></div>
		{/each}
	{/if}

	{#if overlay}
		<div class="background-overlay" style:--overlay-opacity={overlayOpacity}></div>
	{/if}

	<div class="noise-layer"></div>

	<div class="background-content">
		{#if children}
			{@render children()}
		{/if}
	</div>
</div>

<style>
	.background-container {
		position: relative;
		width: 100%;
		min-height: 100vh;
		overflow: hidden;
	}

	.wallpaper-section {
		position: absolute;
		left: -20px;
		right: -20px;
		background-position: center;
		background-repeat: no-repeat;
		z-index: 0;
		opacity: 1;
		transition:
			top 300ms ease-out,
			height 300ms ease-out;
		will-change: top, height;
	}

	/* Only animate fade-in on initial mount, not on section recreation */
	.wallpaper-section.animate-in {
		opacity: 0;
		animation: fade-in 800ms ease-out forwards;
	}

	@keyframes fade-in {
		from {
			opacity: 0;
		}

		to {
			opacity: 1;
		}
	}

	/* Gradient masks for smooth blending */
	.wallpaper-section.has-fade-top {
		mask-image: linear-gradient(to bottom, transparent 0, black var(--blend-height));
	}

	.wallpaper-section.has-fade-bottom {
		mask-image: linear-gradient(
			to bottom,
			black calc(100% - var(--blend-height)),
			transparent 100%
		);
	}

	.wallpaper-section.has-fade-top.has-fade-bottom {
		mask-image: linear-gradient(
			to bottom,
			transparent 0,
			black var(--blend-height),
			black calc(100% - var(--blend-height)),
			transparent 100%
		);
	}

	.wallpaper-light {
		display: block;
	}

	.wallpaper-dark {
		display: none;
	}

	:global(.dark) .wallpaper-light {
		display: none;
	}

	:global(.dark) .wallpaper-dark {
		display: block;
	}

	.background-overlay {
		position: absolute;
		inset: 0;
		background: var(--background);
		opacity: var(--overlay-opacity);
		z-index: 1;
	}

	.background-content {
		position: relative;
		z-index: 3;
	}

	.noise-layer {
		position: absolute;
		inset: 0;
		background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 512 512' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='3' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)'/%3E%3C/svg%3E");
		opacity: 0.18;
		pointer-events: none;
		z-index: 2;
	}

	:global(.dark) .noise-layer {
		opacity: 0.08;
	}
</style>
