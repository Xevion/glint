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

<nav class="nav-header dark:border-b">
	<div class="container mx-auto flex h-16 items-center px-4">
		<a href="/" class="glint-title mr-8 text-xl font-bold">Glint</a>

		<div class="flex flex-1 gap-6">
			{#each navItems as item}
				<a
					href={item.href}
					class="nav-link text-sm font-medium transition-colors"
					class:active={isActive(item.href)}
				>
					{item.label}
				</a>
			{/each}
		</div>

		<button
			onclick={() => themeStore.toggle()}
			class="nav-icon rounded-md p-2 transition-colors"
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

	.nav-header {
		background: oklch(1 0 0 / 76%);
		backdrop-filter: blur(12px);
		-webkit-backdrop-filter: blur(12px);
	}

	:global(.dark) .nav-header {
		background: oklch(0.141 0.005 285.823 / 60%);
	}

	/* Light mode: dark text with white glow for contrast */
	.nav-link {
		color: oklch(0.3 0 0 / 80%);
		text-shadow:
			0 0 8px oklch(1 0 0 / 80%),
			0 0 3px oklch(1 0 0 / 90%);
	}

	.nav-link:hover {
		color: oklch(0.15 0 0);
	}

	.nav-link.active {
		color: oklch(0.1 0 0);
	}

	.nav-icon {
		color: oklch(0.3 0 0);
		filter: drop-shadow(0 0 3px oklch(1 0 0 / 80%));
	}

	.nav-icon:hover {
		color: oklch(0.15 0 0);
		background: oklch(0 0 0 / 8%);
	}

	/* Dark mode: light text, no glow needed */
	:global(.dark) .nav-link {
		color: oklch(1 0 0 / 60%);
		text-shadow: none;
	}

	:global(.dark) .nav-link:hover {
		color: oklch(1 0 0 / 100%);
	}

	:global(.dark) .nav-link.active {
		color: oklch(1 0 0 / 100%);
	}

	:global(.dark) .nav-icon {
		color: oklch(1 0 0 / 60%);
		filter: none;
	}

	:global(.dark) .nav-icon:hover {
		color: oklch(1 0 0 / 100%);
		background: oklch(1 0 0 / 10%);
	}
</style>
