<script lang="ts">
import { onMount } from 'svelte';
import { Button } from '$lib/components/ui/button';
import * as Select from '$lib/components/ui/select';
import { RefreshCw, Trash2 } from '@lucide/svelte';
import AdminTable from '$lib/components/admin-table.svelte';
import { AdminSlideOver, AdminDetailField } from '$lib/components/admin';
import TimeAgo from '$lib/components/time-ago.svelte';
import { api } from '$lib/api';
import type { User, UserWithSessions } from '$lib/bindings';
import { escapeHtml } from '$lib/utils/display';

let users = $state<User[]>([]);
let selected = $state<UserWithSessions | null>(null);
let loading = $state(true);
let refreshing = $state(false);
let error = $state<string | null>(null);

const ROLES = ['user', 'admin', 'agent'] as const;

const columns = [
	{
		id: 'avatar',
		key: 'discord_avatar',
		name: '',
		render: (value: string | null, row: User) => {
			if (value) {
				const avatarUrl = `https://cdn.discordapp.com/avatars/${escapeHtml(row.discord_id)}/${escapeHtml(value)}.png?size=40`;
				return `<img src="${avatarUrl}" alt="Avatar" class="h-8 w-8 rounded-full" />`;
			}
			return '<div class="h-8 w-8 rounded-full bg-muted flex items-center justify-center text-xs">?</div>';
		}
	},
	{
		id: 'username',
		key: 'discord_username',
		name: 'Username',
		render: (value: string) => `<span class="font-medium">${escapeHtml(value)}</span>`
	},
	{
		id: 'discord_id',
		key: 'discord_id',
		name: 'Discord ID',
		render: (value: string) =>
			`<code class="text-xs text-muted-foreground">${escapeHtml(value)}</code>`
	},
	{
		id: 'role',
		key: 'role',
		name: 'Role',
		render: (value: string) => {
			const colorMap: Record<string, string> = {
				admin: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300',
				agent: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300',
				user: 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'
			};
			const colorClass = colorMap[value] || colorMap.user;
			return `<span class="rounded px-2 py-1 text-xs font-medium ${colorClass}">${escapeHtml(value)}</span>`;
		}
	},
	{
		id: 'created_at',
		key: 'created_at',
		name: 'Joined',
		component: 'time' as const
	}
];

async function load() {
	refreshing = true;
	error = null;
	const result = await api.admin.listUsers();
	if (result.isOk) {
		users = result.value;
	} else {
		error = result.error.message;
	}
	loading = false;
	refreshing = false;
}

async function loadUser(id: number) {
	const result = await api.admin.getUser(id);
	if (result.isOk) {
		selected = result.value;
	} else {
		error = result.error.message;
	}
}

async function handleRoleChange(newRole: string) {
	if (!selected) return;
	const result = await api.admin.updateUserRole(selected.id, newRole);
	if (result.isOk) {
		// Update selected user
		selected = { ...selected, ...result.value };
		// Update list
		users = users.map((u) => (u.id === selected!.id ? result.value : u));
	} else {
		error = result.error.message;
	}
}

async function handleDeleteSessions() {
	if (!selected) return;
	if (!confirm(`Delete all sessions for ${selected.discord_username}? They will be logged out.`))
		return;
	const result = await api.admin.deleteUserSessions(selected.id);
	if (result.isOk) {
		// Reload user to get updated session list
		await loadUser(selected.id);
	} else {
		error = result.error.message;
	}
}

function _handleDeleteSession(_token_prefix: string) {
	if (!selected) return;
	// Note: We can't actually delete by prefix, we'd need the full token
	// For now this is a UX limitation - admin can delete all sessions
	error = 'Individual session deletion requires full token. Use "Delete All Sessions" instead.';
}

function handleRowClick(user: User) {
	void loadUser(user.id);
}

onMount(() => {
	void load();
});
</script>

<div class="space-y-4">
	<header class="flex items-center justify-between">
		<div class="flex items-baseline gap-3">
			<h1 class="text-2xl font-semibold">Users</h1>
			{#if !loading}
				<span class="text-lg text-muted-foreground">{users.length}</span>
			{/if}
		</div>
		<Button variant="outline" size="icon" onclick={load} disabled={refreshing}>
			<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
		</Button>
	</header>

	{#if loading}
		<div class="text-center text-muted-foreground">Loading...</div>
	{:else if error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
			Error: {error}
		</div>
	{:else if users.length === 0}
		<p class="text-muted-foreground">No users yet.</p>
	{:else}
		<AdminTable
			data={users}
			{columns}
			selectedId={selected?.id?.toString()}
			onRowClick={handleRowClick}
			getRowId={(u: User) => u.id.toString()}
		/>
	{/if}
</div>

<AdminSlideOver
	open={selected !== null}
	title={selected?.discord_username ?? ''}
	onClose={() => (selected = null)}
	width="wide"
>
	{#if selected}
		<dl class="space-y-4">
			<div class="flex items-center gap-4">
				{#if selected.discord_avatar}
					<img
						src="https://cdn.discordapp.com/avatars/{selected.discord_id}/{selected.discord_avatar}.png?size=80"
						alt="Avatar"
						class="h-16 w-16 rounded-full"
					/>
				{:else}
					<div class="flex h-16 w-16 items-center justify-center rounded-full bg-muted text-2xl">
						?
					</div>
				{/if}
				<div>
					<div class="text-lg font-medium">{selected.discord_username}</div>
					<div class="text-sm text-muted-foreground">{selected.discord_id}</div>
				</div>
			</div>

			<AdminDetailField label="User ID">
				{selected.id}
			</AdminDetailField>

			<AdminDetailField label="Role">
				<Select.Root
					type="single"
					value={selected.role}
					onValueChange={(v: string) => v && handleRoleChange(v)}
				>
					<Select.Trigger class="w-32">
						{selected.role}
					</Select.Trigger>
					<Select.Content>
					{#each ROLES as role (role)}
						<Select.Item value={role}>{role}</Select.Item>
					{/each}
					</Select.Content>
				</Select.Root>
			</AdminDetailField>

			<AdminDetailField label="Joined">
				<TimeAgo timestamp={selected.created_at} />
			</AdminDetailField>

			<AdminDetailField label="Last Updated">
				<TimeAgo timestamp={selected.updated_at} />
			</AdminDetailField>

			<div class="border-t pt-4">
				<div class="mb-2 flex items-center justify-between">
					<h3 class="font-medium">Sessions ({selected.sessions.length})</h3>
					{#if selected.sessions.length > 0}
						<Button variant="destructive" size="sm" onclick={handleDeleteSessions}>
							<Trash2 class="mr-2 h-4 w-4" />
							Delete All
						</Button>
					{/if}
				</div>

				{#if selected.sessions.length === 0}
					<p class="text-sm text-muted-foreground">No active sessions</p>
				{:else}
					<div class="space-y-2">
						{#each selected.sessions as session (session.token_prefix)}
							<div class="flex items-center justify-between rounded border p-2 text-sm">
								<div>
									<code class="text-xs">{session.token_prefix}...</code>
									<div class="text-xs text-muted-foreground">
										Created <TimeAgo timestamp={session.created_at} /> &middot;
										Expires <TimeAgo timestamp={session.expires_at} />
									</div>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</dl>
	{/if}
</AdminSlideOver>
