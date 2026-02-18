import type { CaptureWithContext } from '$lib/bindings';
import {
	type ColumnDef,
	renderComponent,
	textColumn,
	timeColumn
} from '$lib/components/data-table';
import FreshnessCell from './freshness-cell.svelte';
import PreviewCell from './preview-cell.svelte';
import ResolutionCell from './resolution-cell.svelte';
import RunCell from './run-cell.svelte';
import ShaderCell from './shader-cell.svelte';
import SizeCell from './size-cell.svelte';

export const columns: ColumnDef<CaptureWithContext>[] = [
	{
		accessorKey: 'image_url',
		header: 'Preview',
		size: 96,
		enableSorting: false,
		cell: ({ row }) => renderComponent(PreviewCell, { capture: row.original })
	},
	{
		accessorKey: 'shader_name',
		header: 'Shader',
		enableSorting: false,
		cell: ({ row }) => renderComponent(ShaderCell, { capture: row.original })
	},
	textColumn<CaptureWithContext>('scene_name', 'Scene', { sortable: false }),
	textColumn<CaptureWithContext>('profile_name', 'Profile', { sortable: false }),
	{
		accessorKey: 'resolution_width',
		header: 'Resolution',
		enableSorting: false,
		cell: ({ row }) => renderComponent(ResolutionCell, { capture: row.original })
	},
	{
		accessorKey: 'file_size_bytes',
		header: 'Size',
		enableSorting: false,
		cell: ({ row }) => renderComponent(SizeCell, { capture: row.original })
	},
	timeColumn<CaptureWithContext>('captured_at', 'Captured'),
	{
		accessorKey: 'freshness',
		header: 'Freshness',
		enableSorting: false,
		cell: ({ row }) => renderComponent(FreshnessCell, { capture: row.original })
	},
	{
		accessorKey: 'run_id',
		header: 'Run',
		enableSorting: false,
		cell: ({ row }) => renderComponent(RunCell, { capture: row.original })
	}
];
