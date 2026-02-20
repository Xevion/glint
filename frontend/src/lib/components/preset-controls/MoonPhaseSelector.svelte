<script lang="ts">
import { cn } from '$lib/utils';

interface Props {
	value: number | undefined;
	onchange?: (value: number | undefined) => void;
}
let { value = $bindable(), onchange }: Props = $props();

const PHASES = [
	{ phase: 0, emoji: '\u{1F315}', name: 'Full Moon' },
	{ phase: 1, emoji: '\u{1F316}', name: 'Waning Gibbous' },
	{ phase: 2, emoji: '\u{1F317}', name: 'Third Quarter' },
	{ phase: 3, emoji: '\u{1F318}', name: 'Waning Crescent' },
	{ phase: 4, emoji: '\u{1F311}', name: 'New Moon' },
	{ phase: 5, emoji: '\u{1F312}', name: 'Waxing Crescent' },
	{ phase: 6, emoji: '\u{1F313}', name: 'First Quarter' },
	{ phase: 7, emoji: '\u{1F314}', name: 'Waxing Gibbous' }
] as const;

function select(phase: number) {
	if (value === phase) {
		value = undefined;
		onchange?.(undefined);
	} else {
		value = phase;
		onchange?.(phase);
	}
}

let selectedName = $derived(value != null ? (PHASES[value]?.name ?? 'Unknown') : 'None (auto)');
</script>

<div class="space-y-2">
	<div class="flex items-center justify-between">
		<span class="text-xs text-muted-foreground">Moon Phase</span>
		<span class="text-xs text-muted-foreground">{selectedName}</span>
	</div>
	<div class="grid grid-cols-8 gap-1">
		{#each PHASES as phase (phase.phase)}
			<button
				type="button"
				title={phase.name}
				onclick={() => select(phase.phase)}
				class={cn(
					'flex aspect-square items-center justify-center rounded-md border text-lg transition-colors',
					value === phase.phase
						? 'border-primary bg-primary/10 ring-1 ring-primary'
						: 'border-border hover:bg-muted/50'
				)}
			>
				{phase.emoji}
			</button>
		{/each}
	</div>
</div>
