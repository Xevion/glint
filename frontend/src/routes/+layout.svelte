<script lang="ts">
import { OverlayScrollbars } from 'overlayscrollbars';
import type { OverlayScrollbars as OverlayScrollbarsInstance } from 'overlayscrollbars';
import type { Snippet } from 'svelte';
import { onMount } from 'svelte';
import 'overlayscrollbars/overlayscrollbars.css';
import BackgroundImage from '$lib/components/BackgroundImage.svelte';
import Navigation from '$lib/components/Navigation.svelte';
import Sidebar from '$lib/components/Sidebar.svelte';
import { initNavigation } from '$lib/stores/navigation.svelte';
import { themeStore } from '$lib/stores/theme.svelte';
import './layout.css';

interface Props {
	children: Snippet;
}

let { children }: Props = $props();

let osInstance: OverlayScrollbarsInstance | null = null;

initNavigation();

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

<BackgroundImage
	lightWallpapers={[13, 53, 31, 71, 18, 65, 32, 97]}
	darkWallpapers={[102, 69, 23, 94, 13]}
	blur={4}
	overlayOpacity={0.7}
	lightBrightness={1.7}
>
	<div class="flex min-h-screen flex-col overflow-x-hidden px-3 md:px-5">
		<div class="w-full max-w-6xl mx-auto flex flex-col flex-1">
			<!-- Navbar - excluded from view transitions to avoid ghost highlights -->
			<div class="pt-5 pb-5">
				<Navigation />
			</div>

			<!-- Content with contextual sidebar -->
			<main class="flex-1 flex gap-8 pb-5">
				<Sidebar />
				<div class="flex-1 min-w-0" style="view-transition-name: app-content">
					{@render children()}
				</div>
			</main>
		</div>
	</div>
</BackgroundImage>
