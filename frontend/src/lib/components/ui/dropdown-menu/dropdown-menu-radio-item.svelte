<script lang="ts">
import { cn } from '$lib/utils.js';
import CircleIcon from '@lucide/svelte/icons/circle';
import { DropdownMenu as DropdownMenuPrimitive } from 'bits-ui';

type Props = Omit<DropdownMenuPrimitive.RadioItemProps, 'child'> & {
	ref?: HTMLElement | null;
};

let {
	ref = $bindable(null),
	class: className,
	children: childrenProp,
	...restProps
}: Props = $props();
</script>

<DropdownMenuPrimitive.RadioItem
	bind:ref
	data-slot="dropdown-menu-radio-item"
	class={cn(
		"relative flex cursor-default items-center gap-2 rounded-sm py-1.5 ps-8 pe-2 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
		className
	)}
	{...restProps}
>
	{#snippet children({ checked })}
		<span class="pointer-events-none absolute start-2 flex size-3.5 items-center justify-center">
			{#if checked}
				<CircleIcon class="size-2 fill-current" />
			{/if}
		</span>
		<!-- eslint-disable-next-line @typescript-eslint/no-unsafe-assignment -->
		{@render childrenProp?.({ checked })}
	{/snippet}
</DropdownMenuPrimitive.RadioItem>
