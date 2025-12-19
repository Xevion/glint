<script lang="ts">
	import { page } from '$app/stores';
	import { themeStore } from '$lib/stores/theme.svelte';
	import { Sun, Moon } from '@lucide/svelte';

	const navItems = [
		{ href: '/', label: 'Home' },
		{ href: '/shaders', label: 'Shaders' },
		{ href: '/scenes', label: 'Scenes' },
		{ href: '/compare', label: 'Compare' }
	];

	function isActive(href: string): boolean {
		if (href === '/') {
			return $page.url.pathname === '/';
		}
		return $page.url.pathname.startsWith(href);
	}
</script>

<nav class="border-b">
	<div class="container mx-auto flex h-16 items-center px-4">
		<a href="/" class="glint-title mr-8 text-xl font-bold">Glint</a>

		<div class="flex flex-1 gap-6">
			{#each navItems as item}
				<a
					href={item.href}
					class="text-sm font-medium transition-colors hover:text-primary {isActive(item.href)
						? 'text-foreground'
						: 'text-muted-foreground'}"
				>
					{item.label}
				</a>
			{/each}
		</div>

		<button
			onclick={() => themeStore.toggle()}
			class="rounded-md p-2 text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
			aria-label="Toggle theme"
		>
			{#if themeStore.isDark}
				<Sun class="size-5" />
			{:else}
				<Moon class="size-5" />
			{/if}
		</button>
	</div>
</nav>

<style>
	.glint-title {
		position: relative;
		color: var(--foreground);
		background: linear-gradient(
			110deg,
			var(--foreground) 0%,
			var(--foreground) 92%,
			oklch(0.65 0.25 300) 94%,
			oklch(0.75 0.2 320) 96%,
			oklch(0.65 0.25 300) 98%,
			var(--foreground) 100%
		);
		background-size: 900% 100%;
		background-clip: text;
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		animation: glint-sweep 9s linear infinite;
	}

	@keyframes glint-sweep {
		0% {
			background-position: 117% 0;
		}
		15% {
			background-position: 90% 0;
		}
	}
</style>
