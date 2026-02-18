<script lang="ts">
import type { CaptureWithContext } from '$lib/bindings';
import { freshnessColors } from '$lib/utils/status';

interface Props {
	capture: CaptureWithContext;
}

let { capture }: Props = $props();
</script>

<span
	class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {freshnessColors[capture.freshness]}"
>
	{capture.freshness}
</span>
