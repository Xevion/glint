<script lang="ts">
import type { CaptureWithContext } from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import type { Snippet } from 'svelte';

interface Props {
	capture: CaptureWithContext;
	/** Alt text for the image. Defaults to shader_name. */
	alt?: string;
	/** Content rendered below the image (metadata footer). */
	children?: Snippet;
}

let { capture, alt = capture.shader_name, children }: Props = $props();
</script>

<a
	href="/admin/captures/{capture.id}"
	class="overflow-hidden rounded-lg border transition-colors hover:bg-muted/50"
>
	{#if capture.image_url}
		<CaptureImage
			src={capture.image_url}
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
	{#if children}
		{@render children()}
	{/if}
</a>
