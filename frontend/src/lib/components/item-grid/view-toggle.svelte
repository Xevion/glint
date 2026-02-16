<script lang="ts">
import { cn } from '$lib/utils';
import { LayoutGrid, List } from '@lucide/svelte';
import type { GridMode } from './types';

interface Props {
	mode: GridMode;
	onchange?: (mode: GridMode) => void;
	class?: string;
}

let { mode, onchange, class: className }: Props = $props();

const modes = [
	{ value: 'card' as const, icon: LayoutGrid, label: 'Grid view' },
	{ value: 'row' as const, icon: List, label: 'List view' }
] as const;
</script>

<div class={cn('flex items-center gap-0.5 rounded-lg bg-muted p-0.5', className)}>
	{#each modes as m (m.value)}
		<button
			type="button"
			aria-label={m.label}
			aria-pressed={mode === m.value}
			onclick={() => onchange?.(m.value)}
			class={cn(
				'rounded-md p-1.5 transition-colors',
				mode === m.value
					? 'bg-background text-foreground shadow-sm'
					: 'text-muted-foreground hover:text-foreground'
			)}
		>
			<m.icon class="h-4 w-4" strokeWidth={2} />
		</button>
	{/each}
</div>
