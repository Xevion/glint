<script lang="ts">
import * as ToggleGroup from '$lib/components/ui/toggle-group';
import { Cloud, CloudLightning, CloudRain, Snowflake, Sun } from '@lucide/svelte';

type Weather = 'clear' | 'rain' | 'thunder' | 'snow';

interface Props {
	value: Weather;
	onchange?: (value: Weather) => void;
}
let { value = $bindable(), onchange }: Props = $props();

function handleValueChange(v: string) {
	if (v && v !== value) {
		value = v as Weather;
		onchange?.(v as Weather);
	}
}

const OPTIONS = [
	{ value: 'clear' as const, label: 'Clear', icon: Sun },
	{ value: 'rain' as const, label: 'Rain', icon: CloudRain },
	{ value: 'thunder' as const, label: 'Thunder', icon: CloudLightning },
	{ value: 'snow' as const, label: 'Snow', icon: Snowflake }
];
</script>

<ToggleGroup.Root
	type="single"
	value={value}
	onValueChange={handleValueChange}
	variant="outline"
	class="w-full"
>
	{#each OPTIONS as opt (opt.value)}
		<ToggleGroup.Item value={opt.value} class="flex-1 gap-1.5 text-xs">
			<opt.icon class="h-3.5 w-3.5" />
			{opt.label}
		</ToggleGroup.Item>
	{/each}
</ToggleGroup.Root>
