<script lang="ts">
	// Placeholder for shader list page
</script>

<div class="container mx-auto px-4 py-8">
	<h1 class="mb-6 text-4xl font-bold">Shaders</h1>
	<p class="text-muted-foreground">Browse all available Minecraft shaders with performance metrics</p>

	<div class="mt-8 grid gap-6 md:grid-cols-2 lg:grid-cols-3">
		{#each Array(6) as _, i}
			<a
				href="/shaders/{i + 1}"
				class="group block rounded-lg border p-6 transition hover:border-primary"
			>
				<div class="mb-4 aspect-video rounded bg-muted"></div>
				<h3 class="mb-2 text-xl font-semibold">Shader Pack {i + 1}</h3>
				<p class="text-sm text-muted-foreground">
					Placeholder shader with preview and performance data
				</p>
			</a>
		{/each}
	</div>
</div>
