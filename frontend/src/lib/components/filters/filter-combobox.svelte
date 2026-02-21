<script lang="ts">
import { Button } from '$lib/components/ui/button';
import * as Command from '$lib/components/ui/command';
import * as Popover from '$lib/components/ui/popover';
import { cn } from '$lib/utils';
import { Check, ChevronDown, X } from '@lucide/svelte';

interface Props {
	/** Display label when nothing is selected (e.g., "Shader"). */
	placeholder: string;
	/** Available options to select from. */
	options: { value: string; label: string }[];
	/** Currently selected value (bindable). Null = no filter. */
	value: string | null;
	class?: string;
}

let { placeholder, options, value = $bindable(null), class: className }: Props = $props();

let open = $state(false);

const selectedLabel = $derived(
	value ? (options.find((o) => o.value === value)?.label ?? value) : null
);

function select(optionValue: string) {
	value = value === optionValue ? null : optionValue;
	open = false;
}

function clear() {
	value = null;
}
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
					value && 'border-primary/50',
					className
				)}
			>
				{selectedLabel ?? placeholder}
				{#if value}
					<span
						role="presentation"
						class="ml-0.5 rounded-full p-0.5 hover:bg-accent"
						onclick={(e) => { e.stopPropagation(); clear(); }}
					>
						<X class="size-3" />
					</span>
				{:else}
					<ChevronDown class="size-3.5 opacity-50" />
				{/if}
			</Button>
		{/snippet}
	</Popover.Trigger>
	<Popover.Content
		class="w-56 p-0"
		align="start"
		onOpenAutoFocus={(e) => {
			e.preventDefault();
			// Explicitly focus the search input inside the command
			const input = (e.currentTarget as HTMLElement)?.querySelector('input');
			input?.focus();
		}}
	>
		<Command.Root>
			<Command.Input placeholder="Search..." />
			<Command.List>
				<Command.Empty>No results found.</Command.Empty>
				<Command.Group>
					{#each options as opt (opt.value)}
						<Command.Item value={opt.label} onSelect={() => select(opt.value)}>
							<Check
								class={cn('mr-2 size-4', value !== opt.value && 'text-transparent')}
							/>
							{opt.label}
						</Command.Item>
					{/each}
				</Command.Group>
			</Command.List>
		</Command.Root>
	</Popover.Content>
</Popover.Root>
