<script lang="ts">
import { Slider } from '$lib/components/ui/slider';
import { formatTimeTicks } from '$lib/utils/format';

interface Props {
	value: number;
	onchange?: (value: number) => void;
}
let { value = $bindable(), onchange }: Props = $props();

let sliderValue = $derived([value]);

function handleValueChange(v: number[]) {
	if (v[0] === value) return;
	value = v[0];
	onchange?.(v[0]);
}

const MARKERS = [
	{ ticks: 0, label: 'Dawn' },
	{ ticks: 6000, label: 'Noon' },
	{ ticks: 12000, label: 'Dusk' },
	{ ticks: 18000, label: 'Midnight' }
] as const;

let displayTime = $derived(formatTimeTicks(value));
</script>

<div class="space-y-2">
	<div class="flex items-center justify-between">
		<span class="text-sm font-medium text-muted-foreground">{displayTime}</span>
		<span class="font-mono text-xs text-muted-foreground">{value}t</span>
	</div>
	<Slider
		type="single"
		value={sliderValue}
		min={0}
		max={24000}
		step={100}
		onValueChange={handleValueChange}
	/>
	<div class="relative h-4">
		{#each MARKERS as marker (marker.ticks)}
			<span
				class="absolute -translate-x-1/2 text-[10px] text-muted-foreground"
				style="left: {(marker.ticks / 24000) * 100}%"
			>
				{marker.label}
			</span>
		{/each}
	</div>
</div>
