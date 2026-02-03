<script lang="ts">
import { page } from '$app/stores';
import { resolve } from '$app/paths';
import { Sun, Moon, Settings } from '@lucide/svelte';
import { themeStore } from '$lib/stores/theme.svelte';

const navItems = [
	{ href: '/' as const, label: 'Home' },
	{ href: '/shaders' as const, label: 'Shaders' },
	{ href: '/scenes' as const, label: 'Scenes' },
	{ href: '/compare' as const, label: 'Compare' }
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
		<a href={resolve('/', {})} class="glint-title mr-8 text-xl font-bold">Glint</a>

		<div class="flex flex-1 gap-6">
			{#each navItems as item (item.href)}
				<a
					href={resolve(item.href, {})}
					class="nav-link text-sm font-medium transition-colors"
					class:active={isActive(item.href)}
				>
					{item.label}
				</a>
			{/each}
		</div>

		<div class="flex gap-2">
			<a
				href={resolve('/admin', {})}
				class="nav-icon rounded-md p-2 transition-colors"
				aria-label="Admin panel"
			>
				<Settings class="size-5" />
			</a>
			<button
				onclick={() => {
					themeStore.toggle();
				}}
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
	</div>
</nav>

<style>
	:root {
		--glint-color-1: oklch(0.65 0.25 300);
		--glint-color-2: oklch(0.75 0.2 320);
	}

	.glint-title {
		position: relative;
		color: var(--foreground);
		background: linear-gradient(
			110deg,
			var(--foreground) 0%,
			var(--foreground) 92%,
			var(--glint-color-1) 94%,
			var(--glint-color-2) 96%,
			var(--glint-color-1) 98%,
			var(--foreground) 100%
		);
		background-size: 900% 100%;
		-webkit-background-clip: text;
		background-clip: text;
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
		-webkit-backdrop-filter: blur(12px);
		backdrop-filter: blur(12px);
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
