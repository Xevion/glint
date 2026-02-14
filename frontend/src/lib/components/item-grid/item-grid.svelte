<script lang="ts" generics="T">
import { cn } from '$lib/utils';
import type { Component, Snippet } from 'svelte';
import { scale } from 'svelte/transition';

type GridMode = 'card' | 'row';
type CardSize = 'small' | 'medium' | 'large';

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
		<div class={cn('grid gap-5', className)} style={gridStyle}>
			{#each items as item, i (key ? key(item) : i)}
				<div in:scale={{ duration: 350, delay: entryDelay(i), start: 0.95 }}>
					{@render card(item, i)}
				</div>
			{/each}
		</div>
	{:else if mode === 'row' && row}
		<div class={cn('flex flex-col divide-y divide-border rounded-xl border border-border bg-card', className)}>
			{#each items as item, i (key ? key(item) : i)}
				{@render row(item, i)}
			{/each}
		</div>
	{/if}

	<!-- Infinite scroll sentinel -->
	{#if hasMore || loading}
		<div bind:this={sentinelEl} class="flex justify-center py-8">
			{#if loading}
				<div class="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
			{/if}
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
