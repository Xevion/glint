<script lang="ts" generics="T">
import { cn } from '$lib/utils';
import type { Snippet } from 'svelte';
import { scale } from 'svelte/transition';
import { getDataViewContext } from './context';
import type { ViewMode } from './types';

type CardSize = 'small' | 'medium' | 'large';

interface Props {
	/** Card render snippet — receives the item and its index. */
	card?: Snippet<[T, number]>;
	/** Row render snippet — receives the item and its index. */
	row?: Snippet<[T, number]>;
	/** Display mode. Unrecognized modes (e.g. 'table') render nothing. */
	mode?: ViewMode;
	/** Card minimum width preset (only applies in grid mode). */
	size?: CardSize;
	/** Link generator (tile mode). When provided, each tile renders as an <a>; otherwise a <div>. */
	href?: (item: T) => string;
	/** Enable xl:grid-cols-4 breakpoint (tile mode). */
	xlCols?: 4;
	/** Unique key extractor. Falls back to index. */
	key?: (item: T) => string | number;
	/** Container class override. */
	class?: string;
}

let {
	card,
	row,
	mode,
	size = 'medium',
	href,
	xlCols,
	key: keyFn,
	class: className
}: Props = $props();

const ctx = getDataViewContext<T>();

const resolvedMode = $derived(mode ?? ctx.viewMode);

const CARD_SIZE_MIN: Record<CardSize, string> = {
	small: '220px',
	medium: '300px',
	large: '380px'
};

const gridStyle = $derived(
	resolvedMode === 'grid'
		? `grid-template-columns: repeat(auto-fill, minmax(${CARD_SIZE_MIN[size]}, 1fr));`
		: ''
);

// Entry animation stagger: cap at 400ms total delay
function entryDelay(index: number): number {
	return Math.min(index * 50, 400) + 150;
}

// Animate entries on initial mount and after appends
let animateEntries = $state(true);
let prevItemCount = $state(0);

$effect(() => {
	const timeout = setTimeout(() => {
		animateEntries = false;
	}, 600);
	return () => clearTimeout(timeout);
});

$effect(() => {
	const count = ctx.items.length;
	if (count > prevItemCount && prevItemCount > 0 && ctx.hasMore) {
		animateEntries = true;
		const timeout = setTimeout(() => {
			animateEntries = false;
		}, 600);
		prevItemCount = count;
		return () => clearTimeout(timeout);
	}
	prevItemCount = count;
});
</script>

{#if resolvedMode === 'grid' && card}
	<div role="list" class={cn('grid gap-5', className)} style={gridStyle}>
		{#each ctx.items as item, i (keyFn ? keyFn(item) : i)}
			<div
				role="listitem"
				in:scale={{
					duration: animateEntries ? 350 : 0,
					delay: animateEntries ? entryDelay(i) : 0,
					start: 0.95
				}}
			>
				{@render card(item, i)}
			</div>
		{/each}
	</div>
{:else if resolvedMode === 'row' && row}
	<div
		role="list"
		class={cn(
			'flex flex-col divide-y divide-border rounded-xl border border-border bg-card',
			className
		)}
	>
		{#each ctx.items as item, i (keyFn ? keyFn(item) : i)}
			<div role="listitem">
				{@render row(item, i)}
			</div>
		{/each}
	</div>
{:else if resolvedMode === 'tile' && card}
	<div
		class={cn(
			'-mb-px -mr-px grid grid-cols-1 overflow-hidden rounded-md border border-border bg-card sm:grid-cols-2 lg:grid-cols-3',
			xlCols === 4 && 'xl:grid-cols-4',
			className
		)}
	>
		{#each ctx.items as item, i (keyFn ? keyFn(item) : i)}
			<svelte:element
				this={href ? 'a' : 'div'}
				href={href ? href(item) : undefined}
				class="group mb-[-1px] mr-[-1px] border-b border-r border-border transition-colors hover:bg-muted/50"
			>
				{@render card(item, i)}
			</svelte:element>
		{/each}
	</div>
{/if}
