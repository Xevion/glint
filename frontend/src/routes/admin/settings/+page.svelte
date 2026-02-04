<script lang="ts">
import { fly } from 'svelte/transition';
</script>

<div class="container mx-auto px-4 py-8">
	<div in:fly={{ y: -10, duration: 400 }} class="mb-8">
		<h1 class="mb-2 text-3xl font-bold">Settings</h1>
		<p class="text-muted-foreground">Configure application settings and preferences</p>
	</div>

	<div
		in:fly={{ y: 10, duration: 400, delay: 100 }}
		class="rounded-xl border border-border bg-card p-6"
	>
		<p class="text-sm text-muted-foreground">Settings interface coming soon in alpha development</p>
	</div>
</div>
