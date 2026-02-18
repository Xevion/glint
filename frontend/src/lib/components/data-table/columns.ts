import type { ColumnDef, RowData } from '@tanstack/table-core';
import type { Component } from 'svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import ImageCell from './cells/image-cell.svelte';
import ImageFallbackCell from './cells/image-fallback-cell.svelte';
import DataTableColumnHeader from './data-table-column-header.svelte';
import { renderComponent } from './render-helpers.js';

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
		header: ({ column }) => renderComponent(DataTableColumnHeader, { column, title: header }),
		cell: ({ row }) => {
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

	return {
		accessorKey,
		header,
		size: size === 'md' ? 64 : 48,
		enableSorting: false,
		cell: ({ row }) => {
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
		}
	};
}
