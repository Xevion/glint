<script lang="ts">
import { resolve } from '$app/paths';
import type { CaptureWithContext } from '$lib/bindings';
import CaptureBadges from '$lib/components/CaptureBadges.svelte';
import CaptureGallery from '$lib/components/CaptureGallery.svelte';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import Meta from '$lib/components/Meta.svelte';
import { ChevronRight, ImageOff } from '@lucide/svelte';

import { fly } from 'svelte/transition';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}

let { data }: Props = $props();
const scene = $derived(data.scene);
const captures = $derived(scene.captures);

// Selected capture for the main preview
let selectedCapture = $state<CaptureWithContext | null>(null);

// Initialize selected capture when captures change
$effect(() => {
	if (captures.length > 0 && !selectedCapture) {
		selectedCapture = captures[0];
	}
});

// Group captures by shader
const capturesByShader = $derived.by(() => {
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- ephemeral map inside $derived.by
	const map = new Map<string, CaptureWithContext>();
	for (const capture of captures) {
		if (!map.has(capture.shader_slug)) {
			map.set(capture.shader_slug, capture);
		}
	}
	return Array.from(map.values());
});

// Weather display helper
const weatherLabel = $derived.by(() => {
	const v = scene.version;
	if (!v) return null;
	if (v.weather === 'clear') return 'Clear';
	if (v.weather === 'rain') return `Rain (${Math.round(v.weather_intensity * 100)}%)`;
	if (v.weather === 'thunder') return `Thunder (${Math.round(v.weather_intensity * 100)}%)`;
	return v.weather;
});

// Time display helper
const timeLabel = $derived.by(() => {
	const v = scene.version;
	if (!v) return null;
	const ticks = v.time_of_day_ticks;
	const hours = Math.floor(ticks / 1000);
	if (hours < 6) return 'Night';
	if (hours < 12) return 'Morning';
	if (hours < 18) return 'Day';
	return 'Evening';
});

// OG metadata
const ogImage = $derived(captures[0]?.image_url ?? null);
const ogDescription = $derived.by(() => {
	const parts = [`${scene.name} scene for Minecraft shader comparison`];
	if (captures.length > 0)
		parts.push(`${captures.length} shader screenshot${captures.length === 1 ? '' : 's'}`);
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
								src={selectedCapture.image_url}
								thumbhash={selectedCapture.thumbhash}
								preset="hero"
								priority
								alt="{scene.name} with {selectedCapture.shader_name}"
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
												{selectedCapture.shader_name}
											</h3>
											<div class="flex items-center gap-2">
								{#if selectedCapture.profile_name}
												<span class="rounded bg-primary px-2 py-0.5 text-xs font-bold text-primary-foreground">
													{selectedCapture.profile_name}
												</span>
											{/if}
												{#if selectedCapture.shader_version}
													<span
														class="rounded bg-white/20 px-2 py-0.5 text-xs font-bold text-white"
													>
														v{selectedCapture.shader_version}
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
										src={capture.image_url}
										thumbhash={capture.thumbhash}
										preset="thumbnail"
										alt="{capture.shader_name} thumbnail"
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
		<CaptureGallery
			{captures}
			title="Shader Renders"
			emptyMessage="Captures for this scene are being generated."
			onclick={(capture: CaptureWithContext) => (selectedCapture = capture)}
			alt={(capture: CaptureWithContext) => `${capture.shader_name} render`}
		>
		{#snippet overlay(capture: CaptureWithContext)}
			<CaptureBadges
				shaderName={capture.shader_name}
				profileName={capture.profile_name}
				version={capture.shader_version}
			/>
		{/snippet}
		</CaptureGallery>
		</div>
	{/key}
{/if}
