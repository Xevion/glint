<script lang="ts">
import { NativeSelect } from '$lib/components/ui/native-select';

interface Props {
	options: { value: string; label: string }[];
	value: string | null;
	placeholder?: string;
	/** Callback when the selected value changes. */
	onchange?: (value: string | null) => void;
	class?: string;
}
let {
	options,
	value = $bindable(null),
	placeholder = 'All',
	onchange,
	class: className
}: Props = $props();

function handleChange(e: Event & { currentTarget: HTMLSelectElement }) {
	const newValue = e.currentTarget.value || null;
	value = newValue;
	onchange?.(newValue);
}
</script>

<NativeSelect size="sm" value={value ?? ''} onchange={handleChange} class={className}>
	<option value="">{placeholder}</option>
	{#each options as opt (opt.value)}
		<option value={opt.value}>{opt.label}</option>
	{/each}
</NativeSelect>
