<script lang="ts">
import type { AdminCaptureNode } from './queries';

interface Props {
	capture: AdminCaptureNode;
}

let { capture }: Props = $props();
</script>

{#if capture.resolutionWidth && capture.resolutionHeight}
	{capture.resolutionWidth}x{capture.resolutionHeight}
{:else}
	<span class="text-muted-foreground">-</span>
{/if}
