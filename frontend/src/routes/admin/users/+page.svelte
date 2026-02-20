<script lang="ts">
import type { User } from '$lib/bindings';
import { AdminBreadcrumb } from '$lib/components/admin';
import { DataTable, DataTablePagination, createDataTable } from '$lib/components/data-table';
import { DataList } from '$lib/components/data-list';
import type { PageData } from './$types';
import { columns } from './columns.js';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let users = $derived(data.users);
const table = createDataTable<User>({
	get data() {
		return users;
	},
	columns,
	pageSize: 25,
	selection: false
});

const roleColors: Record<string, string> = {
	admin: 'bg-destructive/15 text-destructive',
	agent: 'bg-info/15 text-info',
	user: 'bg-muted text-muted-foreground'
};
</script>

<svelte:head><title>Users - Glint</title></svelte:head>

<div class="space-y-4">
	<AdminBreadcrumb segments={[{ label: 'Users' }]} />

	<DataList items={users} error={data.error} emptyMessage="No users yet.">
		{#snippet content()}
			<DataTable {table} getRowHref={(user: User) => `/admin/users/${user.id}`}>
				{#snippet card(user: User)}
					<div class="flex items-center gap-3">
						{#if user.discord_avatar}
							<img
								src="https://cdn.discordapp.com/avatars/{user.discord_id}/{user.discord_avatar}.png?size=40"
								alt="Avatar"
								class="h-10 w-10 shrink-0 rounded-full"
							/>
						{:else}
							<div
								class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-muted text-sm"
							>
								?
							</div>
						{/if}
						<div class="min-w-0 flex-1">
							<div class="font-medium">{user.discord_username}</div>
							<code class="text-xs text-muted-foreground">{user.discord_id}</code>
						</div>
					<span
							class="shrink-0 rounded px-2 py-1 text-xs font-medium {roleColors[user.role] || roleColors.user}"
						>
							{user.role}
						</span>
					</div>
				{/snippet}
			</DataTable>
			<div class="mt-3">
				<DataTablePagination {table} />
			</div>
		{/snippet}
	</DataList>
</div>
