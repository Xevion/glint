<script lang="ts">
	import { page } from '$app/stores';
	import { Button } from '$lib/components/ui/button';

	const shaderId = $page.params.id;
</script>

<div class="container mx-auto px-4 py-8">
	<div class="mb-8">
		<h1 class="mb-2 text-4xl font-bold">Shader Pack {shaderId}</h1>
		<p class="text-muted-foreground">Detailed view with all scenes and performance metrics</p>
	</div>

	<div class="mb-8 grid gap-6 lg:grid-cols-3">
		<div class="lg:col-span-2">
			<div class="aspect-video rounded-lg bg-muted"></div>
		</div>
		<div class="rounded-lg border p-6">
			<h2 class="mb-4 text-xl font-semibold">Metadata</h2>
			<dl class="space-y-2 text-sm">
				<div>
					<dt class="text-muted-foreground">Version</dt>
					<dd class="font-medium">1.0.0</dd>
				</div>
				<div>
					<dt class="text-muted-foreground">Minecraft</dt>
					<dd class="font-medium">1.20.4</dd>
				</div>
				<div>
					<dt class="text-muted-foreground">Avg FPS</dt>
					<dd class="font-medium">60</dd>
				</div>
			</dl>
			<div class="mt-6">
				<Button class="w-full" href="https://modrinth.com" target="_blank">View on Modrinth</Button>
			</div>
		</div>
	</div>

	<div>
		<h2 class="mb-4 text-2xl font-semibold">All Scenes</h2>
		<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
			{#each ['Forest', 'Village', 'Cave', 'Nether', 'End', 'Ocean'] as scene}
				<div class="rounded-lg border p-4">
					<div class="mb-3 aspect-video rounded bg-muted"></div>
					<h3 class="font-medium">{scene}</h3>
					<p class="text-sm text-muted-foreground">Sunset, Clear Weather</p>
				</div>
			{/each}
		</div>
	</div>
</div>
