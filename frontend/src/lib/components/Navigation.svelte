<script lang="ts">
	import { page } from '$app/stores';

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
		<a href="/" class="mr-8 text-xl font-bold">Glint</a>

		<div class="flex gap-6">
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
	</div>
</nav>
