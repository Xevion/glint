import {
	type ColumnDef,
	DataTableColumnHeader,
	renderComponent,
	textColumn,
	timeColumn
} from '$lib/components/data-table';
import FreshnessCell from './freshness-cell.svelte';
import PreviewCell from './preview-cell.svelte';
import type { AdminCaptureNode } from './queries';
import ResolutionCell from './resolution-cell.svelte';
import RunCell from './run-cell.svelte';
import ShaderCell from './shader-cell.svelte';
import SizeCell from './size-cell.svelte';

export const columns: ColumnDef<AdminCaptureNode>[] = [
	{
		accessorKey: 'imagePath',
		header: 'Preview',
		size: 96,
		enableSorting: false,
		cell: ({ row }) => renderComponent(PreviewCell, { capture: row.original })
	},
	{
		accessorKey: 'shaderName',
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Shader' }),
		cell: ({ row }) => renderComponent(ShaderCell, { capture: row.original })
	},
	textColumn<AdminCaptureNode>('sceneName', 'Scene', { sortable: false }),
	textColumn<AdminCaptureNode>('profileDisplayName', 'Profile', { sortable: false }),
	{
		accessorKey: 'resolutionWidth',
		header: 'Resolution',
		enableSorting: false,
		cell: ({ row }) => renderComponent(ResolutionCell, { capture: row.original })
	},
	{
		accessorKey: 'fileSizeBytes',
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Size' }),
		cell: ({ row }) => renderComponent(SizeCell, { capture: row.original })
	},
	timeColumn<AdminCaptureNode>('capturedAt', 'Captured'),
	{
		accessorKey: 'freshness',
		header: 'Freshness',
		enableSorting: false,
		cell: ({ row }) => renderComponent(FreshnessCell, { capture: row.original })
	},
	{
		accessorKey: 'runId',
		header: 'Run',
		enableSorting: false,
		cell: ({ row }) => renderComponent(RunCell, { capture: row.original })
	}
];
