<script lang="ts">
import { invalidateAll } from '$app/navigation';
import { api } from '$lib/api';
import type { UserWithSessions } from '$lib/bindings';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { AdminBreadcrumb } from '$lib/components/admin';
import { Alert } from '$lib/components/ui/alert';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import * as Select from '$lib/components/ui/select';
import { Trash2 } from '@lucide/svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let user: UserWithSessions = $derived(data.user);

let error = $state<string | null>(null);
let showDeleteSessionsConfirm = $state(false);
let deletingSessionToken = $state<string | null>(null);

const ROLES = ['user', 'admin', 'agent'] as const;

const roleColors: Record<string, string> = {
	admin: 'bg-destructive/15 text-destructive',
	agent: 'bg-info/15 text-info',
	user: 'bg-muted text-muted-foreground'
};

let roleColorClass = $derived(roleColors[user.role] || roleColors.user);

function avatarUrl(size: number): string | null {
	if (!user.discord_avatar) return null;
	return `https://cdn.discordapp.com/avatars/${user.discord_id}/${user.discord_avatar}.png?size=${size}`;
}

async function handleRoleChange(newRole: string) {
	error = null;
	const result = await api.admin.updateUserRole(user.id, newRole);
	result.match({
		Ok: () => void invalidateAll(),
		Err: (err) => {
			error = err.message;
		}
	});
}

async function confirmDeleteAllSessions() {
	error = null;
	const result = await api.admin.deleteUserSessions(user.id);
	result.match({
		Ok: () => void invalidateAll(),
		Err: (err) => {
			error = err.message;
		}
	});
}

async function deleteSession(tokenPrefix: string) {
	error = null;
	deletingSessionToken = tokenPrefix;
	const result = await api.admin.deleteSession(user.id, tokenPrefix);
	result.match({
		Ok: () => void invalidateAll(),
		Err: (err) => {
			error = err.message;
		}
	});
	deletingSessionToken = null;
}
</script>

<svelte:head><title>{user.discord_username} - Users - Glint</title></svelte:head>

<div class="space-y-6">
	<!-- Breadcrumb -->
	<AdminBreadcrumb
		backHref="/admin/users"
		backLabel="Back to users"
		segments={[{ label: 'Users', href: '/admin/users' }, { label: user.discord_username }]}
	>
		{#snippet trailing()}
			<span class="ml-2 rounded px-2 py-0.5 text-xs font-medium {roleColorClass}">
				{user.role}
			</span>
		{/snippet}
	</AdminBreadcrumb>

	<!-- User identity -->
	<div class="flex items-center gap-4">
		{#if avatarUrl(80)}
			<img
				src={avatarUrl(80)}
				alt="{user.discord_username}'s avatar"
				class="h-14 w-14 rounded-full"
			/>
		{:else}
			<div
				class="flex h-14 w-14 items-center justify-center rounded-full bg-muted text-xl font-medium"
			>
				{user.discord_username.charAt(0).toUpperCase()}
			</div>
		{/if}
		<div>
			<h1 class="text-2xl font-semibold">{user.discord_username}</h1>
			<code class="text-xs text-foreground">{user.discord_id}</code>
		</div>
	</div>

	{#if error}
		<Alert variant="destructive">{error}</Alert>
	{/if}

	<!-- User Info -->
	<div class="rounded-lg border bg-card p-4">
		<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-3 text-sm">
			<dt class="text-muted-foreground">User ID</dt>
			<dd>{user.id}</dd>

			<dt class="text-muted-foreground">Discord ID</dt>
			<dd><code class="text-xs">{user.discord_id}</code></dd>

			<dt class="text-muted-foreground">Username</dt>
			<dd class="font-medium">{user.discord_username}</dd>

			<dt class="text-muted-foreground">Role</dt>
			<dd>
				<Select.Root
					type="single"
					value={user.role}
					onValueChange={(v: string) => v && handleRoleChange(v)}
				>
					<Select.Trigger class="w-32">
						{user.role}
					</Select.Trigger>
					<Select.Content>
						{#each ROLES as role (role)}
							<Select.Item value={role}>{role}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</dd>

			<dt class="text-muted-foreground">Joined</dt>
			<dd><TimeAgo timestamp={user.created_at} /></dd>

			<dt class="text-muted-foreground">Last updated</dt>
			<dd><TimeAgo timestamp={user.updated_at} /></dd>
		</dl>
	</div>

	<!-- Sessions -->
	<div class="rounded-lg border bg-card p-4">
		<div class="mb-3 flex items-center justify-between">
			<h3 class="text-xs font-medium uppercase tracking-wider text-muted-foreground">
				Sessions ({user.sessions.length})
			</h3>
			{#if user.sessions.length > 0}
				<Button
					variant="destructive"
					size="sm"
					onclick={() => (showDeleteSessionsConfirm = true)}
				>
					<Trash2 class="mr-1.5 h-3.5 w-3.5" />
					Delete All
				</Button>
			{/if}
		</div>

		{#if user.sessions.length === 0}
			<p class="text-sm text-muted-foreground">No active sessions</p>
		{:else}
			<div class="space-y-2">
				{#each user.sessions as session (session.token_prefix)}
					<div class="flex items-center justify-between rounded border bg-card p-3 text-sm">
						<div>
							<code class="text-xs">{session.token_prefix}</code>
							<div class="mt-0.5 text-xs text-muted-foreground">
								Created <TimeAgo timestamp={session.created_at} /> · Expires <TimeAgo
									timestamp={session.expires_at}
								/>
							</div>
						</div>
						<Button
							variant="ghost"
							size="icon"
							class="h-8 w-8 text-muted-foreground hover:text-destructive"
							onclick={() => deleteSession(session.token_prefix)}
							disabled={deletingSessionToken === session.token_prefix}
							aria-label="Delete session"
						>
							<Trash2 class="h-4 w-4" />
						</Button>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>

<ConfirmDialog
	bind:open={showDeleteSessionsConfirm}
	title="Delete All Sessions"
	description="Delete all sessions for {user.discord_username}? They will be logged out everywhere."
	confirmLabel="Delete All"
	onConfirm={confirmDeleteAllSessions}
/>
