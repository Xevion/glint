<script lang="ts">
import type { Snippet } from 'svelte';

interface Props {
	label: string;
	children: Snippet;
}

let { label, children }: Props = $props();
</script>

<div class="space-y-1">
	<dt class="text-sm font-medium text-muted-foreground">{label}</dt>
	<dd class="text-sm">
		{@render children()}
	</dd>
</div>
