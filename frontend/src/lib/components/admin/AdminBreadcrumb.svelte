<script lang="ts">
import { ArrowLeft, ChevronRight } from '@lucide/svelte';
import type { Snippet } from 'svelte';

export interface BreadcrumbSegment {
	label: string;
	href?: string;
}

interface Props {
	backHref: string;
	backLabel?: string;
	segments: BreadcrumbSegment[];
	trailing?: Snippet;
}

let { backHref, backLabel = 'Go back', segments, trailing }: Props = $props();
</script>

<nav class="flex items-center gap-1.5 text-sm">
	<a href={backHref} class="text-muted-foreground hover:text-foreground" aria-label={backLabel}>
		<ArrowLeft class="h-4 w-4" />
	</a>
	{#each segments as segment, i (i)}
		{#if i > 0}
			<ChevronRight class="h-3.5 w-3.5 text-muted-foreground/60" />
		{/if}
		{#if segment.href}
			<a href={segment.href} class="text-muted-foreground hover:text-foreground">
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
