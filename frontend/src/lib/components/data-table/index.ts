// Core table creation
export { createDataTable } from './create-table.svelte.js';
export { createSvelteTable } from './svelte-table.svelte.js';

// Rendering utilities
export { default as FlexRender } from './flex-render.svelte';
export { renderComponent, renderSnippet } from './render-helpers.js';

// Composed components
export { default as DataTable } from './data-table.svelte';
export { default as DataTablePagination } from './data-table-pagination.svelte';
export { default as DataTableColumnHeader } from './data-table-column-header.svelte';

// Column helpers
export { textColumn, timeColumn, imageColumn } from './columns.js';

// Re-export commonly used TanStack types for convenience
export type { ColumnDef } from '@tanstack/table-core';
