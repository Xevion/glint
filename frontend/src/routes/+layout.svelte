<script lang="ts">
	import Navigation from '$lib/components/Navigation.svelte';
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="flex min-h-screen flex-col">
	<Navigation />
	<main class="flex-1">
		{@render children()}
	</main>
</div>
