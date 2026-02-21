<script lang="ts">
import { cn } from '$lib/utils';
import type { Snippet } from 'svelte';

interface Props {
	children?: Snippet;
	/** Custom CSS class for the outer container. */
	class?: string;
}

let { children, class: className }: Props = $props();
</script>

<div
	class={cn(
		'flex flex-wrap items-center gap-3 rounded-xl border border-border bg-card px-4 py-3',
		className
	)}
>
	{#if children}
		{@render children()}
	{/if}
</div>
