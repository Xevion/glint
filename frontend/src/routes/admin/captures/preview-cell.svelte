<script lang="ts">
import CaptureImage from '$lib/components/CaptureImage.svelte';
import type { AdminCaptureNode } from './queries';

interface Props {
	capture: AdminCaptureNode;
}

let { capture }: Props = $props();
</script>

{#if capture.imagePath ?? capture.thumbhash}
	<CaptureImage
		src={capture.imagePath}
		thumbhash={capture.thumbhash}
		preset="thumbnail"
		alt="Capture preview"
		class="h-full w-full object-cover"
		containerClass="h-12 w-20 rounded pointer-events-none"
	/>
{:else}
	<div
		class="flex h-12 w-20 items-center justify-center rounded bg-muted text-xs text-muted-foreground"
	>
		{capture.imagePath ? 'No URL' : 'N/A'}
	</div>
{/if}
