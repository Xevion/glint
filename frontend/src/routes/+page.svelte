<script lang="ts">
import { resolve } from '$app/paths';
import { fly, fade } from 'svelte/transition';
import { Button } from '$lib/components/ui/button';
</script>

<div class="container mx-auto px-4 py-16">
	<div class="mx-auto max-w-4xl text-center">
		<h1 in:fly={{ y: -10, duration: 400 }} class="mb-6 text-5xl font-bold tracking-tight">
			Glint <span class="text-muted-foreground">Shader Preview Catalog</span>
		</h1>
		<p in:fly={{ y: 10, duration: 400, delay: 100 }} class="mb-8 text-xl text-muted-foreground">
			Automated shader capture and comparison platform for Minecraft shaders
		</p>

		<div in:fade={{ duration: 300, delay: 200 }} class="mb-12 grid gap-6 md:grid-cols-2">
			<a
				href={resolve('/shaders', {})}
				class="glass group block rounded-lg border p-8 transition hover:border-primary"
			>
				<h2 class="mb-2 text-2xl font-semibold">Browse by Shader</h2>
				<p class="text-muted-foreground">
					Explore all shaders with performance metrics and scene comparisons
				</p>
			</a>

			<a
				href={resolve('/scenes', {})}
				class="glass group block rounded-lg border p-8 transition hover:border-primary"
			>
				<h2 class="mb-2 text-2xl font-semibold">Browse by Scene</h2>
				<p class="text-muted-foreground">
					Compare how different shaders render the same environments
				</p>
			</a>
		</div>

		<div in:fade={{ duration: 300, delay: 300 }} class="flex justify-center gap-4">
			<Button href={resolve('/compare', {})} variant="default" size="lg">Start Comparing</Button>
			<Button href={resolve('/shaders', {})} variant="outline" size="lg">View All Shaders</Button>
		</div>
	</div>
</div>
