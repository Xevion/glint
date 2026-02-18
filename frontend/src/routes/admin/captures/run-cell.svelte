<script lang="ts">
import type { CaptureWithContext } from '$lib/bindings';
import { statusColorFallback, statusColors } from '$lib/utils/status';

interface Props {
	capture: CaptureWithContext;
}

let { capture }: Props = $props();
</script>

{#if capture.run_id}
	<a
		href="/admin/runs/{capture.run_id}"
		class="inline-flex items-center gap-1"
		onclick={(e) => e.stopPropagation()}
	>
		<span
			class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {statusColors[capture.run_status ?? ''] ?? statusColorFallback}"
		>
			{capture.run_status ?? '?'}
		</span>
	</a>
{:else}
	<span class="text-muted-foreground">&mdash;</span>
{/if}
