<script lang="ts">
import { type WithElementRef, cn } from '$lib/utils';
import type { HTMLTableAttributes } from 'svelte/elements';

let {
	ref = $bindable(null),
	class: className,
	children,
	...restProps
}: WithElementRef<HTMLTableAttributes> = $props();
</script>

<div data-slot="table-container" class={cn("relative w-full overflow-x-auto rounded-md", className)}>
	<table
		bind:this={ref}
		data-slot="table"
		class="w-full caption-bottom text-sm bg-card"
		{...restProps}
	>
		{@render children?.()}
	</table>
</div>
