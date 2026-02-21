<script lang="ts">
import { Button } from '$lib/components/ui/button';
import * as Popover from '$lib/components/ui/popover';
import { cn } from '$lib/utils';
import { X } from '@lucide/svelte';

interface Props {
	/** Display label (e.g., "File Size", "Duration"). */
	label: string;
	/** Unit label shown after values (e.g., "MB", "s"). */
	unit?: string;
	/** Current minimum value (bindable). Null = no minimum. */
	min: number | null;
	/** Current maximum value (bindable). Null = no maximum. */
	max: number | null;
	/** Scale factor: raw input is multiplied by this before storing.
	 *  E.g., if the filter is in bytes but you want the user to enter MB, set scale=1048576. */
	scale?: number;
	/** Minimum step for the input (e.g., 0.1 for decimal values). */
	step?: number;
	class?: string;
}

let {
	label,
	unit = '',
	min = $bindable(null),
	max = $bindable(null),
	scale = 1,
	step = 1,
	class: className
}: Props = $props();

let open = $state(false);

// Internal display values (pre-scale), synced from bound min/max props.
// Writable $derived: recomputes when min/max change, but allows user input to override.
let minInput = $derived(min !== null ? String(min / scale) : '');
let maxInput = $derived(max !== null ? String(max / scale) : '');

function apply() {
	const minVal = minInput ? Number(minInput) : null;
	const maxVal = maxInput ? Number(maxInput) : null;
	min = minVal !== null && !isNaN(minVal) ? minVal * scale : null;
	max = maxVal !== null && !isNaN(maxVal) ? maxVal * scale : null;
	open = false;
}

function clear() {
	min = null;
	max = null;
	minInput = '';
	maxInput = '';
}

const hasValue = $derived(min !== null || max !== null);

const displayLabel = $derived.by(() => {
	if (!hasValue) return label;
	const unitSuffix = unit ? ` ${unit}` : '';
	if (min !== null && max !== null) {
		return `${min / scale}–${max / scale}${unitSuffix}`;
	}
	if (min !== null) return `≥ ${min / scale}${unitSuffix}`;
	if (max !== null) return `≤ ${max / scale}${unitSuffix}`;
	return label;
});
</script>

<Popover.Root bind:open>
	<Popover.Trigger>
		{#snippet child({ props })}
			<Button
				{...props}
				variant="outline"
				size="sm"
				class={cn('h-8 gap-1.5 border-dashed font-normal', hasValue && 'border-primary/50', className)}
			>
				{displayLabel}
				{#if hasValue}
					<span
						role="presentation"
						class="ml-0.5 rounded-full p-0.5 hover:bg-accent"
						onclick={(e) => {
							e.stopPropagation();
							clear();
							open = false;
						}}
					>
						<X class="size-3" />
					</span>
				{/if}
			</Button>
		{/snippet}
	</Popover.Trigger>
	<Popover.Content class="w-52 p-2" align="start">
		<div class="flex flex-col gap-2">
			<label class="flex flex-col gap-1 text-xs font-medium">
				Min{unit ? ` (${unit})` : ''}
				<input
					type="number"
					class="rounded-md border bg-transparent px-2 py-1 text-sm"
					placeholder="No min"
					{step}
					bind:value={minInput}
				/>
			</label>
			<label class="flex flex-col gap-1 text-xs font-medium">
				Max{unit ? ` (${unit})` : ''}
				<input
					type="number"
					class="rounded-md border bg-transparent px-2 py-1 text-sm"
					placeholder="No max"
					{step}
					bind:value={maxInput}
				/>
			</label>
			<div class="flex gap-1">
				{#if hasValue}
					<Button
						variant="ghost"
						size="sm"
						class="h-7 flex-1 text-xs"
						onclick={() => {
							clear();
							open = false;
						}}
					>
						Clear
					</Button>
				{/if}
				<Button size="sm" class="h-7 flex-1 text-xs" onclick={apply}>Apply</Button>
			</div>
		</div>
	</Popover.Content>
</Popover.Root>
