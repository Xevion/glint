<script lang="ts">
import { Button } from '$lib/components/ui/button';
import { cfImageUrl } from '$lib/utils/image';
import { decodeThumbhash } from '$lib/utils/thumbhash';
import { ChevronLeft, ChevronRight, ImageOff, X } from '@lucide/svelte';
import { fade, fly } from 'svelte/transition';

interface CaptureItem {
	id: string;
	image_url?: string | null;
	thumbhash?: string | null;
	profile_name?: string | null;
	shader_version?: string | null;
	scene_id?: string;
	scene_name?: string | null;
}

interface Props {
	captures: CaptureItem[];
	currentIndex: number;
	onClose: () => void;
	onNavigate: (index: number) => void;
}

let { captures, currentIndex, onClose, onNavigate }: Props = $props();

// Track navigation direction for slide animation
let direction = $state<'left' | 'right'>('left');
let prevIndex: number = $state(0);
$effect(() => {
	// Runs on mount (syncing prevIndex) and on every navigation
	if (currentIndex !== prevIndex) {
		direction = currentIndex > prevIndex ? 'left' : 'right';
		prevIndex = currentIndex;
	}
});

const currentCapture = $derived(captures[currentIndex]);
const hasPrev = $derived(currentIndex > 0);
const hasNext = $derived(currentIndex < captures.length - 1);

// Resolved image URL — single source, no srcset duplication
const fullImageUrl = $derived(cfImageUrl(currentCapture?.image_url, 'full'));

// Thumbhash placeholder for current image
const placeholderUrl = $derived(decodeThumbhash(currentCapture?.thumbhash));

// Zoom and pan state
let zoomLevel = $state(1);
let panOffset = $state({ x: 0, y: 0 });
let isDragging = $state(false);
let dragStart = $state({ x: 0, y: 0 });
let panStart = $state({ x: 0, y: 0 });
let imageLoaded = $state(false);
let imageErrored = $state(false);
let imgEl = $state<HTMLImageElement | null>(null);

// Reset zoom and loaded state when navigating to a different image
$effect(() => {
	void currentIndex;
	zoomLevel = 1;
	panOffset = { x: 0, y: 0 };
	imageLoaded = false;
	imageErrored = false;
});

// Handle images already cached by the browser
$effect(() => {
	if (imgEl && !imageLoaded && imgEl.complete && imgEl.naturalWidth > 0) {
		imageLoaded = true;
	}
});

// Preload adjacent images only after current image finishes loading
$effect(() => {
	if (!imageLoaded) return;
	const preloadIndices = [currentIndex - 1, currentIndex + 1].filter(
		(i) => i >= 0 && i < captures.length
	);
	for (const i of preloadIndices) {
		const url = captures[i]?.image_url;
		if (url) {
			const img = new Image();
			img.src = cfImageUrl(url, 'full') ?? '';
		}
	}
});

function handleKeydown(e: KeyboardEvent) {
	switch (e.key) {
		case 'Escape':
			if (zoomLevel > 1) {
				zoomLevel = 1;
				panOffset = { x: 0, y: 0 };
			} else {
				onClose();
			}
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
		if (zoomLevel > 1) {
			zoomLevel = 1;
			panOffset = { x: 0, y: 0 };
		} else {
			onClose();
		}
	}
}

function handleWheel(e: WheelEvent) {
	e.preventDefault();
	const delta = e.deltaY > 0 ? -0.25 : 0.25;
	zoomLevel = Math.min(5, Math.max(1, zoomLevel + delta));
	if (zoomLevel === 1) {
		panOffset = { x: 0, y: 0 };
	}
}

function handleDblClick() {
	if (zoomLevel === 1) {
		zoomLevel = 2;
	} else {
		zoomLevel = 1;
		panOffset = { x: 0, y: 0 };
	}
}

function handlePointerDown(e: PointerEvent) {
	if (zoomLevel <= 1) return;
	isDragging = true;
	dragStart = { x: e.clientX, y: e.clientY };
	panStart = { x: panOffset.x, y: panOffset.y };
	(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}

function handlePointerMove(e: PointerEvent) {
	if (!isDragging) return;
	panOffset = {
		x: panStart.x + (e.clientX - dragStart.x) / zoomLevel,
		y: panStart.y + (e.clientY - dragStart.y) / zoomLevel
	};
}

function handlePointerUp() {
	isDragging = false;
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
			class="absolute left-4 z-10 cursor-pointer text-white hover:bg-white/10"
			onclick={() => onNavigate(currentIndex - 1)}
		>
			<ChevronLeft class="h-8 w-8" />
		</Button>
	{/if}

	{#if hasNext}
		<Button
			variant="ghost"
			size="icon"
			class="absolute right-4 z-10 cursor-pointer text-white hover:bg-white/10"
			onclick={() => onNavigate(currentIndex + 1)}
		>
			<ChevronRight class="h-8 w-8" />
		</Button>
	{/if}

	<!-- Main image -->
	{#if currentCapture?.image_url}
		<div
			class="relative h-[90vh] w-[90vw] select-none overflow-hidden"
			class:cursor-grab={zoomLevel > 1 && !isDragging}
			class:cursor-grabbing={isDragging}
			style:touch-action="none"
			role="img"
			tabindex="-1"
			onwheel={handleWheel}
			ondblclick={handleDblClick}
			onpointerdown={handlePointerDown}
			onpointermove={handlePointerMove}
			onpointerup={handlePointerUp}
			onpointercancel={handlePointerUp}
		>
			{#key currentIndex}
				<div
					in:fly={{ x: direction === 'left' ? 40 : -40, duration: 200, opacity: 1 }}
					out:fade={{ duration: 100 }}
					class="absolute inset-0 flex items-center justify-center"
				>
					<div
						style="transform: scale({zoomLevel}) translate({panOffset.x}px, {panOffset.y}px); transition: transform {isDragging ? '0s' : '0.15s'} ease;"
						class="grid max-h-[90vh] max-w-[90vw]"
						style:grid-template="1fr / 1fr"
					>
						{#if placeholderUrl && !imageLoaded}
							<img
								src={placeholderUrl}
								alt=""
								class="col-start-1 row-start-1 h-[90vh] w-[90vw] object-contain"
								aria-hidden="true"
							/>
						{/if}
						<img
							bind:this={imgEl}
							src={fullImageUrl}
							alt="Capture fullscreen view"
							class="col-start-1 row-start-1 max-h-[90vh] max-w-[90vw] object-contain transition-opacity duration-300"
							class:opacity-0={!imageLoaded || imageErrored}
							loading="eager"
							decoding="async"
							draggable="false"
							onload={() => (imageLoaded = true)}
							onerror={() => (imageErrored = true)}
						/>
						{#if imageErrored}
							<div class="col-start-1 row-start-1 flex flex-col items-center justify-center text-white/70">
								<ImageOff class="h-10 w-10" strokeWidth={1.5} />
								<span class="mt-2 text-sm">Image unavailable</span>
							</div>
						{/if}
					</div>

					<!-- Capture info overlay -->
					<div
						class="pointer-events-none absolute right-0 bottom-0 left-0 bg-gradient-to-t from-black/80 to-transparent p-4"
					>
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-2">
								{#if currentCapture.scene_name}
									<span class="rounded bg-white/20 px-2 py-1 text-sm font-medium text-white">
										{currentCapture.scene_name}
									</span>
								{/if}
							{#if currentCapture.profile_name}
								<span class="rounded bg-primary px-2 py-1 text-sm font-medium text-primary-foreground">
									{currentCapture.profile_name}
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
		</div>
	{/if}
</div>
