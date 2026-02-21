<script lang="ts">
import { resolve } from '$app/paths';
import CaptureBadges from '$lib/components/CaptureBadges.svelte';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { ImageOverlay } from '$lib/components/ui/image-overlay';
import Meta from '$lib/components/Meta.svelte';
import { lightbox } from '$lib/stores/lightbox.svelte';
import { ChevronRight, ImageOff } from '@lucide/svelte';
import type { SceneCapture } from './+page';

import { formatMoonPhase, formatPercent, formatTimeTicks } from '$lib/utils/format';
import { fly } from 'svelte/transition';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}

let { data }: Props = $props();
const scene = $derived(data.scene);

// Client-side "show more" from the full captures array (all loaded via GraphQL)
const capturesPageSize = 24;

let visibleCount = $state(capturesPageSize);
const allCaptures = $derived(
	data.scene.captures.filter((c): c is SceneCapture & { imagePath: string } => c.imagePath != null)
);
const visibleCaptures = $derived(allCaptures.slice(0, visibleCount));
const hasMoreCaptures = $derived(visibleCount < allCaptures.length);

// Reset visible count when scene data changes
$effect(() => {
	// eslint-disable-next-line @typescript-eslint/no-unused-expressions -- Svelte $effect dependency tracking
	data.capturesData;
	visibleCount = capturesPageSize;
});

function loadMoreCaptures() {
	visibleCount = Math.min(visibleCount + capturesPageSize, allCaptures.length);
}

function observeSentinel(node: HTMLElement) {
	const observer = new IntersectionObserver(
		(entries) => {
			if (entries[0]?.isIntersecting) loadMoreCaptures();
		},
		{ rootMargin: '200px' }
	);
	observer.observe(node);
	return { destroy: () => observer.disconnect() };
}

// Use original scene captures for the shader thumbnail strip (not paginated)
const sceneCaptures = $derived(scene.captures);

// Selected capture for the main preview
let selectedCapture = $state<SceneCapture | null>(null);

// Initialize selected capture when scene captures change
$effect(() => {
	if (sceneCaptures.length > 0 && !selectedCapture) {
		selectedCapture = sceneCaptures[0];
	}
});

// Group captures by shader (from full scene data, not paginated)
const capturesByShader = $derived.by(() => {
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- ephemeral map inside $derived.by
	const map = new Map<string, SceneCapture>();
	for (const capture of sceneCaptures) {
		if (!map.has(capture.shaderSlug)) {
			map.set(capture.shaderSlug, capture);
		}
	}
	return Array.from(map.values());
});

// Weather display helper
const weatherLabel = $derived.by(() => {
	const v = scene.version;
	if (!v) return null;
	if (v.weather === 'clear') return 'Clear';
	if (v.weather === 'rain') return `Rain (${formatPercent(v.weatherIntensity, 0)})`;
	if (v.weather === 'thunder') return `Thunder (${formatPercent(v.weatherIntensity, 0)})`;
	return v.weather;
});

// Time display helper
const timeLabel = $derived.by(() => {
	const v = scene.version;
	if (!v) return null;
	return formatTimeTicks(v.timeOfDayTicks);
});

// OG metadata
const ogImage = $derived(sceneCaptures[0]?.imagePath ?? null);
const ogDescription = $derived.by(() => {
	const parts = [`${scene.name} scene for Minecraft shader comparison`];
	if (sceneCaptures.length > 0)
		parts.push(`${sceneCaptures.length} shader screenshot${sceneCaptures.length === 1 ? '' : 's'}`);
	return parts.join(' \u00b7 ');
});
</script>

<Meta
	title={scene.name}
	description={ogDescription}
	image={ogImage}
	ogImagePath={`/og/scene/${scene.slug}/og.png`}
/>

