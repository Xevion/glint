<script lang="ts">
	import { resolve } from '$app/paths';
	import {
		getSceneById,
		getCapturesForScene,
		getShaderById,
		getTierColor,
		getTierLabel,
		getTimeColor,
		getWeatherColor,
		getDimensionColor,
		type Capture,
		type PerformanceTier
	} from '$lib/data/mock';
	import { cn } from '$lib/utils';
	import TierIcon from '$lib/components/TierIcon.svelte';
	import { Button } from '$lib/components/ui/button';

	interface Props {
		data: { id: string };
	}

	let { data }: Props = $props();
	const scene = $derived(getSceneById(data.id));
	const captures = $derived(scene ? getCapturesForScene(data.id) : []);

	// Selected capture for the main preview
	// eslint-disable-next-line svelte/prefer-writable-derived -- user can manually select captures, effect resets on navigation
	let selectedCapture = $state<Capture | null>(null);

	$effect(() => {
		selectedCapture = captures[0] || null;
	});

	// Get shader info for the selected capture
	const selectedShader = $derived(() => {
		if (!selectedCapture) return null;
		return getShaderById(selectedCapture.shaderId);
	});

	// Group captures by performance tier
	const capturesByTier = $derived(() => {
		const groups: Record<PerformanceTier, number> = {
			potato: 0,
			low: 0,
			medium: 0,
			high: 0,
			ultra: 0
		};
		for (const capture of captures) {
			const shader = getShaderById(capture.shaderId);
			if (shader) {
				groups[shader.tier]++;
			}
		}
		return groups;
	});
</script>

