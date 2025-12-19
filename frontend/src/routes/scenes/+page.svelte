<script lang="ts">
	// Placeholder for scene list page
</script>

<div class="container mx-auto px-4 py-8">
	<h1 class="mb-6 text-4xl font-bold">Scenes</h1>
	<p class="text-muted-foreground">Browse all test scenes used for shader capture</p>

	<div class="mt-8 grid gap-6 md:grid-cols-2 lg:grid-cols-3">
		{#each ['Forest', 'Village', 'Cave', 'Nether', 'End', 'Ocean'] as scene, i}
			<a
				href="/scenes/{i + 1}"
				class="group block rounded-lg border p-6 transition hover:border-primary"
			>
				<div class="mb-4 aspect-video rounded bg-muted"></div>
				<h3 class="mb-2 text-xl font-semibold">{scene}</h3>
				<p class="text-sm text-muted-foreground">Compare all shaders in this environment</p>
			</a>
		{/each}
	</div>
</div>
