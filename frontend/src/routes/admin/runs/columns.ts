import {
	type ColumnDef,
	DataTableColumnHeader,
	renderComponent,
	textColumn,
	timeColumn
} from '$lib/components/data-table';
import DurationCell from './duration-cell.svelte';
import ProgressCell from './progress-cell.svelte';
import type { RunNode } from './queries';
import StatusCell from './status-cell.svelte';

export const columns: ColumnDef<RunNode>[] = [
	textColumn<RunNode>('id', 'Id', { size: 80, sortable: false }),
	{
		accessorKey: 'status',
		size: 100,
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Status' }),
		cell: ({ row }) => renderComponent(StatusCell, { status: row.original.status })
	},
	textColumn<RunNode>('totalItems', 'Captures', { size: 90 }),
	{
		accessorKey: 'completedItems',
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Progress' }),
		cell: ({ row }) =>
			renderComponent(ProgressCell, {
				total: row.original.totalItems,
				completed: row.original.completedItems,
				failed: row.original.failedItems,
				skipped: row.original.skippedItems,
				status: row.original.status
			}),
		enableSorting: false
	},
	{
		accessorKey: 'durationSecs',
		id: 'duration',
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Duration' }),
		cell: ({ row }) =>
			renderComponent(DurationCell, {
				durationSecs: row.original.durationSecs
			})
	},
	timeColumn<RunNode>('startedAt', 'Started'),
	{
		accessorKey: 'resolutionWidth',
		id: 'resolution',
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Resolution' }),
		cell: ({ row }) => {
			const { resolutionWidth, resolutionHeight } = row.original;
			return `${resolutionWidth}×${resolutionHeight}`;
		},
		enableSorting: false
	}
];
