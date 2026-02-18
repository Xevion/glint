<script lang="ts">
import type { CaptureWithContext } from '$lib/bindings';

interface Props {
	capture: CaptureWithContext;
}

let { capture }: Props = $props();
</script>

{#if capture.resolution_width && capture.resolution_height}
	{capture.resolution_width}x{capture.resolution_height}
{:else}
	<span class="text-muted-foreground">-</span>
{/if}
