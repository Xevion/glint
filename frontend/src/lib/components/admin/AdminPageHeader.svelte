<script lang="ts">
import type { Snippet } from 'svelte';
import RefreshButton from '$lib/components/RefreshButton.svelte';

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
			<span class="text-lg text-foreground">{count}</span>
		{/if}
	</div>
	<div class="flex items-center gap-2">
		{#if actions}
			{@render actions()}
		{/if}
		{#if onrefresh}
			<RefreshButton {refreshing} onclick={onrefresh} />
		{/if}
	</div>
</header>
