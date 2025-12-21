<script lang="ts">
	import { onMount } from 'svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		lightWallpapers: number[];
		darkWallpapers: number[];
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
		lightWallpapers,
		darkWallpapers,
		blur = 0,
		overlay = true,
		overlayOpacity = 0.5,
		children,
		scale = 1.75,
		blendHeight = 150,
		lightBrightness = 1.0
	}: Props = $props();

	// 4K wallpaper dimensions
	const WALLPAPER_WIDTH = 3840;
	const WALLPAPER_HEIGHT = 2160;

	let containerHeight = $state(2160);
	let viewportWidth = $state(1920);
	let containerEl: HTMLDivElement | undefined = $state();
	let mounted = $state(false);

	// Calculate the displayed height of each wallpaper section based on scale
	const sectionHeight = $derived(() => {
		// At scale 1.75, the wallpaper is zoomed in 175%
		// The displayed width is viewport width, so displayed height maintains aspect ratio
		const displayedHeight = (viewportWidth * WALLPAPER_HEIGHT) / WALLPAPER_WIDTH / scale;
		return displayedHeight;
	});

	// Calculate how many wallpaper sections we need
	const sectionsNeeded = $derived(() => {
		const height = sectionHeight();
		if (height <= 0) return 3; // Start with minimum 3 sections
		return Math.max(3, Math.ceil(containerHeight / height) + 1);
	});

	// Generate wallpaper URLs for a given theme
	function getWallpaperUrl(index: number): string {
		return `/wallpapers/${index}.jpg`;
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

	// Cache for shuffled wallpaper arrays (initialized in onMount to prevent double-render)
	let lightShuffled = $state<number[]>([]);
	let darkShuffled = $state<number[]>([]);

	// Get the wallpaper indices for the required sections (with looping)
	function getWallpaperIndices(wallpapers: number[], count: number): number[] {
		const indices: number[] = [];
		for (let i = 0; i < count; i++) {
			indices.push(wallpapers[i % wallpapers.length]);
		}
		return indices;
	}

	const lightIndices = $derived(getWallpaperIndices(lightShuffled, sectionsNeeded()));
	const darkIndices = $derived(getWallpaperIndices(darkShuffled, sectionsNeeded()));

	// Track image loading state
	let imagesLoaded = $state(false);

	// Calculate background size based on scale
	const backgroundSize = $derived(() => {
		// Scale is how much we zoom in, so 1.75 means the image is 175% of viewport width
		return `${scale * 100}%`;
	});

	onMount(() => {
		// Initialize shuffled arrays immediately on mount (before images load)
		lightShuffled = shuffleArray(lightWallpapers);
		darkShuffled = shuffleArray(darkWallpapers);

		let debounceTimer: ReturnType<typeof setTimeout> | undefined;
		let previousHeight = 0;

		const updateHeight = () => {
			const newHeight = Math.max(
				document.documentElement.scrollHeight,
				document.documentElement.clientHeight,
				window.innerHeight
			);

			// Only update if height changed by more than 50px to prevent micro-adjustments
			if (Math.abs(newHeight - previousHeight) > 50) {
				containerHeight = newHeight;
				previousHeight = newHeight;
			}
		};

		const updateDimensions = () => {
			viewportWidth = window.innerWidth;
			updateHeight();
		};

		// Initial dimension capture - do this immediately to prevent layout shift
		updateDimensions();
		mounted = true;

		const debouncedHeightUpdate = () => {
			if (debounceTimer) clearTimeout(debounceTimer);
			debounceTimer = setTimeout(updateHeight, 150);
		};

		// Preload all wallpaper images before showing them
		const preloadImages = async () => {
			const allIndices = [...new Set([...lightWallpapers, ...darkWallpapers])];
			const promises = allIndices.map((index) => {
				return new Promise<void>((resolve, reject) => {
					const img = new Image();
					img.onload = () => {
						resolve();
					};
					img.onerror = () => {
						reject(new Error(`Failed to load wallpaper ${index}`));
					};
					img.src = getWallpaperUrl(index);
				});
			});

			try {
				await Promise.all(promises);
				imagesLoaded = true;
			} catch (error) {
				console.error('Failed to preload wallpaper images:', error);
				imagesLoaded = true; // Show anyway even if some failed
			}
		};

		void preloadImages();

		// Use MutationObserver for height changes only (not width)
		const observer = new MutationObserver(debouncedHeightUpdate);

		observer.observe(document.body, {
			childList: true,
			subtree: true,
			attributes: true,
			attributeFilter: ['style', 'class']
		});

		// Only update width on actual window resize
		window.addEventListener('resize', updateDimensions);

		return () => {
			if (debounceTimer) clearTimeout(debounceTimer);
			observer.disconnect();
			window.removeEventListener('resize', updateDimensions);
		};
	});

	const lightFilterStyle = $derived(() => {
		const filters: string[] = [];
		if (blur > 0) filters.push(`blur(${blur}px)`);
		if (lightBrightness !== 1.0) filters.push(`brightness(${lightBrightness})`);
		return filters.length > 0 ? filters.join(' ') : undefined;
	});

	const darkFilterStyle = $derived(blur > 0 ? `blur(${blur}px)` : undefined);
</script>

<div class="background-container" bind:this={containerEl}>
	<!-- Light theme wallpapers -->
	{#if mounted && imagesLoaded}
		{#each lightIndices as wallpaperIndex, i (i)}
			{@const top = i * sectionHeight()}
			{@const isFirst = i === 0}
			{@const isLast = i === lightIndices.length - 1}
			<div
				class="wallpaper-section wallpaper-light"
				style:top="{top}px"
				style:height="{sectionHeight() + (isLast ? 0 : blendHeight)}px"
				style:background-image="url({getWallpaperUrl(wallpaperIndex)})"
				style:background-size={backgroundSize()}
				style:filter={lightFilterStyle() ?? undefined}
				style:--blend-height="{blendHeight}px"
				class:has-fade-top={!isFirst}
				class:has-fade-bottom={!isLast}
			></div>
		{/each}
	{/if}

	<!-- Dark theme wallpapers -->
	{#if mounted && imagesLoaded}
		{#each darkIndices as wallpaperIndex, i (i)}
			{@const top = i * sectionHeight()}
			{@const isFirst = i === 0}
			{@const isLast = i === darkIndices.length - 1}
			<div
				class="wallpaper-section wallpaper-dark"
				style:top="{top}px"
				style:height="{sectionHeight() + (isLast ? 0 : blendHeight)}px"
				style:background-image="url({getWallpaperUrl(wallpaperIndex)})"
				style:background-size={backgroundSize()}
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
		opacity: 0;
		animation: fade-in 800ms ease-out forwards;
		transition:
			top 300ms ease-out,
			height 300ms ease-out;
		will-change: top, height;
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
		top: 4rem; /* Start below header to allow header glass effect */
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
		top: 4rem; /* Start below header */
		background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 512 512' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='3' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)'/%3E%3C/svg%3E");
		opacity: 0.18;
		pointer-events: none;
		z-index: 2;
	}

	:global(.dark) .noise-layer {
		opacity: 0.08;
	}
</style>
