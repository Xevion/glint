<script lang="ts">
import { Input } from '$lib/components/ui/input';
import { cn } from '$lib/utils';
import { Search, X } from '@lucide/svelte';

interface Props {
	value: string;
	placeholder?: string;
	disabled?: boolean;
	onInput?: (value: string) => void;
	onClear?: () => void;
	/** Custom CSS class for the outer container. */
	class?: string;
}

let {
	value = $bindable(''),
	placeholder = 'Search...',
	disabled = false,
	onInput,
	onClear,
	class: className
}: Props = $props();

function handleInput(e: Event) {
	const val = (e.target as HTMLInputElement).value;
	value = val;
	onInput?.(val);
}

function handleClear() {
	value = '';
	onClear?.();
	onInput?.('');
}
</script>

<div class={cn('relative min-w-0 flex-1 sm:min-w-48 sm:flex-initial', className)}>
	<Search
		class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground"
		strokeWidth={2}
	/>
	<Input
		type="text"
		{placeholder}
		{value}
		oninput={handleInput}
		{disabled}
		class="w-full pr-8 pl-9 sm:w-48 sm:focus:w-64"
	/>
	{#if value}
		<button
			class="absolute top-1/2 right-2 -translate-y-1/2 rounded-sm p-0.5 text-muted-foreground hover:text-foreground"
			onclick={handleClear}
			aria-label="Clear search"
		>
			<X class="h-3.5 w-3.5" />
		</button>
	{/if}
</div>
