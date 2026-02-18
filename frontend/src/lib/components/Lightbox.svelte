<script lang="ts">
import { Button } from '$lib/components/ui/button';
import { imageUrl, rawImageUrl } from '$lib/utils/image';
import { decodeThumbhash } from '$lib/utils/thumbhash';
import { ChevronLeft, ChevronRight, ImageOff, X } from '@lucide/svelte';
import { fade } from 'svelte/transition';
import { Spring } from 'svelte/motion';

interface CaptureItem {
	id: string;
	image_path?: string | null;
	thumbhash?: string | null;
	profile_display_name?: string | null;
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

// Adjacent captures for 3-panel rendering
const prevCapture = $derived(currentIndex > 0 ? captures[currentIndex - 1] : null);
const nextCapture = $derived(
	currentIndex < captures.length - 1 ? captures[currentIndex + 1] : null
);

// Gesture state machine — exactly one gesture active at a time
type GestureState =
	| { type: 'idle' }
	| { type: 'pending'; startX: number; startY: number }
	| { type: 'swiping'; startX: number }
	| {
			type: 'panning';
			dragStart: { x: number; y: number };
			panStart: { x: number; y: number };
	  }
	| { type: 'pinching'; lastDistance: number; startZoom: number };

let gesture = $state<GestureState>({ type: 'idle' });
let hasMoved = false;
let suppressClick = false;

// Swipe spring: drives translateX for all three panels
const swipeSpring = new Spring(0, { stiffness: 0.3, damping: 0.8 });
let containerWidth = $state(0);
let swipeOffset = $state(0);
// Display offset: raw pixel offset during drag, spring-driven during animation
const displayOffset = $derived(gesture.type === 'swiping' ? swipeOffset : swipeSpring.current);

// Navigation animation tracking
let swipeGeneration = 0;
let isAnimating = false;
let pendingNavigationIndex: number | null = null;

// Velocity tracking (ring buffer of recent pointer samples)
let velocitySamples: { x: number; t: number }[] = [];

const currentCapture = $derived(captures[currentIndex]);
const hasPrev = $derived(currentIndex > 0);
const hasNext = $derived(currentIndex < captures.length - 1);

const fullImageUrl = $derived(imageUrl(currentCapture?.image_path, 'full'));
const placeholderUrl = $derived(decodeThumbhash(currentCapture?.thumbhash));

// Zoom and pan state
let zoomLevel = $state(1);
let panOffset = $state({ x: 0, y: 0 });
let imageLoaded = $state(false);
let imageErrored = $state(false);
let imgEl = $state<HTMLImageElement | null>(null);

// Progressive raw image: loaded on zoom for full fidelity
let rawImageLoaded = $state(false);
let rawImageRequested = $state(false);
const rawUrl = $derived(rawImageUrl(currentCapture?.image_path) ?? null);

// Reset zoom and loaded state when navigating to a different image
$effect(() => {
	void currentIndex;
	zoomLevel = 1;
	panOffset = { x: 0, y: 0 };
	imageLoaded = false;
	imageErrored = false;
	rawImageLoaded = false;
	rawImageRequested = false;
});

// Load raw (untransformed) image when user zooms in for full fidelity
$effect(() => {
	if (zoomLevel > 1 && !rawImageRequested && rawUrl) {
		rawImageRequested = true;
	}
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
		const path = captures[i]?.image_path;
		if (path) {
			const img = new Image();
			img.src = imageUrl(path, 'full') ?? '';
		}
	}
});

// Navigation

function navigateTo(newIndex: number) {
	if (newIndex < 0 || newIndex >= captures.length) return;

	// Complete any in-progress animation instantly
	if (isAnimating && pendingNavigationIndex !== null) {
		swipeGeneration++;
		void swipeSpring.set(0, { instant: true });
		onNavigate(pendingNavigationIndex);
		pendingNavigationIndex = null;
		isAnimating = false;
	}

	// Fallback: no container measured yet, navigate instantly
	if (containerWidth === 0) {
		onNavigate(newIndex);
		return;
	}

	const dir = newIndex > currentIndex ? -1 : 1;
	pendingNavigationIndex = newIndex;
	void animateNavigation(dir * containerWidth, newIndex);
}

