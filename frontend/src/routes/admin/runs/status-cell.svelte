<script lang="ts">
import { StatusBadge, type StatusBadgeStatus } from '$lib/components/ui/status-badge';

interface Props {
	status: string;
}

let { status }: Props = $props();

const badgeStatus = $derived(status.toLowerCase() as StatusBadgeStatus);
const displayLabel = $derived(status.toLowerCase().replace(/_/g, ' '));
</script>

<StatusBadge status={badgeStatus}>{displayLabel}</StatusBadge>
