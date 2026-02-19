import type { ColumnDef, HeaderContext, RowData } from '@tanstack/table-core';
import type { Component } from 'svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import ImageCell from './cells/image-cell.svelte';
import ImageFallbackCell from './cells/image-fallback-cell.svelte';
import DataTableColumnHeader from './data-table-column-header.svelte';
import { renderComponent } from './render-helpers.js';

/** Builds a sortable header callback for DataTableColumnHeader. */
function sortableHeaderDef<TData extends RowData>(title: string) {
	return (ctx: HeaderContext<TData, unknown>) =>
		renderComponent(DataTableColumnHeader, { column: ctx.column, title });
}

/**
 * Creates a basic text column with an optional sortable header.
 *
 * @example
 * ```ts
 * const columns: ColumnDef<User>[] = [
 *   textColumn('discord_username', 'Username'),
 *   textColumn('role', 'Role', { sortable: false }),
 * ];
 * ```
 */
export function textColumn<TData extends RowData>(
	accessorKey: string & keyof TData,
	header: string,
	opts: {
		sortable?: boolean;
		size?: number;
		minSize?: number;
	} = {}
): ColumnDef<TData> {
	const { sortable = true, size, minSize } = opts;
	return {
		accessorKey,
		...(size != null && { size }),
		...(minSize != null && { minSize }),
		header: sortable ? sortableHeaderDef(header) : header,
		enableSorting: sortable
	};
}

/**
 * Creates a time-ago column that renders a `<TimeAgo>` component.
 *
 * @example
 * ```ts
 * const columns: ColumnDef<Shader>[] = [
 *   timeColumn('created_at', 'Created'),
 * ];
 * ```
 */
export function timeColumn<TData extends RowData>(
	accessorKey: string & keyof TData,
	header: string
): ColumnDef<TData> {
	return {
		accessorKey,
		size: 120,
		header: sortableHeaderDef<TData>(header),
		cell: ({ row }: { row: { getValue: <T>(key: string) => T } }) => {
			const value = row.getValue<string | undefined>(accessorKey);
			if (!value) return '-';
			return renderComponent(TimeAgo, { timestamp: value });
		}
	};
}

/**
 * Creates an image column with a rounded thumbnail and optional fallback.
 *
 * The `fallback` option accepts a Svelte component (e.g. a Lucide icon) to
 * render when the image URL is missing.
 *
 * @example
 * ```ts
 * import { Sparkles } from '@lucide/svelte';
 *
 * const columns: ColumnDef<Shader>[] = [
 *   imageColumn('icon_url', { fallback: Sparkles }),
 * ];
 * ```
 */
export function imageColumn<TData extends RowData>(
	accessorKey: string & keyof TData,
	opts: {
		header?: string;
		size?: 'sm' | 'md';
		rounded?: 'full' | 'md';
		fallback?: Component;
	} = {}
): ColumnDef<TData> {
	const { header = '', size = 'sm', rounded = 'md' } = opts;

	const sizeClass = size === 'md' ? 'h-12 w-12' : 'h-8 w-8';
	const roundedClass = rounded === 'full' ? 'rounded-full' : 'rounded';

	const cellDef = ({ row }: { row: { getValue: <T>(key: string) => T } }) => {
		const url = row.getValue<string | undefined>(accessorKey);
		if (url) {
			return renderComponent(ImageCell, { url, sizeClass, roundedClass });
		}
		if (opts.fallback) {
			return renderComponent(ImageFallbackCell, {
				icon: opts.fallback,
				sizeClass,
				roundedClass
			});
		}
		return '';
	};

	return {
		accessorKey,
		header,
		size: size === 'md' ? 64 : 48,
		enableSorting: false,
		cell: cellDef
	};
}
