<script lang="ts">
import { cn } from '$lib/utils.js';
import { Tabs as TabsPrimitive } from 'bits-ui';
import type { Snippet } from 'svelte';
import { setContext } from 'svelte';

interface Props {
	value?: string;
	onValueChange?: (value: string) => void;
	class?: string;
	/** Tab trigger components (FolderCard.Tab) */
	tabs: Snippet;
	/** Tab content panels (FolderCard.Content) */
	children: Snippet;
	/** Optional content rendered to the right of the tab buttons */
	trailing?: Snippet;
}

let {
	value = $bindable(''),
	onValueChange,
	class: className,
	tabs,
	children,
	trailing
}: Props = $props();

// Tab registration: each FolderCardTab registers its element ref by value
let tabRefs = $state(new Map<string, HTMLElement>());
let tabBarRef: HTMLDivElement | undefined = $state();
let pillRef: HTMLDivElement | undefined = $state();

function registerTab(tabValue: string, el: HTMLElement) {
	tabRefs.set(tabValue, el);
}

function unregisterTab(tabValue: string) {
	tabRefs.delete(tabValue);
}

setContext('folder-card', { registerTab, unregisterTab });

// Pill positioning via CSS transitions
$effect(() => {
	void value;
	void tabRefs.size;

	const activeEl = tabRefs.get(value);
	const pill = pillRef;
	const bar = tabBarRef;
	if (!activeEl || !pill || !bar) {
		if (pill) pill.style.opacity = '0';
		return;
	}

	const barRect = bar.getBoundingClientRect();
	const tabRect = activeEl.getBoundingClientRect();

	pill.style.transform = `translateX(${tabRect.left - barRect.left}px)`;
	pill.style.width = `${tabRect.width}px`;
	pill.style.opacity = '1';
});

// Snap pill on resize without animation
$effect(() => {
	const bar = tabBarRef;
	if (!bar) return;

	const observer = new ResizeObserver(() => {
		const activeEl = tabRefs.get(value);
		const pill = pillRef;
		const currentBar = tabBarRef;
		if (!activeEl || !pill || !currentBar) return;

		const barRect = currentBar.getBoundingClientRect();
		const tabRect = activeEl.getBoundingClientRect();

		// Temporarily disable transition for instant repositioning
		pill.style.transition = 'none';
		pill.style.transform = `translateX(${tabRect.left - barRect.left}px)`;
		pill.style.width = `${tabRect.width}px`;
		// Re-enable on next frame
		requestAnimationFrame(() => {
			const p = pillRef;
			if (p) p.style.transition = '';
		});
	});

	observer.observe(bar);
	return () => observer.disconnect();
});
</script>

<TabsPrimitive.Root
	bind:value
	{onValueChange}
	class={cn('flex flex-col', className)}
>
	<!-- Tab bar row -->
	<div class="flex items-end">
		<!-- Tab group: bg-card, border on 3 sides, overlaps card top border by 1px -->
		<div
			bind:this={tabBarRef}
			class="relative z-10 -mb-px inline-flex items-center gap-1 rounded-t-lg border border-b-0 border-border bg-card p-1 pb-2"
		>
			<!-- Sliding pill (contrasting bg behind active tab) -->
			<div
				bind:this={pillRef}
				class="absolute top-1 left-0 h-[calc(100%-12px)] rounded-md bg-foreground/[0.08] opacity-0 shadow-sm transition-[transform,width,opacity] duration-200 ease-out"
			></div>

			<TabsPrimitive.List class="contents">
				{@render tabs()}
			</TabsPrimitive.List>
		</div>

		{#if trailing}
			<div class="ml-auto flex items-center pb-2">
				{@render trailing()}
			</div>
		{/if}
	</div>

	<!-- Card body -->
	<div class="rounded-b-lg rounded-tr-lg border border-border bg-card shadow-sm">
		{@render children()}
	</div>
</TabsPrimitive.Root>
