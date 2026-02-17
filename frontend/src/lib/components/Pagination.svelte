<script lang="ts">
import { formatNumber } from '$lib/utils/display';
import { ChevronUp } from '@lucide/svelte';
import * as Select from '$lib/components/ui/select';
import type { Action } from 'svelte/action';

const slideIn: Action<HTMLElement, number> = (node, direction) => {
	if (direction !== 0) {
		node.animate(
			[
				{ transform: `translateX(${direction * 20}px)`, opacity: 0 },
				{ transform: 'translateX(0)', opacity: 1 }
			],
			{ duration: 200, easing: 'ease-out' }
		);
	}
};

interface Props {
	totalCount: number;
	page: number;
	pageSize: number;
	loading?: boolean;
	onPageChange: (page: number) => void;
}

let { totalCount, page, pageSize, loading = false, onPageChange }: Props = $props();

const totalPages = $derived(Math.ceil(totalCount / pageSize));
const start = $derived((page - 1) * pageSize + 1);
const end = $derived(Math.min(page * pageSize, totalCount));

// Track direction for slide animation
let direction = $state(0);

// 5 page slots: current-2, current-1, current, current+1, current+2
const pageSlots = $derived([-2, -1, 0, 1, 2].map((delta) => page + delta));

function isSlotVisible(p: number): boolean {
	return p >= 1 && p <= totalPages;
}

function goToPage(p: number) {
	direction = p > page ? 1 : -1;
	onPageChange(p);
}

const MAX_DROPDOWN_ITEMS = 50;
const pageItems = $derived.by(() => {
	if (totalPages <= MAX_DROPDOWN_ITEMS) {
		return Array.from({ length: totalPages }, (_, i) => ({
			value: String(i + 1),
			label: String(i + 1)
		}));
	}
	const half = Math.floor(MAX_DROPDOWN_ITEMS / 2);
	const start = Math.max(1, page - half);
	const end = Math.min(totalPages, start + MAX_DROPDOWN_ITEMS - 1);
	const adjustedStart = Math.max(1, end - MAX_DROPDOWN_ITEMS + 1);
	return Array.from({ length: end - adjustedStart + 1 }, (_, i) => ({
		value: String(adjustedStart + i),
		label: String(adjustedStart + i)
	}));
});

const selectValue = $derived(String(page));
</script>

{#if totalCount > 0 && totalPages > 1}
	<div class="mt-2 flex items-start pl-2 text-xs">
		<!-- Left zone: result count -->
		<div class="flex-1">
			<span class="hidden select-none text-foreground/70 md:inline">
				Showing {formatNumber(start)}&ndash;{formatNumber(end)} of {formatNumber(totalCount)} items
			</span>
			<span class="select-none tabular-nums text-foreground/70 md:hidden">
				{formatNumber(start)}&ndash;{formatNumber(end)} / {formatNumber(totalCount)}
			</span>
		</div>

		<!-- Center zone: page buttons -->
		<div class="flex items-center gap-1">
			{#key page}
				{#each pageSlots as p, i (i)}
					{#if i === 2}
						<!-- Center slot: current page with dropdown trigger -->
						<Select.Root
							type="single"
							value={selectValue}
							onValueChange={(v) => {
								if (v) goToPage(Number(v));
							}}
							items={pageItems}
						>
							<Select.Trigger
								class="inline-flex h-9 min-w-9 cursor-pointer items-center justify-center gap-1 rounded-md border border-border bg-card px-2.5 text-sm font-medium tabular-nums text-foreground outline-none transition-colors select-none hover:bg-muted/50 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background active:bg-muted {loading
									? 'animate-pulse'
									: ''}"
								aria-label="Page {page} of {totalPages}, click to select page"
							>
								<span use:slideIn={direction}>{page}</span>
								<ChevronUp class="size-3 text-muted-foreground" />
							</Select.Trigger>
							<Select.Content side="top">
								{#each pageItems as item (item.value)}
									<Select.Item
										class="flex h-8 w-full items-center justify-center rounded-sm px-3 text-sm tabular-nums outline-hidden select-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground data-[selected]:font-semibold"
										value={item.value}
										label={item.label}
									/>
								{/each}
							</Select.Content>
						</Select.Root>
					{:else}
						<!-- Side slot: navigable page button or invisible placeholder -->
						<button
							class="inline-flex h-9 w-9 cursor-pointer items-center justify-center rounded-md text-sm tabular-nums text-muted-foreground transition-colors select-none hover:bg-muted/50 hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background active:bg-muted {!isSlotVisible(
								p
							)
								? 'invisible'
								: loading
									? 'opacity-40'
									: ''} {!isSlotVisible(p) || loading ? 'pointer-events-none' : ''}"
							onclick={() => goToPage(p)}
							aria-label="Go to page {p}"
							aria-hidden={!isSlotVisible(p)}
							tabindex={isSlotVisible(p) ? 0 : -1}
							disabled={!isSlotVisible(p) || loading}
							use:slideIn={direction}
						>
							{p}
						</button>
					{/if}
				{/each}
			{/key}
		</div>

		<!-- Right zone: spacer for centering -->
		<div class="flex-1"></div>
	</div>
{:else if totalCount > 0}
	<!-- Single page: just show the count, no pagination controls -->
	<div class="mt-2 flex items-start pl-2 text-xs">
		<span class="hidden select-none text-foreground/70 md:inline">
			Showing {formatNumber(start)}&ndash;{formatNumber(end)} of {formatNumber(totalCount)} items
		</span>
		<span class="select-none tabular-nums text-foreground/70 md:hidden">
			{formatNumber(start)}&ndash;{formatNumber(end)} / {formatNumber(totalCount)}
		</span>
	</div>
{/if}
