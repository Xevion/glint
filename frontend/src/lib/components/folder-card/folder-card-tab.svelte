<script lang="ts">
import { cn } from '$lib/utils.js';
import { Tabs as TabsPrimitive } from 'bits-ui';
import type { Snippet } from 'svelte';
import { getContext, onMount } from 'svelte';

interface Props {
	value: string;
	class?: string;
	children: Snippet;
}

let { value, class: className, children }: Props = $props();

const { registerTab, unregisterTab } = getContext<{
	registerTab: (value: string, el: HTMLElement) => void;
	unregisterTab: (value: string) => void;
}>('folder-card');

let triggerRef: HTMLElement | null = $state(null);

onMount(() => {
	if (triggerRef) registerTab(value, triggerRef);
	return () => unregisterTab(value);
});

// Re-register when the ref changes (e.g. if bits-ui recreates the element)
$effect(() => {
	if (triggerRef) registerTab(value, triggerRef);
});
</script>

<TabsPrimitive.Trigger
	bind:ref={triggerRef}
	{value}
	class={cn(
		'relative z-10 inline-flex items-center gap-1.5 rounded-md bg-accent px-3 py-1.5 text-sm font-medium transition-colors select-none',
		'text-muted-foreground hover:text-foreground data-[state=active]:bg-transparent data-[state=active]:text-foreground',
		'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1',
		className
	)}
>
	{@render children()}
</TabsPrimitive.Trigger>
