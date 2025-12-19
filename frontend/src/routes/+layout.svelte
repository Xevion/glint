<script lang="ts">
	import { onMount } from 'svelte';
	import { OverlayScrollbars } from 'overlayscrollbars';
	import type { OverlayScrollbars as OverlayScrollbarsInstance } from 'overlayscrollbars';
	import 'overlayscrollbars/overlayscrollbars.css';
	import Navigation from '$lib/components/Navigation.svelte';
	import BackgroundImage from '$lib/components/BackgroundImage.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();

	let osInstance: OverlayScrollbarsInstance | null = null;

	// Reactively update scrollbar theme when theme changes
	$effect(() => {
		const scrollbarTheme = themeStore.isDark ? 'os-theme-light' : 'os-theme-dark';
		osInstance?.options({ scrollbars: { theme: scrollbarTheme } });
	});

	onMount(() => {
		// Set up system preference listener (theme is already applied by blocking script)
		themeStore.init();

		// Initialize OverlayScrollbars on the body for full-page scrolling
		osInstance = OverlayScrollbars(document.body, {
			scrollbars: {
				theme: themeStore.isDark ? 'os-theme-light' : 'os-theme-dark',
				autoHide: 'leave',
				autoHideDelay: 400
			}
		});

		return () => {
			osInstance?.destroy();
			osInstance = null;
		};
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<BackgroundImage lightSrc="/hero-bg-light.jpg" darkSrc="/hero-bg-dark.jpg" blur={4} overlayOpacity={0.7}>
	<div class="flex min-h-screen flex-col">
		<Navigation />
		<main class="flex-1">
			{@render children()}
		</main>
	</div>
</BackgroundImage>
