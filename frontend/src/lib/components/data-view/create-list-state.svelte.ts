/**
 * Re-exports for backward compatibility.
 *
 * Prefer importing directly from:
 * - `./create-client-list.svelte` for client-side lists
 * - `./create-cursor-list.svelte` for GraphQL cursor-paginated lists
 * - `./types` for shared types (ListState, ViewMode, SortOption)
 * - `./connection` for RelayConnection
 */
export { createClientList, type ClientListConfig } from './create-client-list.svelte';
export { createCursorList, type CursorListConfig } from './create-cursor-list.svelte';
export { type RelayConnection } from './connection';
export { type ListState, type SortOption, type ViewMode } from './types';
