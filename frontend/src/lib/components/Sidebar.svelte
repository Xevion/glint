<script lang="ts">
import { page } from '$app/state';
import { Button } from '$lib/components/ui/button';
import * as Sheet from '$lib/components/ui/sheet';
import { contextTitle, getContext, getContextItems, isActive } from '$lib/config/sidebar';
import { LogIn, LogOut, Menu } from '@lucide/svelte';

const user = $derived(page.data.user ?? null);
const isAdmin = $derived(user?.role === 'admin');

let mobileSheetOpen = $state(false);

const context = $derived(getContext(page.url.pathname));

const loginUrl = $derived(`/login?redirect=${encodeURIComponent(page.url.pathname)}`);

function discordAvatarUrl(discordId: string, avatar: string): string {
	return `https://cdn.discordapp.com/avatars/${discordId}/${avatar}.png?size=64`;
}
</script>

{#snippet navItems(items: ReturnType<typeof getContextItems>, sectionTitle: string)}
	<nav class="flex flex-col gap-0.5">
		<span class="mb-0.5 px-2 text-xs font-medium uppercase tracking-wide text-sidebar-muted-foreground/60">
			{sectionTitle}
		</span>
		{#each items as item (item.href)}
			{@const active = isActive(page.url.pathname, item.href)}
			<a
				href={item.href}
				class="flex items-center gap-2 px-2 py-1 text-sm no-underline transition-colors
					{active
					? 'border-l-2 border-muted-foreground pl-1.5 font-medium text-foreground'
					: 'text-sidebar-muted-foreground hover:bg-muted/50 hover:text-foreground'}"
			>
				<item.icon size={15} strokeWidth={2} />
				{item.label}
			</a>
		{/each}
	</nav>
{/snippet}

{#snippet separator()}
	<div class="mx-2 my-1.5 border-t border-sidebar-muted-foreground/15"></div>
{/snippet}

{#snippet userSection(showSeparator: boolean)}
	<div class="mt-3 flex flex-col gap-0.5">
		{#if showSeparator}
			{@render separator()}
		{/if}
		{#if user}
			<div class="flex items-center gap-2 px-2 py-1">
				{#if user.discord_avatar}
					<img
						src={discordAvatarUrl(user.discord_id, user.discord_avatar)}
						alt={user.discord_username}
						class="h-5 w-5 rounded-full"
					/>
				{:else}
					<div
						class="flex h-5 w-5 items-center justify-center rounded-full bg-muted text-[10px] font-medium text-muted-foreground"
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
					class="flex w-full items-center gap-2 rounded-md px-2 py-1 text-sm text-sidebar-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
				>
					<LogOut size={15} strokeWidth={2} />
					Sign out
				</button>
			</form>
		{:else}
			<a
				href={loginUrl}
				class="flex items-center gap-2 rounded-md px-2 py-1 text-sm text-sidebar-muted-foreground no-underline transition-colors hover:bg-muted/50 hover:text-foreground"
			>
				<LogIn size={15} strokeWidth={2} />
				Sign in
			</a>
		{/if}
	</div>
{/snippet}

{#snippet sidebarNav()}
	{@const hasBrowseNav = context != null && context !== 'admin'}
	{@const hasAdminNav = context === 'admin' || (isAdmin && context !== null)}

	{#if hasBrowseNav}
		{@render navItems(getContextItems(context), contextTitle[context])}
	{/if}

	{@render userSection(hasBrowseNav)}

	{#if hasAdminNav}
		<div class="mt-3 flex flex-col gap-0.5">
			{@render separator()}
			{@render navItems(getContextItems('admin'), 'Admin')}
		</div>
	{/if}
{/snippet}

{#if context !== null}
	<!-- Desktop sidebar -->
	<aside class="hidden w-40 shrink-0 pt-1 md:flex md:flex-col">
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
				<Sheet.Title>{contextTitle[context]}</Sheet.Title>
			</Sheet.Header>
			<div class="mt-4">
				{@render sidebarNav()}
			</div>
		</Sheet.Content>
	</Sheet.Root>
{/if}
