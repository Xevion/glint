<script lang="ts">
import { Badge } from '$lib/components/ui/badge';
import { Button } from '$lib/components/ui/button';
import * as Checkbox from '$lib/components/ui/checkbox';
import * as Popover from '$lib/components/ui/popover';
import { cn } from '$lib/utils';
import { ChevronDown, X } from '@lucide/svelte';

interface Props {
	/** Display label for the filter trigger (e.g., "Status"). */
	label: string;
	/** Available options to select from. */
	options: { value: string; label: string }[];
	/** Currently selected values (bindable). Empty array or null = all / no filter. */
	value: string[] | null;
	class?: string;
}

let { label, options, value = $bindable([]), class: className }: Props = $props();

let open = $state(false);

/** Normalized value — treats null as empty array. */
const selected = $derived(value ?? []);

function toggle(optionValue: string) {
	if (selected.includes(optionValue)) {
		value = selected.filter((v) => v !== optionValue);
	} else {
		value = [...selected, optionValue];
	}
}

function clear() {
	value = [];
}

const triggerLabel = $derived(
	selected.length === 0
		? label
		: selected.length === 1
			? (options.find((o) => o.value === selected[0])?.label ?? selected[0])
			: label
);
</script>

<Popover.Root bind:open>
	<Popover.Trigger>
		{#snippet child({ props })}
			<Button
				{...props}
				variant="outline"
				size="sm"
				class={cn(
					'h-8 gap-1.5 border-dashed font-normal',
					selected.length > 0 && 'border-primary/50',
					className
				)}
			>
				{triggerLabel}
				{#if selected.length > 1}
					<Badge
						variant="secondary"
						class="h-4 min-w-4 rounded-full px-1 text-[10px] font-medium leading-none"
					>
						{selected.length}
					</Badge>
				{/if}
				<ChevronDown class="size-3.5 opacity-50" />
			</Button>
		{/snippet}
	</Popover.Trigger>
	<Popover.Content class="w-48 p-2" align="start">
		<div class="flex flex-col gap-0.5">
			{#each options as opt (opt.value)}
				<button
					type="button"
					class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm hover:bg-accent"
					onclick={() => toggle(opt.value)}
				>
				<Checkbox.Root
					checked={selected.includes(opt.value)}
					class="pointer-events-none"
				/>
					<span>{opt.label}</span>
				</button>
			{/each}
		</div>
		{#if selected.length > 0}
			<div class="mt-1 border-t pt-1">
				<button
					type="button"
					class="flex w-full items-center justify-center gap-1 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-foreground"
					onclick={clear}
				>
					<X class="size-3" />
					Clear
				</button>
			</div>
		{/if}
	</Popover.Content>
</Popover.Root>
