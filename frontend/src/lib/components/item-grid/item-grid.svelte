<script lang="ts" generics="T">
import { cn } from '$lib/utils';
import type { Component, Snippet } from 'svelte';
import { scale } from 'svelte/transition';
import type { CardSize, GridMode } from './types';

interface EmptyState {
	icon?: Component<{ class?: string; strokeWidth?: number }>;
	title: string;
	message?: string;
}

interface Props {
	items: T[];
	/** Unique key extractor for each item. Falls back to index if not provided. */
	key?: (item: T) => string | number;
	/** Display mode: 'card' for grid layout, 'row' for compact list */
	mode?: GridMode;
	/** Card size preset (only applies in card mode) */
	size?: CardSize;
	/** Custom CSS class for the outer container */
	class?: string;
	/** Empty state configuration shown when items array is empty */
	empty?: EmptyState;
	/** Whether more items are currently being loaded */
	loading?: boolean;
	/** Whether there are more items available to load */
	hasMore?: boolean;
	/** Number of skeleton placeholders shown during initial load (default: 6) */
	skeletonCount?: number;
	/** Called when the scroll sentinel becomes visible */
	onLoadMore?: () => void;
	/** Card render snippet — receives the item and its index */
	card?: Snippet<[T, number]>;
	/** Compact row render snippet — receives the item and its index */
	row?: Snippet<[T, number]>;
}

let {
	items,
	key,
	mode = 'card',
	size = 'medium',
	class: className,
	empty,
	loading = false,
	hasMore = false,
	skeletonCount = 6,
	onLoadMore,
	card,
	row
}: Props = $props();

const CARD_SIZE_MIN: Record<CardSize, string> = {
	small: '220px',
	medium: '300px',
	large: '380px'
};

const gridStyle = $derived(
	mode === 'card'
		? `grid-template-columns: repeat(auto-fill, minmax(${CARD_SIZE_MIN[size]}, 1fr));`
		: ''
);

// Entry animation stagger: cap at 400ms total delay
function entryDelay(index: number): number {
	return Math.min(index * 50, 400) + 150;
}

// Only animate entries on initial mount and after infinite scroll appends.
// Without this, every filter change (e.g. search keystroke) re-fires entry
// animations for items that re-enter the {#each} block.
let animateEntries = $state(true);
let prevItemCount = $state(0);

// Disable entry animations after the initial stagger completes
$effect(() => {
	const timeout = setTimeout(() => {
		animateEntries = false;
	}, 600);
	return () => clearTimeout(timeout);
});

// Re-enable animations briefly when items are appended via infinite scroll
$effect(() => {
	const count = items.length;
	if (count > prevItemCount && prevItemCount > 0 && onLoadMore) {
		animateEntries = true;
		const timeout = setTimeout(() => {
			animateEntries = false;
		}, 600);
		prevItemCount = count;
		return () => clearTimeout(timeout);
	}
	prevItemCount = count;
});

// Infinite scroll via IntersectionObserver
let sentinelEl = $state<HTMLDivElement | null>(null);

$effect(() => {
	const el = sentinelEl;
	if (!el) return;

	const observer = new IntersectionObserver(
		(entries) => {
			const entry = entries[0];
			if (entry?.isIntersecting && hasMore && !loading && onLoadMore) {
				onLoadMore();
			}
		},
		{ rootMargin: '200px' }
	);

	observer.observe(el);

	return () => observer.disconnect();
});
</script>

{#if items.length > 0}
	{#if mode === 'card' && card}
		<div role="list" class={cn('grid gap-5', className)} style={gridStyle}>
			{#each items as item, i (key ? key(item) : i)}
				<div role="listitem" in:scale={{ duration: animateEntries ? 350 : 0, delay: animateEntries ? entryDelay(i) : 0, start: 0.95 }}>
					{@render card(item, i)}
				</div>
			{/each}
		</div>
	{:else if mode === 'row' && row}
		<div role="list" class={cn('flex flex-col divide-y divide-border rounded-xl border border-border bg-card', className)}>
			{#each items as item, i (key ? key(item) : i)}
				<div role="listitem">
					{@render row(item, i)}
				</div>
			{/each}
		</div>
	{/if}

	<!-- Infinite scroll sentinel -->
	{#if hasMore || loading}
		<div bind:this={sentinelEl} class="flex justify-center py-8">
			{#if loading}
				<div role="status" aria-label="Loading more items" class="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
			{/if}
		</div>
	{/if}
{:else if loading && skeletonCount > 0}
	<!-- Skeleton placeholders during initial load -->
	{#if mode === 'card'}
		<div aria-busy="true" aria-label="Loading items" class={cn('grid gap-5', className)} style={gridStyle}>
			{#each { length: skeletonCount } as _, i (i)}
				<div class="animate-pulse overflow-hidden rounded-xl border border-border bg-card">
					<div class="aspect-video w-full bg-muted"></div>
					<div class="space-y-2 p-4">
						<div class="h-4 w-3/4 rounded bg-muted"></div>
						<div class="h-3 w-1/2 rounded bg-muted"></div>
					</div>
				</div>
			{/each}
		</div>
	{:else}
		<div aria-busy="true" aria-label="Loading items" class={cn('flex flex-col divide-y divide-border rounded-xl border border-border bg-card', className)}>
			{#each { length: skeletonCount } as _, i (i)}
				<div class="flex animate-pulse items-center gap-3 px-4 py-2.5">
					<div class="h-10 w-10 shrink-0 rounded-md bg-muted"></div>
					<div class="min-w-0 flex-1 space-y-1.5">
						<div class="h-3.5 w-2/5 rounded bg-muted"></div>
						<div class="h-3 w-1/4 rounded bg-muted"></div>
					</div>
				</div>
			{/each}
		</div>
	{/if}
{:else if empty}
	<div class="flex flex-col items-center justify-center py-16 text-center">
		{#if empty.icon}
			{@const Icon = empty.icon}
			<Icon class="mb-4 h-16 w-16 text-muted-foreground opacity-50" strokeWidth={1.5} />
		{/if}
		<h3 class="text-lg font-semibold text-foreground">{empty.title}</h3>
		{#if empty.message}
			<p class="mt-1 text-sm text-foreground/70">{empty.message}</p>
		{/if}
	</div>
{/if}
