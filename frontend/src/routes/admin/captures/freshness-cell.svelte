<script lang="ts">
import { StatusBadge, type StatusBadgeStatus } from '$lib/components/ui/status-badge';
import type { AdminCaptureNode } from './queries';

interface Props {
	capture: AdminCaptureNode;
}

let { capture }: Props = $props();
</script>

<StatusBadge status={capture.freshness.toLowerCase() as StatusBadgeStatus}>{capture.freshness}</StatusBadge>
