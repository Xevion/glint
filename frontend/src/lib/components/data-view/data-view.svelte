<script lang="ts" generics="T">
import { Alert } from '$lib/components/ui/alert';
import type { Snippet } from 'svelte';
import { setContext } from 'svelte';
import { DATA_VIEW_KEY, type DataViewContext } from './context';
import type { ListState, ViewMode } from './types';

interface Props {
	/** State object from createListState. */
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	list?: ListState<T, any>;
	/** Raw items array (creates a minimal internal state). */
	items?: T[];
	/** Error message for items-only mode (when not using createListState). */
	errorMessage?: string | null;
	/** View mode override for items-only mode (when not using createListState). */
	viewMode?: ViewMode;
	/** Content rendered when state has data. */
	children?: Snippet;
	/** Custom loading state. */
	loading?: Snippet;
	/** Custom empty state. */
	empty?: Snippet;
	/** Custom error state. Receives the error message. */
	error?: Snippet<[string]>;
}

let {
	list,
	items,
	errorMessage,
	viewMode,
	children,
	loading: loadingSnippet,
	empty: emptySnippet,
	error: errorSnippet
}: Props = $props();

// Stable context object with reactive getters so setContext captures a single
// reference that stays up-to-date as props/list state change.
const ctx: DataViewContext<T> = {
	get items() {
		if (list) return list.items;
		return items ?? [];
	},
	get loading() {
		return list?.loading ?? false;
	},
	get error() {
		return list?.error ?? errorMessage ?? null;
	},
	get hasMore() {
		return list?.hasMore ?? false;
	},
	get totalCount() {
		if (list) return list.totalCount;
		return (items ?? []).length;
	},
	get viewMode() {
		return list?.viewMode ?? viewMode ?? 'grid';
	},
	loadMore: async () => {
		await list?.loadMore();
	},
	get listState() {
		return list;
	}
};

setContext<DataViewContext<T>>(DATA_VIEW_KEY, ctx);

const showLoading = $derived(ctx.loading && ctx.items.length === 0);
const showError = $derived(ctx.error !== null && ctx.items.length === 0);
const showEmpty = $derived(!ctx.loading && ctx.error === null && ctx.items.length === 0);
const showContent = $derived(ctx.items.length > 0);
</script>

{#if showError}
	{#if errorSnippet}
		{@render errorSnippet(ctx.error!)}
	{:else}
		<Alert variant="destructive">Error: {ctx.error}</Alert>
	{/if}
{:else if showLoading}
	{#if loadingSnippet}
		{@render loadingSnippet()}
	{:else}
		<div class="flex justify-center py-12">
			<div
				role="status"
				aria-label="Loading"
				class="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent"
			></div>
		</div>
	{/if}
{:else if showEmpty}
	{#if emptySnippet}
		{@render emptySnippet()}
	{:else}
		<p class="py-12 text-center text-foreground">No items found.</p>
	{/if}
{:else if showContent && children}
	{@render children()}
{/if}
