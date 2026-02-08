<script lang="ts">
import type { Snippet } from 'svelte';
import type { CaptureWithContext } from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';

interface Props {
	captures: CaptureWithContext[];
	/** Returns alt text for a capture's image. Defaults to shader_name. */
	alt?: (capture: CaptureWithContext) => string;
	/** Custom footer content rendered below the image in each card. Receives the capture. */
	footer?: Snippet<[CaptureWithContext]>;
}

let { captures, alt = (c) => c.shader_name, footer }: Props = $props();
</script>

<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
	{#each captures as capture (capture.id)}
		<a
			href="/admin/captures/{capture.id}"
			class="overflow-hidden rounded-lg border transition-colors hover:bg-muted/50"
		>
			{#if capture.image_url}
				<CaptureImage
					src={capture.image_url}
					thumbhash={capture.thumbhash}
					preset="card"
					alt={alt(capture)}
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
			{#if footer}
				{@render footer(capture)}
			{/if}
		</a>
	{/each}
</div>
