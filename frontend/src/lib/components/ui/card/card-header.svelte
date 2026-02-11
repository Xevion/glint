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
	data-slot="card-header"
	class={cn("flex flex-col gap-1.5 p-6", className)}
	{...restProps}
>
	{@render children?.()}
</div>
