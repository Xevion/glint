<script lang="ts">
import type { User } from '$lib/bindings';
import Breadcrumb from '$lib/components/Breadcrumb.svelte';
import {
	DataView,
	Pagination,
	Search,
	Sort,
	Table,
	Toolbar,
	createClientList,
	filter
} from '$lib/components/data-view';
import type { FilterDescriptor } from '$lib/components/data-view';
import { EnumSelect } from '$lib/components/filters';
import type { PageData } from './$types';
import { columns } from './columns.js';

interface Props {
	data: PageData;
}
let { data }: Props = $props();

// eslint-disable-next-line @typescript-eslint/consistent-type-definitions -- needs implicit index signature for Record<string, FilterDescriptor>
type UserFilters = {
	role: FilterDescriptor<string>;
};

const roleOptions = [
	{ value: 'admin', label: 'Admin' },
	{ value: 'agent', label: 'Agent' },
	{ value: 'user', label: 'User' }
];

const list = createClientList<User, UserFilters>({
	items: () => data.users,
	search: {
		clientFilter: (user, q) =>
			user.discord_username.toLowerCase().includes(q) || user.discord_id.includes(q)
	},
	sort: {
		options: [
			{ value: 'username', label: 'Username' },
			{ value: 'joined', label: 'Joined' }
		],
		default: 'username',
		compare: (sort) => {
			if (sort === 'username') return (a, b) => a.discord_username.localeCompare(b.discord_username);
			if (sort === 'joined')
				return (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
		}
	},
	filters: {
		role: filter<string>()
	},
	applyFilters: (user, filters) => !filters.role || user.role === filters.role,
	syncUrl: true,
	viewMode: 'table'
});

const roleColors: Record<string, string> = {
	admin: 'bg-destructive/15 text-destructive',
	agent: 'bg-info/15 text-info',
	user: 'bg-muted text-muted-foreground'
};
</script>

<svelte:head><title>Users - Glint</title></svelte:head>

<div class="space-y-4">
	<Breadcrumb segments={[{ label: 'Users' }]} />

	<Toolbar>
		<Search bind:value={list.search} placeholder="Search users..." />
		<EnumSelect
			options={roleOptions}
			bind:value={list.filters.role}
			placeholder="All roles"
		/>
		<div class="flex-1"></div>
		<Sort options={list.sortOptions} bind:value={list.sort} />
	</Toolbar>

	<DataView list={list} errorMessage={data.error}>
		{#snippet empty()}
			<p class="py-12 text-center text-foreground">No users found.</p>
		{/snippet}
		<Table {columns} pageSize={25} getRowHref={(user: User) => `/admin/users/${user.id}`}>
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
		</Table>
		<div class="mt-3">
			<Pagination style="pages" />
		</div>
	</DataView>
</div>
