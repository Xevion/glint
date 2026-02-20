<script lang="ts">
import type { CaptureWithContext } from '$lib/bindings';
import { StatusBadge } from '$lib/components/ui/status-badge';

interface Props {
	capture: CaptureWithContext;
}

let { capture }: Props = $props();
</script>

{#if capture.run_id}
	<a
		href="/admin/runs/{capture.run_id}"
		class="relative z-10 inline-flex items-center gap-1"
	>
		<StatusBadge status={capture.run_status ?? 'inactive'}>{capture.run_status ?? '?'}</StatusBadge>
	</a>
{:else}
	<span class="text-muted-foreground">&mdash;</span>
{/if}
