// Root component
export { default as DataView } from './data-view.svelte';

// Renderers
export { default as Grid } from './data-view-grid.svelte';
export { default as Table } from './data-view-table.svelte';

// Toolbar controls
export { default as Toolbar } from './data-view-toolbar.svelte';
export { default as Search } from './data-view-search.svelte';
export { default as Sort } from './data-view-sort.svelte';
export { default as ViewToggle } from './data-view-view-toggle.svelte';
export { default as Pagination } from './data-view-pagination.svelte';
export { default as Counter } from './data-view-counter.svelte';

// State management
export { createClientList, type ClientListConfig } from './create-client-list.svelte';
export { createCursorList, type CursorListConfig } from './create-cursor-list.svelte';
export { VIEW_MODE_META, type ListState, type ViewMode, type SortOption } from './types';
export { type RelayConnection, emptyConnection } from './connection';
export { filter, extractFiltersFromUrl, type FilterDescriptor } from './filter';

// Context (for advanced consumers building custom renderers)
export { getDataViewContext, type DataViewContext } from './context';

// TanStack rendering utilities (re-exported from data-table)
export { renderComponent, renderSnippet } from '$lib/components/data-table';
