<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import type { User } from '$lib/bindings';
import AdminTable from '$lib/components/AdminTable.svelte';
import { AdminPageHeader } from '$lib/components/admin';
import { Alert } from '$lib/components/ui/alert';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let users = $derived(data.users);
let refreshing = $state(false);

const roleColors: Record<string, string> = {
	admin: 'bg-destructive/15 text-destructive',
	agent: 'bg-info/15 text-info',
	user: 'bg-muted text-muted-foreground'
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
	await Promise.all([invalidateAll(), new Promise((r) => setTimeout(r, 300))]);
	refreshing = false;
}

function handleRowClick(user: User) {
	void goto(`/admin/users/${user.id}`);
}
</script>

<svelte:head><title>Users - Glint</title></svelte:head>

<div class="space-y-4">
	<AdminPageHeader title="Users" count={users.length} {refreshing} onrefresh={refresh} />

	{#if data.error}
		<Alert variant="destructive">Error: {data.error}</Alert>
	{:else if users.length === 0}
		<p class="text-muted-foreground">No users yet.</p>
	{:else}
		<AdminTable
			data={users}
			{columns}
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
						<div
							class="flex h-8 w-8 items-center justify-center rounded-full bg-muted text-xs"
						>
							?
						</div>
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
