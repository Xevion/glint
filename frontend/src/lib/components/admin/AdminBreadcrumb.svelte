<script lang="ts">
import { ChevronRight } from '@lucide/svelte';
import type { Snippet } from 'svelte';

export interface BreadcrumbSegment {
	label: string;
	href?: string;
}

interface Props {
	segments: BreadcrumbSegment[];
	trailing?: Snippet;
}

let { segments, trailing }: Props = $props();
</script>

<nav class="flex items-center gap-1.5 text-sm">
	{#each segments as segment, i (i)}
		{#if i > 0}
			<ChevronRight class="h-3.5 w-3.5 text-foreground opacity-40" />
		{/if}
		{#if segment.href}
			<a href={segment.href} class="text-foreground hover:underline">
				{segment.label}
			</a>
		{:else}
			<span class="font-medium text-foreground">{segment.label}</span>
		{/if}
	{/each}
	{#if trailing}
		{@render trailing()}
	{/if}
</nav>
