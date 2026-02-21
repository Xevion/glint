import {
	type ColumnDef,
	renderComponent,
	textColumn,
	timeColumn
} from '$lib/components/data-table';
import type { AdminScene } from './+page';
import ActiveCell from './active-cell.svelte';
import DimensionCell from './dimension-cell.svelte';
import PreviewCell from './preview-cell.svelte';

export const columns: ColumnDef<AdminScene>[] = [
	{
		accessorKey: 'imagePath',
		header: 'Preview',
		size: 96,
		enableSorting: false,
		cell: ({ row }) => renderComponent(PreviewCell, { scene: row.original })
	},
	textColumn<AdminScene>('name', 'Name', { size: 200 }),
	textColumn<AdminScene>('slug', 'Slug', { sortable: false }),
	{
		accessorKey: 'dimension',
		header: 'Dimension',
		enableSorting: false,
		cell: ({ row }) => renderComponent(DimensionCell, { dimension: row.original.dimension })
	},
	textColumn<AdminScene>('captureCount', 'Captures'),
	{
		accessorKey: 'active',
		header: 'Status',
		enableSorting: false,
		cell: ({ row }) => renderComponent(ActiveCell, { active: row.original.active })
	},
	timeColumn<AdminScene>('createdAt', 'Created')
];
