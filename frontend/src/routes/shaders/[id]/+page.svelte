<script lang="ts">
	import { resolve } from '$app/paths';
	import {
		getShaderById,
		getCapturesForShader,
		getSceneById,
		formatNumber,
		formatDate,
		getTierColor,
		getTierLabel,
		getStyleColor,
		getTimeColor,
		getWeatherColor,
		type Capture
	} from '$lib/data/mock';
	import { cn } from '$lib/utils';
	import TierIcon from '$lib/components/TierIcon.svelte';
	import BrandIcon from '$lib/components/icons/BrandIcon.svelte';
	import { Button } from '$lib/components/ui/button';

	interface Props {
		data: { id: string };
	}

	let { data }: Props = $props();
	const shader = $derived(getShaderById(data.id));
	const captures = $derived(shader ? getCapturesForShader(data.id) : []);

	// Selected capture for the main preview
	// eslint-disable-next-line svelte/prefer-writable-derived -- user can manually select captures, effect resets on navigation
	let selectedCapture = $state<Capture | null>(null);

	$effect(() => {
		selectedCapture = captures[0] || null;
	});

	// Get scene info for the selected capture
	const selectedScene = $derived(() => {
		if (!selectedCapture) return null;
		return getSceneById(selectedCapture.sceneId);
	});
</script>

