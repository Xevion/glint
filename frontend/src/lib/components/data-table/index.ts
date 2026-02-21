// Core table creation
export { createSvelteTable } from './svelte-table.svelte.js';

// Rendering utilities
export { default as FlexRender } from './flex-render.svelte';
export { renderComponent, renderSnippet } from './render-helpers.js';

// Components
export { default as DataTableColumnHeader } from './data-table-column-header.svelte';

// Column helpers
export { textColumn, timeColumn, imageColumn } from './columns.js';

// Re-export commonly used TanStack types for convenience
export type { ColumnDef } from '@tanstack/table-core';
