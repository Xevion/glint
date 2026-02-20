<script lang="ts">
import CaptureImage from '$lib/components/CaptureImage.svelte';
import type { Snippet } from 'svelte';

interface CaptureProps {
	id: string;
	image_path?: string;
	imagePath?: string;
	thumbhash?: string | null;
	shader_name?: string;
}

interface Props {
	capture: CaptureProps;
	/** Alt text for the image. Defaults to shader_name. */
	alt?: string;
	/** Content rendered below the image (metadata footer). */
	children?: Snippet;
}

let { capture, alt = capture.shader_name ?? '', children }: Props = $props();

let captureImagePath = $derived(capture.imagePath ?? capture.image_path);
</script>

<a
	href="/admin/captures/{capture.id}"
	class="block rounded-lg border bg-card transition-colors hover:bg-muted/50"
>
	<div class="overflow-hidden rounded-t-lg">
	{#if captureImagePath}
		<CaptureImage
			src={captureImagePath}
				thumbhash={capture.thumbhash}
				preset="card"
				{alt}
				class="w-full"
				containerClass="aspect-video w-full"
			/>
		{:else}
			<div
				class="flex aspect-video w-full items-center justify-center bg-muted text-xs text-muted-foreground"
			>
				No image
			</div>
		{/if}
	</div>
	{#if children}
		{@render children()}
	{/if}
</a>
