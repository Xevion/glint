<script lang="ts">
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { Button } from '$lib/components/ui/button';
import { ChevronLeft, ChevronRight, X } from '@lucide/svelte';
import { fade, scale } from 'svelte/transition';

interface CaptureItem {
	id: string;
	image_url: string | null;
	thumbhash?: string | null;
	profile?: string | null;
	shader_version?: string | null;
	scene_id?: string;
}

interface Props {
	captures: CaptureItem[];
	currentIndex: number;
	onClose: () => void;
	onNavigate: (index: number) => void;
}

let { captures, currentIndex, onClose, onNavigate }: Props = $props();

const currentCapture = $derived(captures[currentIndex]);
const hasPrev = $derived(currentIndex > 0);
const hasNext = $derived(currentIndex < captures.length - 1);

function handleKeydown(e: KeyboardEvent) {
	switch (e.key) {
		case 'Escape':
			onClose();
			break;
		case 'ArrowLeft':
			if (hasPrev) onNavigate(currentIndex - 1);
			break;
		case 'ArrowRight':
			if (hasNext) onNavigate(currentIndex + 1);
			break;
	}
}

function handleBackdropClick(e: MouseEvent) {
	if (e.target === e.currentTarget) {
		onClose();
	}
}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Backdrop -->
<div
	transition:fade={{ duration: 200 }}
	class="fixed inset-0 z-50 flex items-center justify-center bg-black/90 backdrop-blur-sm"
	onclick={handleBackdropClick}
	onkeydown={(e) => e.key === 'Escape' && onClose()}
	role="dialog"
	aria-modal="true"
	tabindex="-1"
>
	<!-- Close button -->
	<Button
		variant="ghost"
		size="icon"
		class="absolute top-4 right-4 z-10 text-white hover:bg-white/10"
		onclick={onClose}
	>
		<X class="h-6 w-6" />
	</Button>

	<!-- Navigation buttons -->
	{#if hasPrev}
		<Button
			variant="ghost"
			size="icon"
			class="absolute left-4 z-10 text-white hover:bg-white/10"
			onclick={() => onNavigate(currentIndex - 1)}
		>
			<ChevronLeft class="h-8 w-8" />
		</Button>
	{/if}

	{#if hasNext}
		<Button
			variant="ghost"
			size="icon"
			class="absolute right-4 z-10 text-white hover:bg-white/10"
			onclick={() => onNavigate(currentIndex + 1)}
		>
			<ChevronRight class="h-8 w-8" />
		</Button>
	{/if}

	<!-- Main image -->
	{#if currentCapture?.image_url}
		{#key currentCapture.id}
			<div
				transition:scale={{ duration: 200, start: 0.95 }}
				class="relative max-h-[90vh] max-w-[90vw]"
			>
				<CaptureImage
					src={currentCapture.image_url}
					thumbhash={currentCapture.thumbhash}
					preset="full"
					priority
					alt="Capture fullscreen view"
					class="max-h-[90vh] max-w-[90vw] object-contain"
					containerClass=""
				/>

				<!-- Capture info overlay -->
				<div class="absolute right-0 bottom-0 left-0 bg-gradient-to-t from-black/80 to-transparent p-4">
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-2">
							{#if currentCapture.profile}
								<span class="rounded bg-primary px-2 py-1 text-sm font-medium text-white">
									{currentCapture.profile}
								</span>
							{/if}
							{#if currentCapture.shader_version}
								<span class="rounded bg-white/20 px-2 py-1 text-sm font-medium text-white">
									v{currentCapture.shader_version}
								</span>
							{/if}
						</div>
						<span class="text-sm text-white/70">
							{currentIndex + 1} / {captures.length}
						</span>
					</div>
				</div>
			</div>
		{/key}
	{/if}
</div>
