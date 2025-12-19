<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		lightSrc: string;
		darkSrc: string;
		blur?: number;
		overlay?: boolean;
		overlayOpacity?: number;
		fixed?: boolean;
		children?: Snippet;
	}

	let {
		lightSrc,
		darkSrc,
		blur = 0,
		overlay = true,
		overlayOpacity = 0.5,
		fixed = false,
		children
	}: Props = $props();

	const filterStyle = blur > 0 ? `blur(${blur}px)` : undefined;
</script>

<div class="background-container" class:fixed-bg={fixed}>
	<div
		class="background-image background-image-light"
		style:background-image="url({lightSrc})"
		style:filter={filterStyle}
	></div>
	<div
		class="background-image background-image-dark"
		style:background-image="url({darkSrc})"
		style:filter={filterStyle}
	></div>

	{#if overlay}
		<div class="background-overlay" style:opacity={overlayOpacity}></div>
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

	.background-image {
		position: absolute;
		inset: -20px; /* Extend beyond container to prevent blur edge artifacts */
		background-size: cover;
		background-position: center;
		background-repeat: no-repeat;
		z-index: 0;
	}

	.background-image-light {
		opacity: 1;
	}

	.background-image-dark {
		opacity: 0;
	}

	:global(.dark) .background-image-light {
		opacity: 0;
	}

	:global(.dark) .background-image-dark {
		opacity: 1;
	}

	.fixed-bg .background-image {
		position: fixed;
		inset: 0;
		background-attachment: fixed;
	}

	.background-overlay {
		position: absolute;
		inset: 0;
		background: var(--background);
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
