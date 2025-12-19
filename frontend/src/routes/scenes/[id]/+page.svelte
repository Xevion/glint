<script lang="ts">
	import { page } from '$app/stores';

	const sceneId = $page.params.id || '1';
	const sceneNames = ['Forest', 'Village', 'Cave', 'Nether', 'End', 'Ocean'];
	const index = parseInt(sceneId) - 1;
	const sceneName = index >= 0 && index < sceneNames.length ? sceneNames[index] : 'Unknown';
</script>

<div class="container mx-auto px-4 py-8">
	<div class="mb-8">
		<h1 class="mb-2 text-4xl font-bold">{sceneName} Scene</h1>
		<p class="text-muted-foreground">All shaders rendered in this scene</p>
	</div>

	<div class="mb-8">
		<div class="aspect-video rounded-lg bg-muted"></div>
		<p class="mt-2 text-sm text-muted-foreground">Reference scene preview</p>
	</div>

	<div>
		<h2 class="mb-4 text-2xl font-semibold">All Shaders in {sceneName}</h2>
		<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
			{#each Array(6) as _, i}
				<a href="/shaders/{i + 1}" class="rounded-lg border p-4 transition hover:border-primary">
					<div class="mb-3 aspect-video rounded bg-muted"></div>
					<h3 class="font-medium">Shader Pack {i + 1}</h3>
					<p class="text-sm text-muted-foreground">60 FPS avg</p>
				</a>
			{/each}
		</div>
	</div>
</div>
