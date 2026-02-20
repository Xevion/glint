<script lang="ts">
import type { Snippet } from 'svelte';

interface Props {
	label: string;
	children: Snippet;
}

let { label, children }: Props = $props();
</script>

<div class="flex items-baseline gap-2">
	<dt class="shrink-0 text-sm font-medium text-muted-foreground">{label}</dt>
	<dd class="text-sm">
		{@render children()}
	</dd>
</div>
