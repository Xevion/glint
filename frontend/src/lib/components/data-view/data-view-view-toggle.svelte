<script lang="ts">
import { cn } from '$lib/utils';
import { VIEW_MODE_META, type ViewMode } from './types';

interface Props {
	modes?: ViewMode[];
	mode: ViewMode;
	/** Custom CSS class. */
	class?: string;
}

let { modes = ['grid', 'row'], mode = $bindable('grid'), class: className }: Props = $props();
</script>

<div class={cn('flex items-center gap-0.5 rounded-lg bg-muted p-0.5', className)}>
	{#each modes as m (m)}
		{@const config = VIEW_MODE_META[m]}
		<button
			type="button"
			aria-label={config.label}
			aria-pressed={mode === m}
			onclick={() => { mode = m; }}
			class={cn(
				'rounded-md p-1.5 transition-colors',
				mode === m
					? 'bg-background text-foreground shadow-sm'
					: 'text-muted-foreground hover:text-foreground'
			)}
		>
			<config.icon class="h-4 w-4" strokeWidth={2} />
		</button>
	{/each}
</div>
