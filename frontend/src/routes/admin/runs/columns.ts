import type { CaptureRun } from '$lib/bindings';
import {
	type ColumnDef,
	DataTableColumnHeader,
	renderComponent,
	textColumn,
	timeColumn
} from '$lib/components/data-table';
import DurationCell from './duration-cell.svelte';
import ProgressCell from './progress-cell.svelte';
import StatusCell from './status-cell.svelte';

export const columns: ColumnDef<CaptureRun>[] = [
	textColumn<CaptureRun>('id', 'Id', { size: 80, sortable: false }),
	{
		accessorKey: 'status',
		size: 100,
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Status' }),
		cell: ({ row }) => renderComponent(StatusCell, { status: row.original.status })
	},
	textColumn<CaptureRun>('total_items', 'Captures', { size: 90 }),
	{
		accessorKey: 'completed_items',
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Progress' }),
		cell: ({ row }) =>
			renderComponent(ProgressCell, {
				total: row.original.total_items,
				completed: row.original.completed_items,
				failed: row.original.failed_items,
				skipped: row.original.skipped_items,
				status: row.original.status
			}),
		enableSorting: false
	},
	{
		accessorKey: 'started_at',
		id: 'duration',
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Duration' }),
		cell: ({ row }) =>
			renderComponent(DurationCell, {
				started_at: row.original.started_at,
				completed_at: row.original.completed_at
			})
	},
	timeColumn<CaptureRun>('started_at', 'Started')
];
