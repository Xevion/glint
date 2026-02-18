import type { ShaderVersionDetail, ShaderVersionId } from '$lib/bindings';
import {
	type ColumnDef,
	DataTableColumnHeader,
	renderComponent,
	timeColumn
} from '$lib/components/data-table';
import { formatGameVersions } from '$lib/utils/display';
import VersionExtractionCell from './version-extraction-cell.svelte';
import VersionFileCell from './version-file-cell.svelte';
import VersionNameCell from './version-name-cell.svelte';

export function createVersionColumns(
	latestVersionId: ShaderVersionId | undefined
): ColumnDef<ShaderVersionDetail>[] {
	return [
		{
			accessorKey: 'version',
			size: 140,
			header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Version' }),
			cell: ({ row }) =>
				renderComponent(VersionNameCell, {
					version: row.original.version,
					isLatest: row.original.id === latestVersionId
				})
		},
		{
			accessorKey: 'game_versions',
			minSize: 120,
			header: 'Game Versions',
			cell: ({ row }) => formatGameVersions(row.original.game_versions),
			enableSorting: false
		},
		{
			accessorKey: 'release_channel',
			size: 90,
			header: 'Channel',
			cell: ({ row }) => {
				const ch = row.original.release_channel;
				return ch ? ch.charAt(0).toUpperCase() + ch.slice(1) : '-';
			},
			enableSorting: false
		},
		{
			accessorKey: 'extraction_status',
			size: 120,
			header: 'Extraction',
			cell: ({ row }) =>
				renderComponent(VersionExtractionCell, {
					status: row.original.extraction_status,
					extractedAt: row.original.extracted_at,
					error: row.original.extraction_error
				}),
			enableSorting: false
		},
		{
			id: 'file',
			size: 140,
			header: 'File',
			cell: ({ row }) =>
				renderComponent(VersionFileCell, {
					fileSize: row.original.file_size,
					fileHash: row.original.file_hash,
					downloadUrl: row.original.download_url
				}),
			enableSorting: false
		},
		timeColumn<ShaderVersionDetail>('created_at', 'Created')
	];
}
