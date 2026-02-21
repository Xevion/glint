<script lang="ts">
import { OverlayScrollbars } from 'overlayscrollbars';
import type { OverlayScrollbars as OverlayScrollbarsInstance } from 'overlayscrollbars';
import type { Snippet } from 'svelte';
import { onMount } from 'svelte';
import 'overlayscrollbars/overlayscrollbars.css';
import BackgroundImage from '$lib/components/BackgroundImage.svelte';
import ConnectivityBanner from '$lib/components/ConnectivityBanner.svelte';
import Footer from '$lib/components/Footer.svelte';
import Navigation from '$lib/components/Navigation.svelte';
import SectionBoundary from '$lib/components/SectionBoundary.svelte';
import Sidebar from '$lib/components/Sidebar.svelte';
import BrandIconSprite from '$lib/components/icons/BrandIconSprite.svelte';
import { initNavigation } from '$lib/stores/navigation.svelte';
import { themeStore } from '$lib/stores/theme.svelte';
import { telemetry } from '$lib/telemetry';
import { Toaster } from 'svelte-sonner';
import type { LayoutData } from './$types';
import './layout.css';

interface Props {
	data: LayoutData;
	children: Snippet;
}

let { data, children }: Props = $props();

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
	telemetry.init();

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

<BrandIconSprite />
<ConnectivityBanner />
<Toaster richColors />

<BackgroundImage
	lightBackgrounds={data.lightBackgrounds}
	darkBackgrounds={data.darkBackgrounds}
	blur={4}
	overlayOpacity={0.7}
	lightOverlayOpacity={0.82}
	lightBrightness={1.7}
>
	<div class="flex min-h-screen flex-col overflow-x-hidden px-3 md:px-5">
		<div class="w-full max-w-7xl mx-auto flex flex-col flex-1">
			<!-- Navbar - excluded from view transitions to avoid ghost highlights -->
			<div class="pt-5 pb-5">
				<svelte:boundary onerror={(e) => console.error('[Navigation]', e)}>
					<Navigation />
					{#snippet failed()}
						<nav class="flex h-10 items-center">
							<a href="/" class="text-lg font-semibold">Glint</a>
						</nav>
					{/snippet}
				</svelte:boundary>
			</div>

			<!-- Content with contextual sidebar -->
			<main class="flex-1 flex gap-8 pb-5">
			<svelte:boundary onerror={(e) => console.error('[Sidebar]', e)}>
				<Sidebar />
				{#snippet failed()}
					<aside class="hidden w-40 shrink-0 pt-1 md:flex md:flex-col">
						<span class="px-2 text-xs text-muted-foreground">Nav unavailable</span>
					</aside>
				{/snippet}
			</svelte:boundary>
			<div class="flex-1 min-w-0" style="view-transition-name: app-content">
				<SectionBoundary title="Page">
					{@render children()}
				</SectionBoundary>
			</div>
			</main>

			<Footer />
		</div>
	</div>
</BackgroundImage>
