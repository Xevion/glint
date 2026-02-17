<script lang="ts">
import { page } from '$app/state';
import { Button } from '$lib/components/ui/button';
import * as Sheet from '$lib/components/ui/sheet';
import {
	Activity,
	BookOpen,
	Camera,
	Cloud,
	Globe,
	HardDrive,
	Layers,
	LayoutDashboard,
	LogIn,
	LogOut,
	Menu,
	Mountain,
	Palette,
	Settings,
	Sparkles,
	Sun,
	TreePine,
	Users,
	Zap
} from '@lucide/svelte';

const user = $derived(page.data.user ?? null);
const isAdmin = $derived(user?.role === 'admin');

let mobileSheetOpen = $state(false);

type SidebarContext = 'home' | 'shaders' | 'scenes' | 'admin' | null;

const context: SidebarContext = $derived.by(() => {
	const path = page.url.pathname;
	if (path.startsWith('/admin')) return 'admin';
	if (path.startsWith('/compare')) return null;
	if (path.startsWith('/shaders')) return 'shaders';
	if (path.startsWith('/scenes')) return 'scenes';
	if (path === '/') return 'home';
	return null;
});

const contextTitle: Record<Exclude<SidebarContext, null>, string> = {
	home: 'Quick Links',
	shaders: 'Filter by Style',
	scenes: 'Filter Scenes',
	admin: 'Admin'
};

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
	{ href: '/admin/runs', label: 'Runs', icon: Activity },
	{ href: '/admin/storage', label: 'Storage', icon: HardDrive },
	{ href: '/admin/users', label: 'Users', icon: Users },
	{ href: '/admin/settings', label: 'Settings', icon: Settings }
];

function isActive(href: string): boolean {
	if (href === '/') return page.url.pathname === '/';
	if (href === '/admin') return page.url.pathname === '/admin';
	if (href.includes('?')) return false;
	return page.url.pathname.startsWith(href);
}

function getContextItems(ctx: SidebarContext) {
	switch (ctx) {
		case 'home':
			return homeItems;
		case 'shaders':
			return shaderFilters;
		case 'scenes':
			return sceneFilters;
		case 'admin':
			return adminItems;
		default:
			return [];
	}
}

const loginUrl = $derived(`/login?redirect=${encodeURIComponent(page.url.pathname)}`);

function discordAvatarUrl(discordId: string, avatar: string): string {
	return `https://cdn.discordapp.com/avatars/${discordId}/${avatar}.png?size=64`;
}
</script>

{#snippet userSection()}
	<div class="flex flex-col gap-0.5 mt-4">
		<div class="mx-2 mb-2 border-t border-border"></div>
		{#if user}
			<div class="flex items-center gap-2 px-2 py-1.5">
				{#if user.discord_avatar}
					<img
						src={discordAvatarUrl(user.discord_id, user.discord_avatar)}
						alt={user.discord_username}
						class="h-6 w-6 rounded-full"
					/>
				{:else}
					<div
						class="flex h-6 w-6 items-center justify-center rounded-full bg-muted text-xs font-medium text-muted-foreground"
					>
						{user.discord_username[0]?.toUpperCase()}
					</div>
				{/if}
				<span class="truncate text-sm text-sidebar-muted-foreground">
					{user.discord_username}
				</span>
			</div>
			<form method="POST" action="/api/auth/logout">
				<button
					type="submit"
					class="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm text-sidebar-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
				>
					<LogOut size={15} strokeWidth={2} />
					Sign out
				</button>
			</form>
		{:else}
			<a
				href={loginUrl}
				class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm no-underline transition-colors text-sidebar-muted-foreground hover:text-foreground hover:bg-muted/50"
			>
				<LogIn size={15} strokeWidth={2} />
				Sign in
			</a>
		{/if}
	</div>
{/snippet}

{#snippet sidebarNav()}
	{#if context && context !== 'admin'}
		<nav class="flex flex-col gap-0.5">
			<span
				class="px-2 text-[11px] font-medium uppercase tracking-wider text-sidebar-muted-foreground/60 mb-0.5"
			>
				{contextTitle[context]}
			</span>
			{#each getContextItems(context) as item (item.href)}
				<a
					href={item.href}
					class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm no-underline transition-colors text-sidebar-muted-foreground hover:text-foreground hover:bg-muted/50"
				>
					<item.icon size={15} strokeWidth={2} />
					{item.label}
				</a>
			{/each}
		</nav>
	{/if}

	{@render userSection()}

	<!-- Admin section (on admin pages, or for non-admin contexts when user is admin) -->
	{#if context === 'admin' || (isAdmin && context !== null)}
		<nav class="flex flex-col gap-0.5 mt-4">
			<div class="mx-2 mb-2 border-t border-border"></div>
			<span
				class="px-2 text-[11px] font-medium uppercase tracking-wider text-sidebar-muted-foreground/60 mb-0.5"
			>
				Admin
			</span>
			{#each adminItems as item (item.href)}
				<a
					href={item.href}
					class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm no-underline transition-colors
						{isActive(item.href)
						? 'text-foreground bg-muted font-medium'
						: 'text-sidebar-muted-foreground hover:text-foreground hover:bg-muted/50'}"
				>
					<item.icon size={15} strokeWidth={2} />
					{item.label}
				</a>
			{/each}
		</nav>
	{/if}
{/snippet}

{#if context !== null}
	<!-- Desktop sidebar -->
	<aside class="hidden md:flex md:flex-col w-48 shrink-0 pt-1">
		{@render sidebarNav()}
	</aside>

	<!-- Mobile FAB trigger -->
	<div class="fixed bottom-4 right-4 z-40 md:hidden">
		<Button
			variant="default"
			size="icon"
			class="h-12 w-12 rounded-full shadow-theme-lg"
			onclick={() => (mobileSheetOpen = true)}
		>
			<Menu size={20} />
			<span class="sr-only">Open navigation</span>
		</Button>
	</div>

	<!-- Mobile bottom sheet -->
	<Sheet.Root bind:open={mobileSheetOpen}>
		<Sheet.Content side="bottom" class="max-h-[60vh] overflow-y-auto px-4 pb-8 pt-6">
			<Sheet.Header class="text-left">
				<Sheet.Title>{context ? contextTitle[context] : 'Navigation'}</Sheet.Title>
			</Sheet.Header>
			<div class="mt-4">
				{@render sidebarNav()}
			</div>
		</Sheet.Content>
	</Sheet.Root>
{/if}
