<script lang="ts">
import { resolve } from '$app/paths';
import type { CaptureWithContext, SceneWithCaptures } from '$lib/bindings';
import { cfImageUrl } from '$lib/utils/image';
import { ChevronRight, ImageOff } from '@lucide/svelte';
import { SvelteMap } from 'svelte/reactivity';
import { fade, fly, scale } from 'svelte/transition';

interface Props {
	data: { scene: SceneWithCaptures };
}

let { data }: Props = $props();
const scene = $derived(data.scene);
const captures = $derived(scene.captures);

// Selected capture for the main preview
let selectedCapture = $derived(captures[0] ?? null);

// Group captures by shader
const capturesByShader = $derived.by(() => {
	const map = new SvelteMap<string, CaptureWithContext>();
	for (const capture of captures) {
		if (!map.has(capture.shader_slug)) {
			map.set(capture.shader_slug, capture);
		}
	}
	return Array.from(map.values());
});

// Weather display helper
const weatherLabel = $derived.by(() => {
	if (scene.weather === 'clear') return 'Clear';
	if (scene.weather === 'rain') return `Rain (${Math.round(scene.weather_intensity * 100)}%)`;
	if (scene.weather === 'thunder') return `Thunder (${Math.round(scene.weather_intensity * 100)}%)`;
	return scene.weather;
});

// Time display helper
const timeLabel = $derived.by(() => {
	const ticks = scene.time_of_day_ticks;
	const hours = Math.floor(ticks / 1000);
	if (hours < 6) return 'Night';
	if (hours < 12) return 'Morning';
	if (hours < 18) return 'Day';
	return 'Evening';
});
</script>

{#if scene}
	{#key scene.id}
		<div class="container mx-auto px-4 py-8">
			<!-- Breadcrumb -->
			<nav
				in:fly|local={{ y: -10, duration: 400 }}
				class="mb-6 flex items-center gap-2 text-sm text-muted-foreground"
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
					<div
						class="shadow-theme-lg relative aspect-video w-full overflow-hidden rounded-xl bg-card"
					>
						{#if selectedCapture}
							<img
								src={cfImageUrl(selectedCapture.screenshot_url, 'hero')}
								alt="{scene.name} with {selectedCapture.shader_name}"
								class="h-full w-full object-cover"
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
												{#if selectedCapture.profile}
													<span class="rounded bg-primary px-2 py-0.5 text-xs font-bold text-white">
														{selectedCapture.profile}
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
							<div class="flex h-full items-center justify-center text-muted-foreground">
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
									class="aspect-video overflow-hidden rounded-lg border-2 transition-all hover:border-primary {selectedCapture?.id ===
									capture.id
										? 'border-primary'
										: 'border-transparent'}"
								>
									{#if capture.screenshot_url}
										<img
											src={cfImageUrl(capture.screenshot_url, 'thumbnail')}
											alt="{capture.shader_name} thumbnail"
											class="h-full w-full object-cover"
										/>
									{/if}
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
						<div class="mb-4 space-y-2">
							<div class="flex items-center justify-between text-sm">
								<span class="text-muted-foreground">Time</span>
								<span class="font-medium text-foreground">{timeLabel}</span>
							</div>
							<div class="flex items-center justify-between text-sm">
								<span class="text-muted-foreground">Weather</span>
								<span class="font-medium text-foreground">{weatherLabel}</span>
							</div>
						</div>

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
			{#if captures.length > 0}
				<div in:fade|local={{ duration: 300, delay: 200 }} class="mb-8">
					<h2 class="mb-4 text-2xl font-bold text-foreground">All Shader Renders</h2>
					<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
						{#each captures as capture, i (capture.id)}
							<button
								type="button"
								onclick={() => (selectedCapture = capture)}
								in:scale|local={{ duration: 350, delay: Math.min(i * 50, 400) + 250, start: 0.95 }}
								class="group relative aspect-video overflow-hidden rounded-xl bg-card transition-transform hover:scale-[1.02]"
							>
								{#if capture.screenshot_url}
									<img
										src={cfImageUrl(capture.screenshot_url, 'card')}
										alt="{capture.shader_name} render"
										class="h-full w-full object-cover"
									/>
									<div
										class="absolute inset-0 bg-linear-to-t from-black/60 to-transparent opacity-0 transition-opacity group-hover:opacity-100"
									>
										<div class="absolute right-0 bottom-0 left-0 p-3">
											<div class="mb-1 font-bold text-white">{capture.shader_name}</div>
											<div class="flex items-center gap-2">
												{#if capture.profile}
													<span class="rounded bg-primary px-2 py-0.5 text-xs font-bold text-white">
														{capture.profile}
													</span>
												{/if}
												{#if capture.shader_version}
													<span
														class="rounded bg-white/20 px-2 py-0.5 text-xs font-bold text-white"
													>
														v{capture.shader_version}
													</span>
												{/if}
											</div>
										</div>
									</div>
								{/if}
							</button>
						{/each}
					</div>
				</div>
			{:else}
				<div
					in:fade|local={{ duration: 300, delay: 200 }}
					class="mb-8 rounded-xl border border-border bg-card p-12 text-center"
				>
					<ImageOff class="mx-auto mb-4 h-16 w-16 text-muted-foreground opacity-30" strokeWidth={1.5} />
					<h3 class="mb-2 text-lg font-semibold text-card-foreground">No Captures Yet</h3>
					<p class="text-sm text-muted-foreground">Captures for this scene are being generated.</p>
				</div>
			{/if}
		</div>
	{/key}
{/if}
