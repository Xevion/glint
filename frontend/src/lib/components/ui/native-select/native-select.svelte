<script lang="ts" module>
import { type VariantProps, tv } from 'tailwind-variants';

export const nativeSelectVariants = tv({
	base:
		'rounded-md border border-input bg-background text-sm shadow-xs transition-[color,box-shadow] outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-input/30',
	variants: {
		size: {
			sm: 'h-8 px-2',
			default: 'h-9 px-3'
		}
	},
	defaultVariants: {
		size: 'default'
	}
});

export type NativeSelectSize = VariantProps<typeof nativeSelectVariants>['size'];
</script>

<script lang="ts">
	import type { HTMLSelectAttributes } from 'svelte/elements';
	import { cn, type WithElementRef } from '$lib/utils.js';

	let {
		ref = $bindable(null),
		class: className,
		size = 'default',
		children,
		...restProps
	}: WithElementRef<Omit<HTMLSelectAttributes, 'size'>> & {
		size?: NativeSelectSize;
	} = $props();
</script>

<select
	bind:this={ref}
	class={cn(nativeSelectVariants({ size }), className)}
	{...restProps}
>
	{@render children?.()}
</select>
