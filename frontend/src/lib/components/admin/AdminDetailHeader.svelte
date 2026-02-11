<script lang="ts">
import { ArrowLeft } from '@lucide/svelte';
import type { Snippet } from 'svelte';

interface Props {
	backHref: string;
	backLabel?: string;
	title: string;
	trailing?: Snippet;
}
let { backHref, backLabel = 'Go back', title, trailing }: Props = $props();
</script>

<header class="space-y-2">
	<div class="flex items-center gap-2">
		<a href={backHref} class="text-muted-foreground hover:text-foreground" aria-label={backLabel}>
			<ArrowLeft class="h-4 w-4" />
		</a>
		<h1 class="text-2xl font-semibold">{title}</h1>
		{#if trailing}
			{@render trailing()}
		{/if}
	</div>
</header>
