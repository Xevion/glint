<script lang="ts">
import { NativeSelect } from '$lib/components/ui/native-select';
import { cn } from '$lib/utils';
import type { SortOption } from './types';

interface Props {
	options: SortOption[];
	value: string;
	disabled?: boolean;
	/** Custom CSS class. */
	class?: string;
}

let { options, value = $bindable(''), disabled = false, class: className }: Props = $props();

function handleChange(e: Event) {
	value = (e.target as HTMLSelectElement).value;
}
</script>

<NativeSelect
	{value}
	onchange={handleChange}
	{disabled}
	class={cn(className)}
>
	{#each options as opt (opt.value)}
		<option value={opt.value}>{opt.label}</option>
	{/each}
</NativeSelect>