{#if scene}
	{#key scene.id}
		<div class="py-8">
			<!-- Breadcrumb -->
			<nav
				in:fly|local={{ y: -10, duration: 400 }}
				class="mb-6 flex items-center gap-2 text-sm text-foreground"
			>
				<a href={resolve('/scenes', {})} class="transition-colors hover:text-foreground">Scenes</a>
				<ChevronRight class="h-4 w-4" strokeWidth={2} />
				<span class="font-medium text-foreground">{scene.name}</span>
			</nav>

			<!-- Hero: Preview + Info -->
			<div
				in:fly|local={{ y: 10, duration: 400, delay: 100 }}
				class="mb-8 grid gap-6 lg:grid-cols-3"
			>
				<!-- Main Preview Area -->
				<div class="space-y-4 lg:col-span-2">
					<!-- Main Image -->
					<div class="shadow-theme-lg relative aspect-video w-full overflow-hidden rounded-xl">
						{#if selectedCapture}
						<CaptureImage
							src={selectedCapture.imagePath}
							thumbhash={selectedCapture.thumbhash}
							preset="hero"
							priority
							alt="{scene.name} with {selectedCapture.shaderName}"
								class="h-full w-full object-cover"
								containerClass="h-full w-full"
							/>

							<!-- Overlay with shader info -->
							<div
								class="absolute inset-0 bg-linear-to-t from-black/80 via-transparent to-transparent"
							>
								<div class="absolute right-0 bottom-0 left-0 p-4">
									<div class="flex items-end justify-between">
										<div>
											<h3 class="mb-2 text-xl font-bold text-white">
												{selectedCapture.shaderName}
											</h3>
											<div class="flex items-center gap-2">
							{#if selectedCapture.profileDisplayName}
											<span class="rounded bg-primary px-2 py-0.5 text-xs font-bold text-primary-foreground">
												{selectedCapture.profileDisplayName}
												</span>
											{/if}
												{#if selectedCapture.shaderVersion}
													<span
														class="rounded bg-white/20 px-2 py-0.5 text-xs font-bold text-white"
													>
														v{selectedCapture.shaderVersion}
													</span>
												{/if}
											</div>
										</div>
									</div>
								</div>
							</div>
						{:else}
							<div class="flex h-full items-center justify-center bg-muted text-muted-foreground">
								<div class="text-center">
									<ImageOff class="mx-auto mb-4 h-16 w-16" strokeWidth={1.5} />
									<p class="text-sm">No captures available yet</p>
								</div>
							</div>
						{/if}
					</div>

					<!-- Shader Thumbnails -->
					{#if capturesByShader.length > 0}
						<div class="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4">
							{#each capturesByShader as capture (capture.id)}
								<button
									type="button"
									onclick={() => (selectedCapture = capture)}
									class="relative overflow-hidden rounded-lg border-2 transition-all hover:border-primary {selectedCapture?.id ===
									capture.id
										? 'border-primary'
										: 'border-transparent'}"
								>
								<CaptureImage
									src={capture.imagePath}
									thumbhash={capture.thumbhash}
									preset="thumbnail"
									alt="{capture.shaderName} thumbnail"
										class="h-full w-full object-cover"
										containerClass="aspect-video"
									/>
								</button>
							{/each}
						</div>
					{/if}
				</div>

				<!-- Scene Info Sidebar -->
				<div class="space-y-4">
					<!-- Basic Info -->
					<div class="rounded-xl border border-border bg-card p-6">
						<h2 class="mb-4 text-2xl font-bold text-card-foreground">{scene.name}</h2>
						{#if scene.description}
							<p class="mb-4 text-sm text-muted-foreground">{scene.description}</p>
						{/if}

					<!-- Environment -->
					{#if timeLabel != null || weatherLabel != null}
						<div class="mb-4 space-y-2">
							{#if timeLabel}
								<div class="flex items-center justify-between text-sm">
									<span class="text-muted-foreground">Time</span>
									<span class="font-medium text-foreground">{timeLabel}</span>
								</div>
							{/if}
							{#if weatherLabel}
								<div class="flex items-center justify-between text-sm">
									<span class="text-muted-foreground">Weather</span>
									<span class="font-medium text-foreground">{weatherLabel}</span>
								</div>
							{/if}
						{#if scene.version?.moonPhase != null}
							<div class="flex items-center justify-between text-sm">
								<span class="text-muted-foreground">Moon</span>
								<span class="font-medium text-foreground">{formatMoonPhase(scene.version.moonPhase)}</span>
							</div>
						{/if}
						</div>
					{/if}

						<!-- Stats -->
						<div class="border-t border-border pt-4">
							<div class="text-2xl font-bold text-foreground">
								{capturesByShader.length}
							</div>
							<div class="text-xs text-muted-foreground">Shaders</div>
						</div>
					</div>
				</div>
			</div>

	<!-- Captures Grid -->
	<div class="mb-8">
		{#if visibleCaptures.length > 0}
			<h2 class="mb-4 text-2xl font-bold text-foreground">Shader Renders</h2>
			<div role="list" class="grid gap-5" style="grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));">
				{#each visibleCaptures as capture (capture.id)}
					<div role="listitem">
						<button
							type="button"
							onclick={() =>
								lightbox.open(
									visibleCaptures.map((c) => ({
										id: c.id,
										imagePath: c.imagePath,
										thumbhash: c.thumbhash,
										profileDisplayName: c.profileDisplayName,
										shaderVersion: c.shaderVersion,
										sceneName: scene.name,
									})),
									visibleCaptures.indexOf(capture),
								)}
							class="group relative overflow-hidden rounded-xl transition-transform hover:scale-[1.02]"
						>
							<CaptureImage
								src={capture.imagePath}
								thumbhash={capture.thumbhash}
								preset="card"
								alt="{capture.shaderName} render"
								class="h-full w-full object-cover"
								containerClass="aspect-video"
							/>
							<ImageOverlay hover />
							<div
								class="absolute right-0 bottom-0 left-0 p-3 opacity-0 transition-opacity group-hover:opacity-100"
							>
								<CaptureBadges
									shaderName={capture.shaderName}
									profileName={capture.profileDisplayName}
									version={capture.shaderVersion}
								/>
							</div>
						</button>
					</div>
				{/each}
			</div>

			{#if hasMoreCaptures}
				<div use:observeSentinel class="flex justify-center py-8"></div>
			{/if}
		{:else}
			<div class="flex flex-col items-center justify-center py-16 text-center">
				<ImageOff class="mb-4 h-16 w-16 text-muted-foreground opacity-50" strokeWidth={1.5} />
				<h3 class="text-lg font-semibold text-foreground">No Captures Yet</h3>
				<p class="mt-1 text-sm text-foreground/70">Captures for this scene are being generated.</p>
			</div>
		{/if}
	</div>
		</div>
	{/key}
{/if}
