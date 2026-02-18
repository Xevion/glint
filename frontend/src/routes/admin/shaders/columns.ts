import type { ShaderListItem } from '$lib/bindings';
import {
	type ColumnDef,
	DataTableColumnHeader,
	imageColumn,
	renderComponent,
	timeColumn
} from '$lib/components/data-table';
import { Sparkles } from '@lucide/svelte';
import DescriptionCell from './description-cell.svelte';
import ExtractionCell from './extraction-cell.svelte';
import ShaderNameCell from './shader-name-cell.svelte';
import SyncStatusCell from './sync-status-cell.svelte';

export const columns: ColumnDef<ShaderListItem>[] = [
	imageColumn<ShaderListItem>('icon_url', { fallback: Sparkles }),
	{
		accessorKey: 'name',
		size: 200,
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Name' }),
		cell: ({ row }) => renderComponent(ShaderNameCell, { shader: row.original })
	},
	{
		accessorKey: 'description',
		minSize: 150,
		header: 'Description',
		cell: ({ row }) => renderComponent(DescriptionCell, { description: row.original.description }),
		enableSorting: false
	},
	{
		accessorKey: 'last_synced_at',
		size: 100,
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Sync' }),
		cell: ({ row }) => renderComponent(SyncStatusCell, { shader: row.original })
	},
	{
		accessorKey: 'version_count',
		size: 180,
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Versions' }),
		cell: ({ row }) =>
			renderComponent(ExtractionCell, {
				versionCount: row.original.version_count,
				summary: row.original.extraction_summary
			})
	},
	timeColumn<ShaderListItem>('created_at', 'Created')
];
