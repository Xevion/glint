<script lang="ts">
	import type { Snippet } from 'svelte';
	import { onMount } from 'svelte';
	import { OverlayScrollbars } from 'overlayscrollbars';
	import type { OverlayScrollbars as OverlayScrollbarsInstance } from 'overlayscrollbars';
	import 'overlayscrollbars/overlayscrollbars.css';
	import { themeStore } from '$lib/stores/theme.svelte';
	import BackgroundImage from '$lib/components/BackgroundImage.svelte';
	import Navigation from '$lib/components/Navigation.svelte';
	import './layout.css';

	interface Props {
		children: Snippet;
	}

	let { children }: Props = $props();

	const favicon = $derived(themeStore.isDark ? '/favicon-dark.ico' : '/favicon-light.ico');

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

<BackgroundImage
	lightWallpapers={[13, 53, 31, 71, 18, 65, 32, 97]}
	darkWallpapers={[102, 69, 23, 94, 13]}
	blur={4}
	overlayOpacity={0.7}
	lightBrightness={1.7}
>
	<div class="flex min-h-screen flex-col">
		<Navigation />
		<main class="flex-1">
			{@render children()}
		</main>
	</div>
</BackgroundImage>
