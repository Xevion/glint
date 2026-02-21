<script lang="ts">
import { formatBytes } from '$lib/utils/format';
import type { AdminCaptureNode } from './queries';

interface Props {
	capture: AdminCaptureNode;
}

let { capture }: Props = $props();
</script>

{#if capture.fileSizeBytes}
	{formatBytes(capture.fileSizeBytes)}
{:else}
	<span class="text-muted-foreground">-</span>
{/if}
