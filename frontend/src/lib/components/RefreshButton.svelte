<script lang="ts">
import { untrack } from 'svelte';
import { Button } from '$lib/components/ui/button';
import { Check, RefreshCw } from '@lucide/svelte';

interface Props {
	refreshing: boolean;
	onclick: () => void;
}

let { refreshing, onclick }: Props = $props();

type State = 'idle' | 'refreshing' | 'done';
let visual = $state<State>('idle');

// Detect refreshing prop transitions
$effect(() => {
	if (refreshing) {
		visual = 'refreshing';
	} else if (untrack(() => visual) === 'refreshing') {
		visual = 'done';
	}
});

// Auto-revert done → idle after 1.5s
$effect(() => {
	if (visual === 'done') {
		const timer = setTimeout(() => {
			visual = 'idle';
		}, 1500);
		return () => clearTimeout(timer);
	}
});
</script>

<Button variant="outline" size="icon" {onclick} disabled={visual !== 'idle'}>
	{#if visual === 'done'}
		<Check class="h-4 w-4 text-success" />
	{:else}
		<RefreshCw class="h-4 w-4 {visual === 'refreshing' ? 'animate-spin' : ''}" />
	{/if}
</Button>
