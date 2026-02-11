<script lang="ts">
import { type WithElementRef, cn } from '$lib/utils.js';
import type { HTMLAttributes } from 'svelte/elements';

let {
	ref = $bindable(null),
	class: className,
	...restProps
}: WithElementRef<HTMLAttributes<HTMLDivElement>> = $props();
</script>

<div
	bind:this={ref}
	data-slot="skeleton"
	class={cn("animate-pulse rounded-md bg-muted", className)}
	{...restProps}
></div>