async function animateNavigation(targetOffset: number, newIndex: number) {
	isAnimating = true;
	const gen = ++swipeGeneration;
	await swipeSpring.set(targetOffset);
	if (gen !== swipeGeneration) {
		isAnimating = false;
		return;
	}
	onNavigate(newIndex);
	void swipeSpring.set(0, { instant: true });
	pendingNavigationIndex = null;
	isAnimating = false;
}

// Event Handlers

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
			if (hasPrev) navigateTo(currentIndex - 1);
			break;
		case 'ArrowRight':
			if (hasNext) navigateTo(currentIndex + 1);
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

function handleDblClick(e: MouseEvent) {
	const target = e.target as HTMLElement;
	if (!target.closest('img')) return;
	if (zoomLevel === 1) {
		zoomLevel = 2;
	} else {
		zoomLevel = 1;
		panOffset = { x: 0, y: 0 };
	}
}

function handlePointerDown(e: PointerEvent) {
	if (gesture.type === 'pinching' || isAnimating) return;

	hasMoved = false;
	velocitySamples = [];
	swipeOffset = 0;

	if (zoomLevel > 1) {
		gesture = {
			type: 'panning',
			dragStart: { x: e.clientX, y: e.clientY },
			panStart: { x: panOffset.x, y: panOffset.y }
		};
	} else {
		gesture = { type: 'pending', startX: e.clientX, startY: e.clientY };
	}

	(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}

function applySwipe(e: PointerEvent, startX: number) {
	let offset = e.clientX - startX;
	if ((offset > 0 && !hasPrev) || (offset < 0 && !hasNext)) {
		offset *= 0.35;
	}
	swipeOffset = offset;

	const now = performance.now();
	velocitySamples.push({ x: e.clientX, t: now });
	if (velocitySamples.length > 4) velocitySamples.shift();
}

function handlePointerMove(e: PointerEvent) {
	switch (gesture.type) {
		case 'idle':
		case 'pinching':
			return;
		case 'panning': {
			const dx = e.clientX - gesture.dragStart.x;
			const dy = e.clientY - gesture.dragStart.y;
			if (Math.abs(dx) > 5 || Math.abs(dy) > 5) hasMoved = true;
			panOffset = {
				x: gesture.panStart.x + dx / zoomLevel,
				y: gesture.panStart.y + dy / zoomLevel
			};
			return;
		}
		case 'pending': {
			const dx = e.clientX - gesture.startX;
			const dy = e.clientY - gesture.startY;
			const absDx = Math.abs(dx);
			const absDy = Math.abs(dy);
			if (absDx > 5 || absDy > 5) hasMoved = true;

			// Need 10px of deliberate movement to commit to an axis
			if (absDx <= 10 && absDy <= 10) return;

			if (absDx >= absDy) {
				const { startX } = gesture;
				gesture = { type: 'swiping', startX };
				applySwipe(e, startX);
			} else {
				// Vertical movement — no gesture to perform
				gesture = { type: 'idle' };
			}
			return;
		}
		case 'swiping': {
			applySwipe(e, gesture.startX);
			return;
		}
	}
}

function handlePointerUp(_e: PointerEvent) {
	switch (gesture.type) {
		case 'pinching':
			return;
		case 'panning':
			if (hasMoved) suppressClick = true;
			gesture = { type: 'idle' };
			return;
		case 'swiping': {
			suppressClick = true;
			const offset = swipeOffset;

			// Compute velocity from recent samples
			let velocity = 0;
			if (velocitySamples.length >= 2) {
				const last = velocitySamples[velocitySamples.length - 1];
				const prev = velocitySamples[velocitySamples.length - 2];
				const dt = last.t - prev.t;
				if (dt > 0 && dt < 50) {
					velocity = (last.x - prev.x) / dt; // px/ms, positive = moving right
				}
			}

			// Sync spring to current drag position (no visual jump)
			void swipeSpring.set(offset, { instant: true });
			gesture = { type: 'idle' };

			const positionThreshold = containerWidth * 0.33;
			const velocityThreshold = 0.4; // px/ms

			if ((offset < -positionThreshold || velocity < -velocityThreshold) && hasNext) {
				void animateNavigation(-containerWidth, currentIndex + 1);
			} else if ((offset > positionThreshold || velocity > velocityThreshold) && hasPrev) {
				void animateNavigation(containerWidth, currentIndex - 1);
			} else {
				void swipeSpring.set(0);
			}
			return;
		}
		default:
			// idle or pending — no gesture to finalize
			if (hasMoved) suppressClick = true;
			gesture = { type: 'idle' };
			return;
	}
}

function handleContainerClick(e: MouseEvent) {
	if (suppressClick) {
		suppressClick = false;
		return;
	}
	const target = e.target as HTMLElement;
	if (target.closest('img')) return;
	if (zoomLevel > 1) {
		zoomLevel = 1;
		panOffset = { x: 0, y: 0 };
		return;
	}
	onClose();
}

function handleTouchStart(e: TouchEvent) {
	if (e.touches.length === 2) {
		const dx = e.touches[0].clientX - e.touches[1].clientX;
		const dy = e.touches[0].clientY - e.touches[1].clientY;
		gesture = {
			type: 'pinching',
			lastDistance: Math.hypot(dx, dy),
			startZoom: zoomLevel
		};
	}
}

function handleTouchMove(e: TouchEvent) {
	if (e.touches.length === 2 && gesture.type === 'pinching') {
		e.preventDefault();
		const dx = e.touches[0].clientX - e.touches[1].clientX;
		const dy = e.touches[0].clientY - e.touches[1].clientY;
		const distance = Math.hypot(dx, dy);
		zoomLevel = Math.min(5, Math.max(1, gesture.startZoom * (distance / gesture.lastDistance)));
		if (zoomLevel === 1) panOffset = { x: 0, y: 0 };
	}
}

function handleTouchEnd(e: TouchEvent) {
	if (e.touches.length < 2 && gesture.type === 'pinching') {
		gesture = { type: 'idle' };
		suppressClick = true;
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
			class="absolute left-4 z-10 cursor-pointer bg-white/10 text-white hover:bg-white/20 sm:bg-transparent sm:hover:bg-white/10"
			onclick={() => navigateTo(currentIndex - 1)}
		>
			<ChevronLeft class="h-8 w-8" />
		</Button>
	{/if}

	{#if hasNext}
		<Button
			variant="ghost"
			size="icon"
			class="absolute right-4 z-10 cursor-pointer bg-white/10 text-white hover:bg-white/20 sm:bg-transparent sm:hover:bg-white/10"
			onclick={() => navigateTo(currentIndex + 1)}
		>
			<ChevronRight class="h-8 w-8" />
		</Button>
	{/if}

	<!-- Main image viewport -->
	{#if currentCapture?.image_path}
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
		<div
			class="relative h-[90vh] w-[90vw] select-none overflow-hidden"
			class:cursor-grab={zoomLevel > 1 && gesture.type !== 'panning'}
			class:cursor-grabbing={gesture.type === 'panning'}
			style:touch-action="none"
			role="img"
			tabindex="-1"
			onclick={handleContainerClick}
			onkeydown={(e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault();
					if (zoomLevel > 1) {
						zoomLevel = 1;
						panOffset = { x: 0, y: 0 };
					} else {
						onClose();
					}
				}
			}}
			onwheel={handleWheel}
			ondblclick={handleDblClick}
			onpointerdown={handlePointerDown}
			onpointermove={handlePointerMove}
			onpointerup={handlePointerUp}
			onpointercancel={handlePointerUp}
			ontouchstart={handleTouchStart}
			ontouchmove={handleTouchMove}
			ontouchend={handleTouchEnd}
			bind:clientWidth={containerWidth}
		>
			<!-- Previous image panel -->
		{#if prevCapture?.image_path && containerWidth > 0}
			{@const prevUrl = imageUrl(prevCapture.image_path, 'full')}
				{@const prevThumb = decodeThumbhash(prevCapture.thumbhash)}
				<div
					class="absolute inset-0 flex items-center justify-center"
					style:transform="translateX({displayOffset - containerWidth}px)"
				>
					<div
						class="grid max-h-[90vh] max-w-[90vw]"
						style:grid-template="1fr / 1fr"
					>
						{#if prevThumb}
							<img
								src={prevThumb}
								alt=""
								class="col-start-1 row-start-1 h-[90vh] w-[90vw] object-contain"
								aria-hidden="true"
							/>
						{/if}
						{#if prevUrl}
							<img
								src={prevUrl}
								alt="Previous capture"
								class="col-start-1 row-start-1 h-[90vh] w-[90vw] object-contain"
								loading="eager"
								draggable="false"
							/>
						{/if}
					</div>
				</div>
			{/if}

			<!-- Current image panel -->
			<div
				class="absolute inset-0 flex items-center justify-center"
				style:transform="translateX({displayOffset}px)"
			>
				<div
					style="transform: scale({zoomLevel}) translate({panOffset.x}px, {panOffset.y}px); transition: transform {gesture.type === 'panning' ? '0s' : '0.15s'} ease;"
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
			{#if rawImageRequested && rawUrl}
				<img
					src={rawUrl}
						alt="Capture fullscreen view (full resolution)"
						class="col-start-1 row-start-1 max-h-[90vh] max-w-[90vw] object-contain transition-opacity duration-200"
						class:opacity-0={!rawImageLoaded}
						loading="eager"
						decoding="async"
						draggable="false"
						onload={() => (rawImageLoaded = true)}
					/>
				{/if}
				{#if imageErrored}
						<div
							class="col-start-1 row-start-1 flex flex-col items-center justify-center text-white/70"
						>
							<ImageOff class="h-10 w-10" strokeWidth={1.5} />
							<span class="mt-2 text-sm">Image unavailable</span>
						</div>
					{/if}
				</div>
			</div>

			<!-- Next image panel -->
		{#if nextCapture?.image_path && containerWidth > 0}
			{@const nextUrl = imageUrl(nextCapture.image_path, 'full')}
				{@const nextThumb = decodeThumbhash(nextCapture.thumbhash)}
				<div
					class="absolute inset-0 flex items-center justify-center"
					style:transform="translateX({displayOffset + containerWidth}px)"
				>
					<div
						class="grid max-h-[90vh] max-w-[90vw]"
						style:grid-template="1fr / 1fr"
					>
						{#if nextThumb}
							<img
								src={nextThumb}
								alt=""
								class="col-start-1 row-start-1 h-[90vh] w-[90vw] object-contain"
								aria-hidden="true"
							/>
						{/if}
						{#if nextUrl}
							<img
								src={nextUrl}
								alt="Next capture"
								class="col-start-1 row-start-1 h-[90vh] w-[90vw] object-contain"
								loading="eager"
								draggable="false"
							/>
						{/if}
					</div>
				</div>
			{/if}

			<!-- Capture info overlay (static, doesn't translate with swipe) -->
			<div
				class="pointer-events-none absolute right-0 bottom-0 left-0 z-10 bg-gradient-to-t from-black/80 to-transparent p-4"
			>
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						{#if currentCapture.scene_name}
							<span class="rounded bg-white/20 px-2 py-1 text-sm font-medium text-white">
								{currentCapture.scene_name}
							</span>
						{/if}
					{#if currentCapture.profile_display_name}
						<span
							class="rounded bg-primary px-2 py-1 text-sm font-medium text-primary-foreground"
						>
							{currentCapture.profile_display_name}
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
	{/if}
</div>