{#if shader}
	<div class="container mx-auto px-4 py-8">
		<!-- Breadcrumb -->
		<nav class="mb-6 flex items-center gap-2 text-sm text-muted-foreground">
			<a href={resolve('/shaders')} class="transition-colors hover:text-foreground">Shaders</a>
			<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
				<path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
			</svg>
			<span class="font-medium text-foreground">{shader.name}</span>
		</nav>

		<!-- Hero: Preview + Info -->
		<div class="mb-8 grid gap-6 lg:grid-cols-3">
			<!-- Main Preview Area -->
			<div class="space-y-4 lg:col-span-2">
				<!-- Main Image -->
				<div
					class="shadow-theme-lg relative aspect-video w-full overflow-hidden rounded-xl bg-card"
				>
					{#if selectedCapture}
						<img
							src={selectedCapture.image}
							alt="{shader.name} in {selectedScene()?.name}"
							class="h-full w-full object-cover"
						/>

						<!-- Overlay with capture info -->
						<div
							class="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent"
						>
							<div class="absolute right-0 bottom-0 left-0 p-4">
								<div class="flex items-end justify-between">
									<div>
										<h3 class="mb-2 text-xl font-bold text-white">{selectedScene()?.name}</h3>
										<div class="flex items-center gap-2">
											<span
												class={cn(
													'rounded px-2 py-0.5 text-xs font-bold capitalize',
													getTimeColor(selectedCapture.timeOfDay)
												)}
											>
												{selectedCapture.timeOfDay}
											</span>
											<span
												class={cn(
													'rounded px-2 py-0.5 text-xs font-bold capitalize',
													getWeatherColor(selectedCapture.weather)
												)}
											>
												{selectedCapture.weather}
											</span>
										</div>
									</div>
								</div>
							</div>
						</div>
					{/if}
				</div>

				<!-- Scene Thumbnails -->
				<div class="scrollbar-thin flex gap-2 overflow-x-auto pb-2">
					{#each captures as capture (capture.id)}
						{@const scene = getSceneById(capture.sceneId)}
						<button
							onclick={() => (selectedCapture = capture)}
							class={cn(
								'relative aspect-video w-36 flex-shrink-0 overflow-hidden rounded-lg transition-all',
								'hover:ring-2 hover:ring-primary/50',
								selectedCapture?.id === capture.id
									? 'shadow-lg ring-2 ring-primary'
									: 'opacity-60 hover:opacity-100'
							)}
						>
							<img
								src={capture.image}
								alt={scene?.name}
								class="h-full w-full object-cover"
								loading="lazy"
							/>
							<div class="absolute inset-0 bg-gradient-to-t from-black/70 to-transparent"></div>
							<div class="absolute right-2 bottom-1.5 left-2">
								<div class="truncate text-xs font-medium text-white">{scene?.name}</div>
							</div>
						</button>
					{/each}
				</div>
			</div>

			<!-- Sidebar -->
			<div class="space-y-4">
				<!-- Shader Info Card -->
				<div class="shadow-theme-sm rounded-xl bg-card p-6">
					<div class="mb-4 flex items-start justify-between">
						<div>
							<h1 class="text-2xl font-bold text-card-foreground">{shader.name}</h1>
							<div class="mt-1 text-sm text-muted-foreground">
								<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- external link -->
								by
								<a
									href={shader.authorUrl}
									target="_blank"
									rel="noopener noreferrer"
									class="font-medium text-card-foreground transition-colors hover:text-primary"
									>{shader.author}</a
								>
							</div>
						</div>
						<span
							class={cn(
								'inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm font-bold shadow-sm',
								getTierColor(shader.tier)
							)}
						>
							<TierIcon tier={shader.tier} size={16} />
							{getTierLabel(shader.tier)}
						</span>
					</div>

					<p class="mb-4 text-sm text-muted-foreground">{shader.description}</p>

					<!-- Style badge -->
					<div class="mb-4 flex items-center gap-2">
						<span
							class={cn(
								'rounded-lg px-3 py-1 text-sm font-medium capitalize',
								getStyleColor(shader.style)
							)}
						>
							{shader.style}
						</span>
					</div>

					<!-- Quick stats -->
					<div class="grid grid-cols-3 gap-3 border-y border-border py-4">
						<div class="text-center">
							<div class="text-xl font-bold text-card-foreground">
								{formatNumber(shader.downloadCount)}
							</div>
							<div class="text-xs text-muted-foreground">Downloads</div>
						</div>
						<div class="text-center">
							<div class="text-xl font-bold text-card-foreground">{formatNumber(shader.likes)}</div>
							<div class="text-xs text-muted-foreground">Likes</div>
						</div>
						<div class="text-center" title="Supports {shader.mcVersions.join(', ')}">
							<div class="text-xl font-bold text-card-foreground">
								{shader.mcVersions[shader.mcVersions.length - 1]}
							</div>
							<div class="text-xs text-muted-foreground">MC Version</div>
						</div>
					</div>

					<!-- Meta info -->
					<dl class="mt-4 space-y-2 text-sm">
						<div class="flex justify-between">
							<dt class="text-muted-foreground">Version</dt>
							<dd class="font-medium">{shader.version}</dd>
						</div>
						<div class="flex justify-between">
							<dt class="text-muted-foreground">Minecraft</dt>
							<dd class="font-medium">{shader.mcVersions.join(', ')}</dd>
						</div>
						<div class="flex justify-between">
							<dt class="text-muted-foreground">Updated</dt>
							<dd class="font-medium">{formatDate(shader.lastUpdated)}</dd>
						</div>
					</dl>
				</div>

				<!-- Actions -->
				<div class="flex flex-col gap-2">
					{#if shader.modrinthUrl}
						<Button href={shader.modrinthUrl} target="_blank" class="group/modrinth w-full gap-2">
							<BrandIcon name="modrinth" colorOnHover />
							Get on Modrinth
						</Button>
					{/if}
					{#if shader.curseforgeUrl}
						<Button
							href={shader.curseforgeUrl}
							target="_blank"
							variant="secondary"
							class="group/curseforge w-full gap-2"
						>
							<BrandIcon name="curseforge" colorOnHover />
							Get on CurseForge
						</Button>
					{/if}
					<Button
						href={`${resolve('/compare')}?shader=${shader.id}`}
						variant="outline"
						class="w-full gap-2"
					>
						<svg
							class="h-4 w-4"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							stroke-width="2"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2"
							/>
						</svg>
						Compare with Others
					</Button>
				</div>
			</div>
		</div>

		<!-- Features -->
		<div class="shadow-theme-sm mb-8 rounded-xl bg-card p-6">
			<h2 class="mb-4 text-lg font-bold text-card-foreground">Features</h2>
			<div class="grid gap-3 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
				{#each shader.features as feature (feature.name)}
					<div class="flex items-center gap-2 rounded-lg bg-muted/50 px-3 py-2">
						<svg
							class="h-4 w-4 flex-shrink-0 text-emerald-500"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							stroke-width="2.5"
						>
							<path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
						</svg>
						<span class="text-sm text-card-foreground">{feature.name}</span>
						<span class="ml-auto text-xs text-muted-foreground capitalize">{feature.category}</span>
					</div>
				{/each}
			</div>
		</div>

		<!-- All Captures Gallery -->
		<div>
			<div class="mb-4 flex items-center justify-between">
				<h2 class="text-lg font-bold text-foreground">All Scene Captures</h2>
			</div>
			<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
				{#each captures as capture (capture.id)}
					{@const scene = getSceneById(capture.sceneId)}
					<a
						href={resolve('/scenes/[id]', { id: capture.sceneId })}
						class="group shadow-theme-sm relative overflow-hidden rounded-xl bg-card transition-all hover:-translate-y-1 hover:ring-2 hover:ring-primary/50"
					>
						<div class="aspect-video overflow-hidden">
							<img
								src={capture.image}
								alt="{shader.name} in {scene?.name}"
								class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
								loading="lazy"
							/>
						</div>
						<div class="p-3">
							<div class="mb-1">
								<h3
									class="truncate font-medium text-card-foreground transition-colors group-hover:text-primary"
								>
									{scene?.name}
								</h3>
							</div>
							<div class="flex items-center gap-2">
								<span
									class={cn(
										'rounded px-1.5 py-0.5 text-xs font-medium capitalize',
										getTimeColor(capture.timeOfDay)
									)}
								>
									{capture.timeOfDay}
								</span>
								<span
									class={cn(
										'rounded px-1.5 py-0.5 text-xs font-medium capitalize',
										getWeatherColor(capture.weather)
									)}
								>
									{capture.weather}
								</span>
							</div>
						</div>
					</a>
				{/each}
			</div>
		</div>
	</div>
{:else}
	<div class="container mx-auto px-4 py-16 text-center">
		<svg
			class="mx-auto mb-4 h-16 w-16 text-muted-foreground/50"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
			stroke-width="1.5"
		>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				d="M9.75 9.75l4.5 4.5m0-4.5l-4.5 4.5M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
			/>
		</svg>
		<h1 class="mb-2 text-2xl font-bold text-foreground">Shader Not Found</h1>
		<p class="mb-6 text-muted-foreground">The shader you're looking for doesn't exist.</p>
		<Button href={resolve('/shaders')}>Back to Shaders</Button>
	</div>
{/if}
