<script lang="ts">
	import { onMount } from 'svelte';
	import Navigation from '$lib/components/Navigation.svelte';
	import BackgroundImage from '$lib/components/BackgroundImage.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();

	// Background images - light and dark variants
	const lightBg = '/hero-bg-light.jpg';
	const darkBg = '/hero-bg-dark.jpg';

	onMount(() => {
		themeStore.init();
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<BackgroundImage src={themeStore.isDark ? darkBg : lightBg} blur={4} overlayOpacity={0.7}>
	<div class="flex min-h-screen flex-col">
		<Navigation />
		<main class="flex-1">
			{@render children()}
		</main>
	</div>
</BackgroundImage>
