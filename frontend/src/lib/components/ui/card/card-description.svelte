<script lang="ts">
import { type WithElementRef, cn } from '$lib/utils.js';
import type { Snippet } from 'svelte';
import type { HTMLAttributes } from 'svelte/elements';

let {
	ref = $bindable(null),
	class: className,
	children,
	...restProps
}: WithElementRef<HTMLAttributes<HTMLDivElement>> & {
	children?: Snippet;
} = $props();
</script>

<div
	bind:this={ref}
	data-slot="card-description"
	class={cn("text-sm text-muted-foreground", className)}
	{...restProps}
>
	{@render children?.()}
</div>