{#if scene}
	<div class="container mx-auto px-4 py-8">
		<!-- Breadcrumb -->
		<nav class="mb-6 flex items-center gap-2 text-sm text-muted-foreground">
			<a href={resolve('/scenes')} class="transition-colors hover:text-foreground">Scenes</a>
			<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
				<path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
			</svg>
			<span class="font-medium text-foreground">{scene.name}</span>
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
						{@const shader = selectedShader()}
						<img
							src={selectedCapture.image}
							alt="{scene.name} with {shader?.name}"
							class="h-full w-full object-cover"
						/>

						<!-- Overlay with shader info -->
						<div
							class="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent"
						>
							<div class="absolute right-0 bottom-0 left-0 p-4">
								<div class="flex items-end justify-between">
									<div>
										<h3 class="mb-2 text-xl font-bold text-white">{shader?.name}</h3>
										<div class="flex items-center gap-2">
											<span class="text-sm text-white/70">by {shader?.author}</span>
											{#if shader}
												<span
													class={cn(
														'inline-flex items-center gap-1 rounded px-2 py-0.5 text-xs font-bold',
														getTierColor(shader.tier)
													)}
												>
													<TierIcon tier={shader.tier} size={12} />
													{getTierLabel(shader.tier)}
												</span>
											{/if}
										</div>
									</div>
								</div>
							</div>
						</div>
					{/if}
				</div>

				<!-- Shader Thumbnails -->
				<div class="scrollbar-thin flex gap-2 overflow-x-auto pb-2">
					{#each captures as capture (capture.id)}
						{@const shader = getShaderById(capture.shaderId)}
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
								alt={shader?.name}
								class="h-full w-full object-cover"
								loading="lazy"
							/>
							<div class="absolute inset-0 bg-gradient-to-t from-black/70 to-transparent"></div>
							<div class="absolute right-2 bottom-1.5 left-2">
								<div class="truncate text-xs font-medium text-white">{shader?.name}</div>
							</div>
						</button>
					{/each}
				</div>
			</div>

			<!-- Sidebar -->
			<div class="space-y-4">
				<!-- Scene Info Card -->
				<div class="shadow-theme-sm rounded-xl bg-card p-6">
					<h1 class="mb-3 text-2xl font-bold text-card-foreground">{scene.name}</h1>

					<!-- Scene attributes -->
					<div class="mb-4 flex flex-wrap items-center gap-2">
						<span
							class={cn(
								'rounded-lg px-3 py-1 text-sm font-medium capitalize',
								getDimensionColor(scene.dimension)
							)}
						>
							{scene.dimension}
						</span>
						<span
							class={cn(
								'rounded-lg px-3 py-1 text-sm font-bold capitalize',
								getTimeColor(scene.defaultTime)
							)}
						>
							{scene.defaultTime}
						</span>
						<span
							class={cn(
								'rounded-lg px-3 py-1 text-sm font-bold capitalize',
								getWeatherColor(scene.defaultWeather)
							)}
						>
							{scene.defaultWeather}
						</span>
					</div>

					<p class="mb-4 text-sm text-muted-foreground">{scene.description}</p>

					<!-- Biome & complexity -->
					<div class="mb-4 flex items-center gap-3 text-sm text-muted-foreground">
						<span class="font-medium text-card-foreground">{scene.biome}</span>
						<span>·</span>
						<span class="capitalize">{scene.complexity} scene</span>
					</div>

					<!-- Stats -->
					<div class="grid grid-cols-2 gap-3 border-y border-border py-4">
						<div class="text-center">
							<div class="text-xl font-bold text-card-foreground">{captures.length}</div>
							<div class="text-xs text-muted-foreground">Shaders</div>
						</div>
						<div class="text-center">
							<div class="text-xl font-bold text-card-foreground">{captures.length}</div>
							<div class="text-xs text-muted-foreground">Captures</div>
						</div>
					</div>

					<!-- Features -->
					<div class="mt-4">
						<h3 class="mb-2 text-sm font-medium text-muted-foreground">Scene Features</h3>
						<div class="flex flex-wrap gap-1.5">
							{#each scene.features as feature (feature)}
								<span
									class="rounded bg-muted/50 px-2 py-1 text-xs text-muted-foreground ring-1 ring-border/50"
								>
									{feature}
								</span>
							{/each}
						</div>
					</div>
				</div>

				<!-- Performance Tier Breakdown -->
				<div class="shadow-theme-sm rounded-xl bg-card p-6">
					<h2 class="mb-4 text-sm font-bold tracking-wider text-muted-foreground uppercase">
						By Weight
					</h2>
					<div class="space-y-2">
						{#each ['potato', 'low', 'medium', 'high', 'ultra'] as const as tier (tier)}
							{@const count = capturesByTier()[tier]}
							{@const percentage = (count / captures.length) * 100}
							<div class="flex items-center gap-3">
								<span
									class={cn(
										'w-24 rounded px-2 py-1 text-center text-xs font-bold',
										getTierColor(tier)
									)}
								>
									{getTierLabel(tier)}
								</span>
								<div class="h-2 flex-1 overflow-hidden rounded-full bg-muted">
									<div
										class="h-full rounded-full bg-primary transition-all"
										style="width: {percentage}%"
									></div>
								</div>
								<span class="w-8 text-right text-xs text-muted-foreground">{count}</span>
							</div>
						{/each}
					</div>
				</div>

				<!-- Action -->
				<Button
					href={`${resolve('/compare')}?scene=${scene.id}`}
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
					Compare Shaders in This Scene
				</Button>
			</div>
		</div>

		<!-- All Shader Renders Gallery -->
		<div>
			<div class="mb-4 flex items-center justify-between">
				<h2 class="text-lg font-bold text-foreground">All Shader Renders</h2>
			</div>
			<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
				{#each captures as capture (capture.id)}
					{@const shader = getShaderById(capture.shaderId)}
					<a
						href={resolve('/shaders/[id]', { id: capture.shaderId })}
						class="group shadow-theme-sm relative overflow-hidden rounded-xl bg-card transition-all hover:-translate-y-1 hover:ring-2 hover:ring-primary/50"
					>
						<div class="aspect-video overflow-hidden">
							<img
								src={capture.image}
								alt="{shader?.name} in {scene.name}"
								class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
								loading="lazy"
							/>
							<!-- Performance tier badge -->
							{#if shader}
								<div class="absolute top-2 right-2">
									<span
										class={cn(
											'inline-flex items-center gap-1 rounded px-2 py-0.5 text-xs font-bold shadow-sm',
											getTierColor(shader.tier)
										)}
									>
										<TierIcon tier={shader.tier} size={12} />
										{getTierLabel(shader.tier)}
									</span>
								</div>
							{/if}
						</div>
						<div class="p-3">
							<div class="mb-1">
								<h3
									class="truncate font-medium text-card-foreground transition-colors group-hover:text-primary"
								>
									{shader?.name}
								</h3>
							</div>
							<p class="text-xs text-muted-foreground">by {shader?.author}</p>
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
		<h1 class="mb-2 text-2xl font-bold text-foreground">Scene Not Found</h1>
		<p class="mb-6 text-muted-foreground">The scene you're looking for doesn't exist.</p>
		<Button href={resolve('/scenes')}>Back to Scenes</Button>
	</div>
{/if}
