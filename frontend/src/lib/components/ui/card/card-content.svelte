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
	data-slot="card-content"
	class={cn("px-6 pb-6", className)}
	{...restProps}
>
	{@render children?.()}
</div>
