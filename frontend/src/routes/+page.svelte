<script lang="ts">
import { resolve } from '$app/paths';
import HeroSlider from '$lib/components/hero/HeroSlider.svelte';
import ShaderCard from '$lib/components/ShaderCard.svelte';
import { Button } from '$lib/components/ui/button';
import { fade, fly } from 'svelte/transition';
import type { PageData } from './$types';

let { data }: { data: PageData } = $props();

// Featured shaders (first 6)
const featuredShaders = $derived((data.shaders ?? []).slice(0, 6));

// Stats
const captures = $derived(data.captures ?? []);
const shaders = $derived(data.shaders ?? []);
const featuredPairs = $derived(data.featuredPairs ?? []);
const hasStats = $derived(shaders.length > 0 || captures.length > 0);
const hasFeaturedPairs = $derived(featuredPairs.length > 0);

const sceneCount = $derived(new Set(captures.map((c) => c.scene_id)).size);
</script>

{#snippet statsBar()}
	<div class="inline-flex items-center gap-6 rounded-full bg-muted/50 backdrop-blur-sm px-6 py-2 text-sm">
		<span><strong class="text-foreground">{shaders.length}</strong> <span class="text-muted-foreground">Shaders</span></span>
		<span class="text-muted-foreground/30">|</span>
		<span><strong class="text-foreground">{sceneCount}</strong> <span class="text-muted-foreground">Scenes</span></span>
		<span class="text-muted-foreground/30">|</span>
		<span><strong class="text-foreground">{captures.length}</strong> <span class="text-muted-foreground">Captures</span></span>
	</div>
{/snippet}

<svelte:head><title>Glint</title></svelte:head>

<!-- Hero Section -->
{#if hasFeaturedPairs}
	<div in:fade={{ duration: 400 }} class="py-8 sm:py-12">
		<!-- Hero slider with overlay text -->
		<div class="relative">
			<HeroSlider pairs={featuredPairs} />

			<!-- Desktop overlay: scrim + centered text (hidden on mobile) -->
			<div
				class="pointer-events-none absolute inset-0 z-30 hidden select-none sm:flex flex-col items-center justify-center"
				style="background: radial-gradient(ellipse at center, rgba(0,0,0,0.4) 0%, rgba(0,0,0,0.15) 40%, transparent 70%);"
			>
				<h1
					class="text-5xl font-bold tracking-tight text-white drop-shadow-lg lg:text-6xl"
					style="text-shadow: 0 2px 12px rgba(0,0,0,0.5);"
				>
					Glint
				</h1>
				<p
					class="mt-2 text-xl text-white/90 drop-shadow-md"
					style="text-shadow: 0 1px 8px rgba(0,0,0,0.5);"
				>
					Shader Preview Catalog for Minecraft
				</p>
				<div class="pointer-events-auto mt-6 flex gap-4">
					<Button href={resolve('/shaders', {})} size="lg" class="shadow-lg">Browse Shaders</Button>
					<Button href={resolve('/compare', {})} variant="outline" size="lg" class="border-white/50 bg-black/30 text-white shadow-lg backdrop-blur-sm hover:bg-black/50">Compare</Button>
				</div>
			</div>
		</div>

		<!-- Mobile hero text (below slider, not overlaid) -->
		<div class="mt-4 text-center sm:hidden">
			<h1 class="text-3xl font-bold tracking-tight">Glint</h1>
			<p class="mt-1 text-base text-muted-foreground">Shader Preview Catalog for Minecraft</p>
			<div class="mt-4 flex justify-center gap-3">
				<Button href={resolve('/shaders', {})} size="default">Browse Shaders</Button>
				<Button href={resolve('/compare', {})} variant="outline" size="default">Compare</Button>
			</div>
		</div>

		<!-- Stats (below hero) -->
		{#if hasStats}
			<div
				in:fly={{ y: 10, duration: 400, delay: 200 }}
				class="mt-6 flex justify-center"
			>
				{@render statsBar()}
			</div>
		{/if}
	</div>
{:else}
	<!-- Fallback: text-only hero when no featured pairs available -->
	<div class="py-16 sm:py-24 text-center">
		<h1
			in:fly={{ y: -20, duration: 500 }}
			class="text-5xl font-bold tracking-tight sm:text-6xl lg:text-7xl"
		>
			Glint
		</h1>
		<p
			in:fly={{ y: -10, duration: 500, delay: 100 }}
			class="mt-4 text-xl text-muted-foreground sm:text-2xl"
		>
			Shader Preview Catalog for Minecraft
		</p>

		<div in:fade={{ duration: 400, delay: 200 }} class="mt-10 flex justify-center gap-4">
			<Button href={resolve('/shaders', {})} size="lg">Browse Shaders</Button>
			<Button href={resolve('/compare', {})} variant="outline" size="lg">Compare</Button>
		</div>

		{#if hasStats}
			<div
				in:fly={{ y: 10, duration: 400, delay: 300 }}
				class="mt-12"
			>
				{@render statsBar()}
			</div>
		{/if}
	</div>
{/if}

<!-- Featured Shaders -->
{#if featuredShaders.length > 0}
	<section in:fly={{ y: 20, duration: 400, delay: 400 }} class="py-8">
		<div class="mb-6 flex items-center justify-between">
			<h2 class="text-xl font-semibold">Featured Shaders</h2>
			<a
				href={resolve('/shaders', {})}
				class="text-sm text-muted-foreground hover:text-foreground transition-colors"
			>
				View all →
			</a>
		</div>

		<div class="grid gap-5" style="grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));">
			{#each featuredShaders as shader, i (shader.id)}
				<div in:fly={{ y: 20, duration: 300, delay: 450 + i * 50 }}>
					<ShaderCard {shader} />
				</div>
			{/each}
		</div>
	</section>
{/if}

<!-- Quick Links -->
<section in:fade={{ duration: 400, delay: 600 }} class="py-8">
	<div class="grid gap-4 sm:grid-cols-2">
		<a
			href={resolve('/scenes', {})}
			class="group rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm p-6 transition-all hover:border-primary/50 hover:bg-card/50"
		>
			<h3 class="text-lg font-medium mb-2">Browse by Scene</h3>
			<p class="text-sm text-muted-foreground">
				Compare how different shaders render the same environments
			</p>
		</a>

		<a
			href={resolve('/compare', {})}
			class="group rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm p-6 transition-all hover:border-primary/50 hover:bg-card/50"
		>
			<h3 class="text-lg font-medium mb-2">Side-by-Side Compare</h3>
			<p class="text-sm text-muted-foreground">
				Put shaders head-to-head to find your perfect match
			</p>
		</a>
	</div>
</section>
