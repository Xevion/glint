<script lang="ts">
import type { CaptureWithContext } from '$lib/bindings';
import { StatusBadge } from '$lib/components/ui/status-badge';

interface Props {
	capture: CaptureWithContext;
}

let { capture }: Props = $props();
</script>

<StatusBadge status={capture.freshness}>{capture.freshness}</StatusBadge>
