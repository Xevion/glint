<script lang="ts">
import { getDataViewContext } from './context';

interface Props {
	/** Singular noun label, e.g. "capture". Pluralized automatically when count != 1. */
	noun?: string;
	class?: string;
}

let { noun = 'item', class: className }: Props = $props();

const ctx = getDataViewContext<unknown>();
const label = $derived(ctx.items.length === 1 ? noun : `${noun}s`);
</script>

{#if ctx.items.length > 0 && ctx.totalCount != null}
	<p class="mt-2 pl-2 text-xs text-muted-foreground {className ?? ''}">
		Showing {ctx.items.length} of {ctx.totalCount}
		{label}
	</p>
{/if}
