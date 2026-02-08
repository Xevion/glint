<script lang="ts">
import type { Snippet } from 'svelte';
import type { CaptureWithContext } from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { ImageOff } from '@lucide/svelte';
import { fade, scale } from 'svelte/transition';

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
}

let {
	captures,
	title,
	emptyMessage = 'Captures are being generated.',
	overlay,
	onclick,
	alt
}: Props = $props();
</script>

{#if captures.length > 0}
	<div in:fade|local={{ duration: 300, delay: 200 }} class="mb-8">
		<h2 class="mb-4 text-2xl font-bold text-foreground">{title}</h2>
		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
			{#each captures as capture, i (capture.id)}
				<button
					type="button"
					onclick={() => onclick?.(capture)}
					in:scale|local={{ duration: 350, delay: Math.min(i * 50, 400) + 250, start: 0.95 }}
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
					<div
						class="absolute inset-0 bg-linear-to-t from-black/60 to-transparent opacity-0 transition-opacity group-hover:opacity-100"
					>
						<div class="absolute right-0 bottom-0 left-0 p-3">
							{#if overlay}
								{@render overlay(capture)}
							{/if}
						</div>
					</div>
				</button>
			{/each}
		</div>
	</div>
{:else}
	<div
		in:fade|local={{ duration: 300, delay: 200 }}
		class="mb-8 rounded-xl border border-border bg-card p-12 text-center"
	>
		<ImageOff class="mx-auto mb-4 h-16 w-16 text-muted-foreground opacity-30" strokeWidth={1.5} />
		<h3 class="mb-2 text-lg font-semibold text-card-foreground">No Captures Yet</h3>
		<p class="text-sm text-muted-foreground">{emptyMessage}</p>
	</div>
{/if}
