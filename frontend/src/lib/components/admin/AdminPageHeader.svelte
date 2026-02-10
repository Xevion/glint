<script lang="ts">
import type { Snippet } from 'svelte';
import { Button } from '$lib/components/ui/button';
import { RefreshCw } from '@lucide/svelte';

interface Props {
	title: string;
	count?: number;
	refreshing?: boolean;
	onrefresh?: () => void;
	actions?: Snippet;
}
let { title, count, refreshing = false, onrefresh, actions }: Props = $props();
</script>

<header class="flex items-center justify-between">
	<div class="flex items-baseline gap-3">
		<h1 class="text-2xl font-semibold">{title}</h1>
		{#if count !== undefined}
			<span class="text-lg text-muted-foreground">{count}</span>
		{/if}
	</div>
	<div class="flex items-center gap-2">
		{#if actions}
			{@render actions()}
		{/if}
		{#if onrefresh}
			<Button variant="outline" size="icon" onclick={onrefresh} disabled={refreshing}>
				<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
			</Button>
		{/if}
	</div>
</header>
