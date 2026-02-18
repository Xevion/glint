<script lang="ts">
import type { CaptureRunStatus } from '$lib/bindings';
import { statusColorFallback, statusColors } from '$lib/utils/status';

interface Props {
	status: CaptureRunStatus;
}

let { status }: Props = $props();
</script>

<span
	class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {statusColors[status] ?? statusColorFallback}"
>
	{status}
</span>
