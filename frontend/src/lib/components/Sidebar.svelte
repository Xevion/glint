<script lang="ts">
import { page } from '$app/stores';
import {
	Sparkles,
	Mountain,
	BookOpen,
	Layers,
	Palette,
	Zap,
	Sun,
	Cloud,
	TreePine,
	LayoutDashboard,
	Settings,
	Globe,
	Camera,
	Briefcase,
	Users
} from '@lucide/svelte';

// Placeholder: would come from auth context
const isAdmin = true;

type SidebarContext = 'home' | 'shaders' | 'scenes' | 'admin' | null;

const context: SidebarContext = $derived.by(() => {
	const path = $page.url.pathname;
	if (path.startsWith('/admin')) return 'admin';
	if (path.startsWith('/compare')) return null;
	if (path.startsWith('/shaders')) return 'shaders';
	if (path.startsWith('/scenes')) return 'scenes';
	if (path === '/') return 'home';
	return null;
});

// Sidebar configurations for each context
const homeItems = [
	{ href: '/shaders', label: 'Popular Shaders', icon: Sparkles },
	{ href: '/scenes', label: 'Browse Scenes', icon: Mountain },
	{ href: '#', label: 'Getting Started', icon: BookOpen }
];

const shaderFilters = [
	{ href: '/shaders?style=realistic', label: 'Realistic', icon: Layers },
	{ href: '/shaders?style=fantasy', label: 'Fantasy', icon: Palette },
	{ href: '/shaders?style=performance', label: 'Performance', icon: Zap }
];

const sceneFilters = [
	{ href: '/scenes?lighting=day', label: 'Daytime', icon: Sun },
	{ href: '/scenes?lighting=weather', label: 'Weather', icon: Cloud },
	{ href: '/scenes?biome=forest', label: 'Forest', icon: TreePine }
];

const adminItems = [
	{ href: '/admin', label: 'Dashboard', icon: LayoutDashboard },
	{ href: '/admin/shaders', label: 'Shaders', icon: Sparkles },
	{ href: '/admin/worlds', label: 'Worlds', icon: Globe },
	{ href: '/admin/scenes', label: 'Scenes', icon: Mountain },
	{ href: '/admin/captures', label: 'Captures', icon: Camera },
	{ href: '/admin/jobs', label: 'Jobs', icon: Briefcase },
	{ href: '/admin/users', label: 'Users', icon: Users },
	{ href: '/admin/settings', label: 'Settings', icon: Settings }
];

function isActive(href: string): boolean {
	if (href === '/') return $page.url.pathname === '/';
	if (href === '/admin') return $page.url.pathname === '/admin';
	if (href.includes('?')) return false;
	return $page.url.pathname.startsWith(href);
}
</script>

{#if context !== null}
	<aside class="hidden md:block w-48 shrink-0 pt-1">
		<nav class="flex flex-col gap-0.5">
			{#if context === 'home'}
				<span
					class="px-2 text-[11px] font-medium uppercase tracking-wider text-muted-foreground/60 mb-0.5"
				>
					Quick Links
				</span>
				{#each homeItems as item (item.href)}
					<a
						href={item.href}
						class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm no-underline transition-colors text-muted-foreground hover:text-foreground hover:bg-muted/50"
					>
						<item.icon size={15} strokeWidth={2} />
						{item.label}
					</a>
				{/each}
			{:else if context === 'shaders'}
				<span
					class="px-2 text-[11px] font-medium uppercase tracking-wider text-muted-foreground/60 mb-0.5"
				>
					Filter by Style
				</span>
				{#each shaderFilters as item (item.href)}
					<a
						href={item.href}
						class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm no-underline transition-colors text-muted-foreground hover:text-foreground hover:bg-muted/50"
					>
						<item.icon size={15} strokeWidth={2} />
						{item.label}
					</a>
				{/each}
			{:else if context === 'scenes'}
				<span
					class="px-2 text-[11px] font-medium uppercase tracking-wider text-muted-foreground/60 mb-0.5"
				>
					Filter Scenes
				</span>
				{#each sceneFilters as item (item.href)}
					<a
						href={item.href}
						class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm no-underline transition-colors text-muted-foreground hover:text-foreground hover:bg-muted/50"
					>
						<item.icon size={15} strokeWidth={2} />
						{item.label}
					</a>
				{/each}
			{:else if context === 'admin'}
				<span
					class="px-2 text-[11px] font-medium uppercase tracking-wider text-muted-foreground/60 mb-0.5"
				>
					Admin
				</span>
				{#each adminItems as item (item.href)}
					<a
						href={item.href}
						class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm no-underline transition-colors
							{isActive(item.href)
							? 'text-foreground bg-muted font-medium'
							: 'text-muted-foreground hover:text-foreground hover:bg-muted/50'}"
					>
						<item.icon size={15} strokeWidth={2} />
						{item.label}
					</a>
				{/each}
			{/if}
		</nav>

		<!-- Admin section (for non-admin contexts, when user is admin) -->
		{#if isAdmin && context !== 'admin'}
			<nav class="flex flex-col gap-0.5 mt-4">
				<div class="mx-2 mb-2 border-t border-border"></div>
				<span
					class="px-2 text-[11px] font-medium uppercase tracking-wider text-muted-foreground/60 mb-0.5"
				>
					Admin
				</span>
				{#each adminItems as item (item.href)}
					<a
						href={item.href}
						class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm no-underline transition-colors
							{isActive(item.href)
							? 'text-foreground bg-muted font-medium'
							: 'text-muted-foreground hover:text-foreground hover:bg-muted/50'}"
					>
						<item.icon size={15} strokeWidth={2} />
						{item.label}
					</a>
				{/each}
			</nav>
		{/if}
	</aside>
{/if}
