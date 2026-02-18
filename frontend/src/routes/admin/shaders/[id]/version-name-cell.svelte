<script lang="ts">
import { Badge } from '$lib/components/ui/badge';

interface Props {
	version: string;
	isLatest: boolean;
}

let { version, isLatest }: Props = $props();
</script>

<div class="flex items-center gap-1.5">
	<span class="font-medium">{version}</span>
	{#if isLatest}
		<Badge variant="outline" class="px-1 py-0 text-[10px]">latest</Badge>
	{/if}
</div>
