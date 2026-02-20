<script lang="ts" module>
import { type VariantProps, tv } from 'tailwind-variants';

export const statusBadgeVariants = tv({
	base:
		'inline-flex w-fit shrink-0 items-center justify-center gap-1 overflow-hidden rounded-full border border-transparent px-2 py-0.5 text-xs font-medium whitespace-nowrap [&>svg]:pointer-events-none [&>svg]:size-3',
	variants: {
		status: {
			active: 'bg-success/15 text-success',
			inactive: 'bg-muted text-muted-foreground',
			pending: 'bg-warning/15 text-warning',
			error: 'bg-destructive/15 text-destructive',
			info: 'bg-info/15 text-info',
			// Capture statuses
			uploading: 'bg-info/15 text-info',
			// Run statuses
			running: 'bg-info/15 text-info',
			completed: 'bg-success/15 text-success',
			partial: 'bg-warning/15 text-warning',
			failed: 'bg-destructive/15 text-destructive',
			timed_out: 'bg-warning/15 text-warning',
			skipped: 'bg-muted text-muted-foreground/70',
			// Freshness statuses
			fresh: 'bg-success/15 text-success',
			stale: 'bg-warning/15 text-warning',
			superseded: 'bg-muted text-muted-foreground'
		}
	},
	defaultVariants: {
		status: 'inactive'
	}
});

export type StatusBadgeStatus = VariantProps<typeof statusBadgeVariants>['status'];
</script>

<script lang="ts">
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		status = "inactive",
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLSpanElement>> & {
		status?: StatusBadgeStatus;
	} = $props();
</script>

<span
	bind:this={ref}
	data-slot="status-badge"
	class={cn(statusBadgeVariants({ status }), className)}
	{...restProps}
>
	{@render children?.()}
</span>
