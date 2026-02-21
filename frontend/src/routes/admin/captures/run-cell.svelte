<script lang="ts">
import { StatusBadge, type StatusBadgeStatus } from '$lib/components/ui/status-badge';
import type { AdminCaptureNode } from './queries';

interface Props {
	capture: AdminCaptureNode;
}

let { capture }: Props = $props();
</script>

{#if capture.runId}
	<a
		href="/admin/runs/{capture.runId}"
		class="relative z-10 inline-flex items-center gap-1"
	>
		<StatusBadge status={(capture.runStatus?.toLowerCase() ?? 'inactive') as StatusBadgeStatus}>{capture.runStatus ?? '?'}</StatusBadge>
	</a>
{:else}
	<span class="text-muted-foreground">&mdash;</span>
{/if}
