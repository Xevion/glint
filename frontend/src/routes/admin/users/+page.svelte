<script lang="ts">
import { invalidateAll } from '$app/navigation';
import { api } from '$lib/api';
import type { User, UserWithSessions } from '$lib/bindings';
import AdminTable from '$lib/components/AdminTable.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { AdminDetailField, AdminSlideOver } from '$lib/components/admin';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import * as Select from '$lib/components/ui/select';
import { RefreshCw, Trash2 } from '@lucide/svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let users = $derived(data.users);
let selected = $state<UserWithSessions | null>(null);
let refreshing = $state(false);
let error = $state<string | null>(null);
let showDeleteSessionsConfirm = $state(false);

const ROLES = ['user', 'admin', 'agent'] as const;

const roleColors: Record<string, string> = {
	admin: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300',
	agent: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300',
	user: 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'
};

const columns = [
	{ id: 'avatar', key: 'discord_avatar', name: '' },
	{ id: 'username', key: 'discord_username', name: 'Username' },
	{ id: 'discord_id', key: 'discord_id', name: 'Discord ID' },
	{ id: 'role', key: 'role', name: 'Role' },
	{ id: 'created_at', key: 'created_at', name: 'Joined', component: 'time' as const }
];

async function refresh() {
	refreshing = true;
	error = null;
	await invalidateAll();
	refreshing = false;
}

async function loadUser(id: number) {
	const result = await api.admin.getUser(id);
	result.match({
		Ok: (user) => {
			selected = user;
		},
		Err: (err) => {
			error = err.message;
		}
	});
}

async function handleRoleChange(newRole: string) {
	if (!selected) return;
	const result = await api.admin.updateUserRole(selected.id, newRole);
	result.match({
		Ok: (value) => {
			selected = { ...selected!, ...value };
			void refresh();
		},
		Err: (err) => {
			error = err.message;
		}
	});
}

function handleDeleteSessions() {
	if (!selected) return;
	showDeleteSessionsConfirm = true;
}

async function confirmDeleteSessions() {
	if (!selected) return;
	const result = await api.admin.deleteUserSessions(selected.id);
	result.match({
		Ok: () => void loadUser(selected!.id),
		Err: (err) => {
			error = err.message;
		}
	});
}

function handleRowClick(user: User) {
	void loadUser(user.id);
}
</script>

<div class="space-y-4">
	<header class="flex items-center justify-between">
		<div class="flex items-baseline gap-3">
			<h1 class="text-2xl font-semibold">Users</h1>
			<span class="text-lg text-muted-foreground">{users.length}</span>
		</div>
		<Button variant="outline" size="icon" onclick={refresh} disabled={refreshing}>
			<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
		</Button>
	</header>

	{#if error}
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
		>
			{#snippet cell({ columnId, value, row }: { columnId: string; value: unknown; row: User })}
				{#if columnId === 'avatar'}
					{#if value}
						<img
							src="https://cdn.discordapp.com/avatars/{row.discord_id}/{value}.png?size=40"
							alt="Avatar"
							class="h-8 w-8 rounded-full"
						/>
					{:else}
						<div class="flex h-8 w-8 items-center justify-center rounded-full bg-muted text-xs">?</div>
					{/if}
				{:else if columnId === 'username'}
					<span class="font-medium">{value}</span>
				{:else if columnId === 'discord_id'}
					<code class="text-xs text-muted-foreground">{value}</code>
				{:else if columnId === 'role'}
					{@const colorClass = roleColors[String(value)] || roleColors.user}
					<span class="rounded px-2 py-1 text-xs font-medium {colorClass}">{value}</span>
				{:else}
					{value ?? '-'}
				{/if}
			{/snippet}
		</AdminTable>
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

<ConfirmDialog
	bind:open={showDeleteSessionsConfirm}
	title="Delete Sessions"
	description={`Delete all sessions for ${selected?.discord_username}? They will be logged out.`}
	confirmLabel="Delete All"
	onConfirm={confirmDeleteSessions}
/>
