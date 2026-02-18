<script lang="ts">
import type { CaptureWithContext } from '$lib/bindings';
import { formatBytes } from '$lib/utils/format';

interface Props {
	capture: CaptureWithContext;
}

let { capture }: Props = $props();
</script>

{#if capture.file_size_bytes}
	{formatBytes(capture.file_size_bytes)}
{:else}
	<span class="text-muted-foreground">-</span>
{/if}
