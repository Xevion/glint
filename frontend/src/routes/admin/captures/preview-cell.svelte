<script lang="ts">
import type { CaptureWithContext } from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';

interface Props {
	capture: CaptureWithContext;
}

let { capture }: Props = $props();
</script>

{#if capture.image_url ?? capture.thumbhash}
	<CaptureImage
		src={capture.image_url}
		thumbhash={capture.thumbhash}
		preset="thumbnail"
		alt="Capture preview"
		class="h-full w-full object-cover"
		containerClass="h-12 w-20 rounded"
	/>
{:else}
	<div
		class="flex h-12 w-20 items-center justify-center rounded bg-muted text-xs text-muted-foreground"
	>
		{capture.image_path ? 'No URL' : 'N/A'}
	</div>
{/if}
