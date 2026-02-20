<script lang="ts">
import { Slider } from '$lib/components/ui/slider';

interface Props {
	value: number;
	onchange?: (value: number) => void;
}
let { value = $bindable(), onchange }: Props = $props();

let sliderValue = $derived(Math.round(value * 100));

function handleValueChange(v: number) {
	const newValue = v / 100;
	if (newValue === value) return;
	value = newValue;
	onchange?.(newValue);
}

let displayPercent = $derived(`${Math.round(value * 100)}%`);
</script>

<div class="space-y-2">
	<div class="flex items-center justify-between">
		<span class="text-xs text-muted-foreground">Intensity</span>
		<span class="font-mono text-xs text-muted-foreground">{displayPercent}</span>
	</div>
	<Slider
		type="single"
		value={sliderValue}
		min={0}
		max={100}
		step={1}
		onValueChange={handleValueChange}
	/>
</div>
