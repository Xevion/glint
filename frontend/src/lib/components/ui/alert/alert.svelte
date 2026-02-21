<script lang="ts" module>
import { type VariantProps, tv } from 'tailwind-variants';

export const alertVariants = tv({
	base:
		'relative grid w-full grid-cols-[0_1fr] items-start gap-y-0.5 rounded-lg border px-4 py-3 text-sm has-[>svg]:grid-cols-[calc(var(--spacing)*4)_1fr] has-[>svg]:gap-x-3 [&>svg]:size-4 [&>svg]:translate-y-0.5 [&>svg]:text-current',
	variants: {
		variant: {
			default: 'bg-card text-card-foreground',
			destructive:
				'border-destructive/50 text-destructive bg-destructive/10 dark:border-destructive *:data-[slot=alert-description]:text-destructive/90 [&>svg]:text-current',
			success:
				'border-success/50 text-success bg-success/10 dark:border-success *:data-[slot=alert-description]:text-success/90 [&>svg]:text-current',
			warning:
				'border-warning/50 text-warning bg-warning/10 dark:border-warning *:data-[slot=alert-description]:text-warning/90 [&>svg]:text-current',
			info:
				'border-info/50 text-info bg-info/10 dark:border-info *:data-[slot=alert-description]:text-info/90 [&>svg]:text-current'
		}
	},
	defaultVariants: {
		variant: 'default'
	}
});

export type AlertVariant = VariantProps<typeof alertVariants>['variant'];
</script>

<script lang="ts">
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils";

	let {
		ref = $bindable(null),
		class: className,
		variant = "default",
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & {
		variant?: AlertVariant;
	} = $props();
</script>

<div
	bind:this={ref}
	data-slot="alert"
	role="alert"
	class={cn(alertVariants({ variant }), className)}
	{...restProps}
>
	{@render children?.()}
</div>
