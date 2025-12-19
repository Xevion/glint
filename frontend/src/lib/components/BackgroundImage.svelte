<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		src: string;
		blur?: number;
		overlay?: boolean;
		overlayOpacity?: number;
		fixed?: boolean;
		children?: Snippet;
	}

	let {
		src,
		blur = 0,
		overlay = true,
		overlayOpacity = 0.5,
		fixed = false,
		children
	}: Props = $props();
</script>

<div class="background-container" class:fixed-bg={fixed}>
	<div
		class="background-image"
		style:background-image="url({src})"
		style:filter={blur > 0 ? `blur(${blur}px)` : undefined}
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
		z-index: 2;
	}
</style>
