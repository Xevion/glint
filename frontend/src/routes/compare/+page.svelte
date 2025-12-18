<script lang="ts">
	import { Button } from '$lib/components/ui/button';

	// Placeholder comparison state
	let selectedShaders: string[] = [];
</script>

<div class="container mx-auto px-4 py-8">
	<h1 class="mb-6 text-4xl font-bold">Compare Shaders</h1>
	<p class="mb-8 text-muted-foreground">
		Select shaders to compare side-by-side across different scenes
	</p>

	<div class="mb-8 rounded-lg border p-6">
		<h2 class="mb-4 text-xl font-semibold">Selected Shaders</h2>
		{#if selectedShaders.length === 0}
			<p class="text-muted-foreground">No shaders selected. Choose from the list below.</p>
		{:else}
			<div class="flex gap-4">
				{#each selectedShaders as shader}
					<div class="rounded border p-4">{shader}</div>
				{/each}
			</div>
		{/if}
	</div>

	<div>
		<h2 class="mb-4 text-xl font-semibold">Available Shaders</h2>
		<div class="grid gap-4 md:grid-cols-3 lg:grid-cols-4">
			{#each Array(8) as _, i}
				<button
					class="rounded-lg border p-4 text-left transition hover:border-primary"
					onclick={() => {
						const id = `shader-${i + 1}`;
						if (selectedShaders.includes(id)) {
							selectedShaders = selectedShaders.filter((s) => s !== id);
						} else if (selectedShaders.length < 4) {
							selectedShaders = [...selectedShaders, id];
						}
					}}
				>
					<div class="mb-2 aspect-video rounded bg-muted"></div>
					<p class="font-medium">Shader {i + 1}</p>
				</button>
			{/each}
		</div>
	</div>
</div>
