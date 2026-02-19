import { type ColumnDef, DataTableColumnHeader, renderComponent } from '$lib/components/data-table';
import { formatGameVersions } from '$lib/utils/display';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import VersionExtractionCell from './version-extraction-cell.svelte';
import VersionFileCell from './version-file-cell.svelte';
import VersionNameCell from './version-name-cell.svelte';
import type { AdminShaderVersion } from './queries';

export function createVersionColumns(
	latestVersionId: string | undefined
): ColumnDef<AdminShaderVersion>[] {
	return [
		{
			id: 'version',
			size: 140,
			header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Version' }),
			cell: ({ row }) =>
				renderComponent(VersionNameCell, {
					version: row.original.version.version,
					isLatest: row.original.version.id === latestVersionId
				})
		},
		{
			id: 'gameVersions',
			minSize: 120,
			header: 'Game Versions',
			cell: ({ row }) => formatGameVersions(row.original.version.gameVersions ?? undefined),
			enableSorting: false
		},
		{
			id: 'releaseChannel',
			size: 90,
			header: 'Channel',
			cell: ({ row }) => {
				const ch = row.original.version.releaseChannel;
				return ch ? ch.charAt(0).toUpperCase() + ch.slice(1) : '-';
			},
			enableSorting: false
		},
		{
			id: 'extractionStatus',
			size: 120,
			header: 'Extraction',
			cell: ({ row }) =>
				renderComponent(VersionExtractionCell, {
					status: row.original.version.extractionStatus,
					extractedAt: row.original.version.extractedAt ?? undefined,
					error: row.original.version.extractionError ?? undefined
				}),
			enableSorting: false
		},
		{
			id: 'file',
			size: 140,
			header: 'File',
			cell: ({ row }) =>
				renderComponent(VersionFileCell, {
					fileSize: row.original.version.fileSize ?? undefined,
					fileHash: row.original.version.fileHash ?? undefined,
					downloadUrl: row.original.version.downloadUrl ?? undefined
				}),
			enableSorting: false
		},
		{
			id: 'createdAt',
			size: 120,
			header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: 'Created' }),
			cell: ({ row }) => {
				const date = row.original.version.createdAt;
				if (!date) return '-';
				return renderComponent(TimeAgo, { timestamp: date });
			}
		}
	];
}
