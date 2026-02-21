<script lang="ts">
import {
	StatusBadge,
	statusBadgeVariants,
	type StatusBadgeStatus
} from '$lib/components/ui/status-badge';

interface Props {
	status: string;
}

let { status }: Props = $props();

const VALID_STATUSES = new Set(Object.keys(statusBadgeVariants.variants?.status ?? {}));
const badgeStatus = $derived.by((): StatusBadgeStatus => {
	const lower = status.toLowerCase();
	return VALID_STATUSES.has(lower) ? (lower as StatusBadgeStatus) : 'inactive';
});
const displayLabel = $derived(status.toLowerCase().replace(/_/g, ' '));
</script>

<StatusBadge status={badgeStatus}>{displayLabel}</StatusBadge>
