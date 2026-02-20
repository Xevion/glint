<script lang="ts">
import type { CaptureRunStatus } from '$lib/bindings';
import { StatusBadge } from '$lib/components/ui/status-badge';

interface Props {
	status: CaptureRunStatus;
}

let { status }: Props = $props();
</script>

<StatusBadge {status}>{status}</StatusBadge>
