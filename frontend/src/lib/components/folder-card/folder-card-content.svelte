<script lang="ts">
import { cn } from '$lib/utils';
import { Tabs as TabsPrimitive } from 'bits-ui';
import type { Snippet } from 'svelte';

interface Props {
	value: string;
	class?: string;
	children: Snippet;
}

let { value, class: className, children }: Props = $props();
</script>

<TabsPrimitive.Content
	{value}
	class={cn('p-5 outline-none', className)}
>
	{@render children()}
</TabsPrimitive.Content>
