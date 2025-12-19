<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { comparisonStore } from '$lib/stores/comparison.svelte';
	import { getAllScenes, getShaderById, getSceneById, getCapture } from '$lib/data/mock';

	const scenes = getAllScenes();

	// Get selected items from store
	const selectedShaderIds = $derived(Array.from(comparisonStore.selectedShaders));
	const selectedSceneIds = $derived(Array.from(comparisonStore.selectedScenes));

	const selectedShaders = $derived(
		selectedShaderIds.map((id) => getShaderById(id)).filter(Boolean)
	);
	const selectedScenes = $derived(
		selectedSceneIds.map((id) => getSceneById(id)).filter(Boolean)
	);

	// Active comparison scene (for shader comparison mode)
	let activeSceneId = $state<string | null>(null);
	const activeScene = $derived(activeSceneId ? getSceneById(activeSceneId) : scenes[0]);

	// Get captures for selected shaders in active scene
	const comparisonCaptures = $derived(
		selectedShaders.map((shader) => {
			if (!shader || !activeScene) return null;
			return {
				shader,
				capture: getCapture(shader.id, activeScene.id)
			};
		}).filter(Boolean)
	);
</script>

<div class="container mx-auto px-4 py-8">
	<div class="animate-fade-in-down mb-8">
		<h1 class="mb-2 text-4xl font-bold tracking-tight">Compare</h1>
		<p class="text-lg text-muted-foreground">
			Compare shaders and scenes side-by-side
		</p>
	</div>

	<!-- Selection Summary -->
	<div class="animate-fade-in-up animation-delay-100 mb-8 grid gap-6 md:grid-cols-2">
		<!-- Selected Shaders -->
		<div class="rounded-xl bg-card p-6">
			<div class="mb-4 flex items-center justify-between">
				<h2 class="text-xl font-semibold">Selected Shaders</h2>
				{#if comparisonStore.shaderCount > 0}
					<Button variant="ghost" size="sm" onclick={() => comparisonStore.clearShaders()}>
						Clear all
					</Button>
				{/if}
			</div>
			{#if selectedShaders.length === 0}
				<p class="text-muted-foreground">
					No shaders selected. Go to <a href="/shaders" class="text-primary hover:underline">Shaders</a> and check the ones you want to compare.
				</p>
			{:else}
				<div class="flex flex-wrap gap-3">
					{#each selectedShaders as shader}
						{#if shader}
							<div class="flex items-center gap-2 rounded-lg bg-muted px-3 py-2">
								<img
									src={shader.thumbnail}
									alt={shader.name}
									class="h-8 w-8 rounded object-cover"
								/>
								<span class="text-sm font-medium">{shader.name}</span>
								<button
									onclick={() => comparisonStore.toggleShader(shader.id)}
									class="ml-1 text-muted-foreground hover:text-foreground"
									aria-label="Remove {shader.name} from comparison"
								>
									<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
										<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
									</svg>
								</button>
							</div>
						{/if}
					{/each}
				</div>
			{/if}
		</div>

		<!-- Selected Scenes -->
		<div class="rounded-xl bg-card p-6">
			<div class="mb-4 flex items-center justify-between">
				<h2 class="text-xl font-semibold">Selected Scenes</h2>
				{#if comparisonStore.sceneCount > 0}
					<Button variant="ghost" size="sm" onclick={() => comparisonStore.clearScenes()}>
						Clear all
					</Button>
				{/if}
			</div>
			{#if selectedScenes.length === 0}
				<p class="text-muted-foreground">
					No scenes selected. Go to <a href="/scenes" class="text-primary hover:underline">Scenes</a> and check the ones you want to compare.
				</p>
			{:else}
				<div class="flex flex-wrap gap-3">
					{#each selectedScenes as scene}
						{#if scene}
							<div class="flex items-center gap-2 rounded-lg bg-muted px-3 py-2">
								<img
									src={scene.thumbnail}
									alt={scene.name}
									class="h-8 w-8 rounded object-cover"
								/>
								<span class="text-sm font-medium">{scene.name}</span>
								<button
									onclick={() => comparisonStore.toggleScene(scene.id)}
									class="ml-1 text-muted-foreground hover:text-foreground"
									aria-label="Remove {scene.name} from comparison"
								>
									<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
										<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
									</svg>
								</button>
							</div>
						{/if}
					{/each}
				</div>
			{/if}
		</div>
	</div>

	<!-- Comparison View -->
	{#if selectedShaders.length >= 2}
		<div class="animate-fade-in animation-delay-200">
			<!-- Scene Selector -->
			<div class="mb-6 flex items-center gap-4">
				<span class="text-sm font-medium text-muted-foreground">Compare in scene:</span>
				<select
					bind:value={activeSceneId}
					class="rounded-lg border border-input bg-background px-3 py-2 text-sm focus:ring-2 focus:ring-ring focus:outline-none"
				>
					{#each scenes as scene}
						<option value={scene.id}>{scene.name}</option>
					{/each}
				</select>
			</div>

			<!-- Comparison Grid -->
			<div class="grid gap-6" style="grid-template-columns: repeat({Math.min(selectedShaders.length, 4)}, 1fr)">
				{#each comparisonCaptures as item}
					{#if item}
						<div class="overflow-hidden rounded-xl bg-card">
							<div class="relative aspect-video">
								<img
									src={item.capture?.image ?? item.shader.thumbnail}
									alt="{item.shader.name} in {activeScene?.name}"
									class="h-full w-full object-cover"
								/>
								{#if item.capture}
									<div class="absolute bottom-2 right-2 rounded bg-black/70 px-2 py-1 text-xs font-medium text-white">
										{item.capture.fps} FPS
									</div>
								{/if}
							</div>
							<div class="p-4">
								<h3 class="font-semibold">{item.shader.name}</h3>
								<p class="text-sm text-muted-foreground">{item.shader.author}</p>
								{#if item.capture}
									<div class="mt-2 flex gap-4 text-xs text-muted-foreground">
										<span>Frame: {item.capture.frameTimeMs}ms</span>
										<span>GPU: {item.capture.gpuUsage}%</span>
									</div>
								{/if}
							</div>
						</div>
					{/if}
				{/each}
			</div>
		</div>
	{:else if selectedShaders.length === 1}
		<div class="animate-fade-in animation-delay-200 rounded-xl border border-dashed border-border p-12 text-center">
			<p class="text-muted-foreground">Select at least one more shader to compare</p>
		</div>
	{:else}
		<div class="animate-fade-in animation-delay-200 rounded-xl border border-dashed border-border p-12 text-center">
			<p class="text-muted-foreground">Select shaders from the <a href="/shaders" class="text-primary hover:underline">Shaders page</a> to start comparing</p>
		</div>
	{/if}
</div>
