<script lang="ts">
import type { CaptureWithContext } from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { ItemGrid } from '$lib/components/item-grid';
import { ImageOverlay } from '$lib/components/ui/image-overlay';
import { ImageOff } from '@lucide/svelte';
import type { Snippet } from 'svelte';
import { fade } from 'svelte/transition';

interface Props {
	captures: CaptureWithContext[];
	title: string;
	/** Text shown in the empty state below "No Captures Yet" */
	emptyMessage?: string;
	/** Custom overlay content rendered over each capture on hover. Receives the capture. */
	overlay?: Snippet<[CaptureWithContext]>;
	/** Called when a capture is clicked */
	onclick?: (capture: CaptureWithContext) => void;
	/** Alt text for each capture image. Defaults to "Capture". */
	alt?: (capture: CaptureWithContext) => string;
	/** Whether more items are available to load (infinite scroll) */
	hasMore?: boolean;
	/** Whether more items are currently being loaded */
	loading?: boolean;
	/** Called when the scroll sentinel becomes visible */
	onLoadMore?: () => void;
}

let {
	captures,
	title,
	emptyMessage = 'Captures are being generated.',
	overlay,
	onclick,
	alt,
	hasMore = false,
	loading = false,
	onLoadMore
}: Props = $props();
</script>

<div in:fade|local={{ duration: 300, delay: 200 }} class="mb-8">
	{#if captures.length > 0}
		<h2 class="mb-4 text-2xl font-bold text-foreground">{title}</h2>
	{/if}

	<ItemGrid
		items={captures}
		key={(capture: CaptureWithContext) => capture.id}
		mode="card"
		empty={{ icon: ImageOff, title: 'No Captures Yet', message: emptyMessage }}
		{hasMore}
		{loading}
		{onLoadMore}
	>
		{#snippet card(capture: CaptureWithContext)}
			<button
				type="button"
				onclick={() => onclick?.(capture)}
				class="group relative overflow-hidden rounded-xl transition-transform hover:scale-[1.02]"
			>
				<CaptureImage
					src={capture.image_url}
					thumbhash={capture.thumbhash}
					preset="card"
					alt={alt ? alt(capture) : 'Capture'}
					class="h-full w-full object-cover"
					containerClass="aspect-video"
				/>
				<ImageOverlay hover />
				<div
					class="absolute right-0 bottom-0 left-0 p-3 opacity-0 transition-opacity group-hover:opacity-100"
				>
					{#if overlay}
						{@render overlay(capture)}
					{/if}
				</div>
			</button>
		{/snippet}
	</ItemGrid>
</div>
