import type { User } from '$lib/bindings';
import {
	type ColumnDef,
	DataTableColumnHeader,
	renderComponent,
	timeColumn
} from '$lib/components/data-table';
import AvatarCell from './avatar-cell.svelte';
import DiscordIdCell from './discord-id-cell.svelte';
import RoleCell from './role-cell.svelte';
import UsernameCell from './username-cell.svelte';

export const columns: ColumnDef<User>[] = [
	{
		accessorKey: 'discord_avatar',
		header: '',
		size: 48,
		minSize: 48,
		enableSorting: false,
		cell: ({ row }) => renderComponent(AvatarCell, { user: row.original })
	},
	{
		accessorKey: 'discord_username',
		size: 200,
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Username' }),
		cell: ({ row }) => renderComponent(UsernameCell, { username: row.original.discord_username })
	},
	{
		accessorKey: 'discord_id',
		size: 180,
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Discord ID' }),
		cell: ({ row }) => renderComponent(DiscordIdCell, { discordId: row.original.discord_id }),
		enableSorting: false
	},
	{
		accessorKey: 'role',
		size: 100,
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Role' }),
		cell: ({ row }) => renderComponent(RoleCell, { role: row.original.role })
	},
	timeColumn<User>('created_at', 'Joined')
];
