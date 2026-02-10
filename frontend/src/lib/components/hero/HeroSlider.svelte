<script lang="ts">
import { onMount } from 'svelte';
import { browser } from '$app/environment';
import type { FeaturedPair } from '$lib/bindings';
import ComparisonSlider from './ComparisonSlider.svelte';
import { SKEW_DEG, type Orientation } from './types';
import { preloadImage } from '$lib/utils/image';
import PauseIcon from '@lucide/svelte/icons/pause';
import PlayIcon from '@lucide/svelte/icons/play';

interface Props {
	pairs: FeaturedPair[];
}

let { pairs }: Props = $props();

// --- Constants ---
const REST_DURATION = 5000;
const SHOWCASE_DURATION = 1200;
const SWEEP_DURATION = 800;
const SWAP_DELAY = 50;
const RESUME_AFTER_DRAG = 3000;

const SHOWCASE_NEAR = 0.2;
const SHOWCASE_FAR = 0.8;

// --- Orientation cycle ---
const ORIENTATIONS: Orientation[] = ['vertical', 'horizontal', 'diagonal'];

// --- Image pool ---
// Flatten pairs into individual image entries that we rotate through independently
// per side. Left slot draws from left images, right slot from right images.
interface ImageEntry {
	url: string;
	thumbhash: string | null;
	label: string;
}

const leftImages = $derived<ImageEntry[]>(
	pairs.map((p) => ({
		url: p.left_image_url,
		thumbhash: p.left_thumbhash ?? null,
		label: p.left_shader_name
	}))
);
const rightImages = $derived<ImageEntry[]>(
	pairs.map((p) => ({
		url: p.right_image_url,
		thumbhash: p.right_thumbhash ?? null,
		label: p.right_shader_name
	}))
);

// --- State ---
type TransitionState =
	| 'resting'
	| 'showcase-a'
	| 'showcase-b'
	| 'sweep-out'
	| 'swapping'
	| 'sweep-in';

let transitionState = $state<TransitionState>('resting');
let dividerPosition = $state(0.5);
let orientation = $state<Orientation>('vertical');
let orientationIndex = $state(0);
let isUserDragging = $state(false);
let isPaused = $state(false);
let reducedMotion = $state(false);

// Monotonically increasing counter — bumped on any interruption (drag, pause, dot click).
// In-flight transitions compare their captured generation to reject stale continuations.
let generation = 0;

// Independent indices into each image pool
let leftIndex = $state(0);
let rightIndex = $state(0);

// Which side gets swapped next (alternates each cycle)
let swapSide = $state<'left' | 'right'>('left');

// Timer IDs
let cycleTimer: ReturnType<typeof setTimeout> | null = null;
let resumeTimer: ReturnType<typeof setTimeout> | null = null;
let swapTimer: ReturnType<typeof setTimeout> | null = null;
let tweenRaf: number | null = null;

// Display state for each slot
let displayLeftImage = $state('');
let displayLeftThumbhash = $state<string | null>(null);
let displayLeftLabel = $state('');
let displayRightImage = $state('');
let displayRightThumbhash = $state<string | null>(null);
let displayRightLabel = $state('');

const hasPairs = $derived(pairs.length > 0);
const hasMultiplePairs = $derived(pairs.length > 1);
const isAnimating = $derived(transitionState !== 'resting');

// Dot indicators: tracks the last pair explicitly synced (via init or dot click).
// leftIndex/rightIndex diverge during alternating swaps, so neither alone is meaningful.
let activePairIndex = $state(0);

// --- Easing ---

function easeInOutCubic(t: number): number {
	return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
}

// --- Tween engine ---

function tweenTo(target: number, duration: number): Promise<void> {
	return new Promise((resolve, reject) => {
		cancelTween();

		const start = dividerPosition;
		const delta = target - start;
		if (Math.abs(delta) < 0.001) {
			dividerPosition = target;
			resolve();
			return;
		}

		const startTime = performance.now();
		let cancelled = false;

		function tick(now: number) {
			if (cancelled) {
				reject(new Error('cancelled'));
				return;
			}

			const elapsed = now - startTime;
			const progress = Math.min(1, elapsed / duration);
			const eased = easeInOutCubic(progress);

			dividerPosition = start + delta * eased;

			if (progress < 1) {
				tweenRaf = requestAnimationFrame(tick);
			} else {
				dividerPosition = target;
				tweenRaf = null;
				resolve();
			}
		}

		tweenRaf = requestAnimationFrame(tick);

		cancelTween = () => {
			if (tweenRaf !== null) {
				cancelAnimationFrame(tweenRaf);
				tweenRaf = null;
			}
			cancelled = true;
			cancelTween = cancelTweenNoop;
		};
	});
}

function cancelTweenNoop() {
	if (tweenRaf !== null) {
		cancelAnimationFrame(tweenRaf);
		tweenRaf = null;
	}
}
let cancelTween = cancelTweenNoop;

// --- Display sync ---

function syncLeftFromIndex(idx: number) {
	const entry = leftImages[idx];
	displayLeftImage = entry.url;
	displayLeftThumbhash = entry.thumbhash;
	displayLeftLabel = entry.label;
}

function syncRightFromIndex(idx: number) {
	const entry = rightImages[idx];
	displayRightImage = entry.url;
	displayRightThumbhash = entry.thumbhash;
	displayRightLabel = entry.label;
}

// --- Transition state machine ---

function startCycle() {
	clearTimers();
	if (!hasMultiplePairs || reducedMotion || isUserDragging || isPaused) return;

	cycleTimer = setTimeout(() => {
		void beginTransition();
	}, REST_DURATION);
}

/** Wait for a duration, rejecting if the generation has been bumped. */
function generationDelay(ms: number, gen: number): Promise<void> {
	return new Promise((resolve, reject) => {
		const id = setTimeout(() => {
			if (generation !== gen) {
				reject(new Error('cancelled'));
			} else {
				resolve();
			}
		}, ms);
		// If the generation bumps before the timeout fires, the next
		// cancelTween() + clearTimers() will handle cleanup. We also
		// store the timeout so clearTimers can cancel it.
		swapTimer = id;
	});
}

async function beginTransition() {
	if (!hasMultiplePairs || reducedMotion) return;

	const gen = generation;

	// Determine sweep direction based on which side we're swapping:
	// To hide the LEFT image, sweep divider to 0% (left edge).
	//   The clip-path clips the right image to show from dividerPos rightward,
	//   so at 0% the right image covers everything and left is hidden.
	// To hide the RIGHT image, sweep divider to 100% (right edge).
	//   At 100% the right image is fully clipped away, left covers everything.
	const hidingLeft = swapSide === 'left';

	// Diagonal clip-paths need to overshoot the edge so the angled split
	// clears the viewport entirely. The polygon offsets by SKEW_DEG% from
	// the position, so we push past the edge by that amount.
	const overshoot = orientation === 'diagonal' ? SKEW_DEG / 100 : 0;
	const edgeTarget = hidingLeft ? 0 - overshoot : 1 + overshoot;

	// Showcase sweep goes toward the side we'll eventually hide, then the opposite
	const nearTarget = hidingLeft ? SHOWCASE_NEAR : SHOWCASE_FAR;
	const farTarget = hidingLeft ? SHOWCASE_FAR : SHOWCASE_NEAR;

	// Pick next orientation (applied on sweep-in)
	orientationIndex = (orientationIndex + 1) % ORIENTATIONS.length;
	const nextOrientation: Orientation = ORIENTATIONS[orientationIndex] ?? 'vertical';

	try {
		// Phase 1: Showcase sweep A (center → toward the swap side)
		transitionState = 'showcase-a';
		await tweenTo(nearTarget, SHOWCASE_DURATION);
		if (generation !== gen) return;

		// Phase 2: Showcase sweep B (across to the other side)
		transitionState = 'showcase-b';
		await tweenTo(farTarget, SHOWCASE_DURATION);
		if (generation !== gen) return;

		// Phase 3: Sweep to edge (hides the target side)
		transitionState = 'sweep-out';
		await tweenTo(edgeTarget, SWEEP_DURATION);
		if (generation !== gen) return;

		// Phase 4: Swap the hidden image only
		transitionState = 'swapping';

		if (hidingLeft) {
			leftIndex = (leftIndex + 1) % leftImages.length;
			syncLeftFromIndex(leftIndex);
		} else {
			rightIndex = (rightIndex + 1) % rightImages.length;
			syncRightFromIndex(rightIndex);
		}

		// Alternate for next cycle
		swapSide = swapSide === 'left' ? 'right' : 'left';

		// Brief pause for image swap to render (cancellable via generation)
		await generationDelay(SWAP_DELAY, gen);
		if (generation !== gen) return;

		// Phase 5: Sweep back to center with new orientation
		transitionState = 'sweep-in';
		orientation = nextOrientation;

		// Jump to the correct overshoot for the new orientation before sweeping in.
		// If the new orientation is diagonal, the divider must start further out so
		// the angled split enters from fully off-screen.
		const inOvershoot = nextOrientation === 'diagonal' ? SKEW_DEG / 100 : 0;
		dividerPosition = hidingLeft ? 0 - inOvershoot : 1 + inOvershoot;

		await tweenTo(0.5, SWEEP_DURATION);
		if (generation !== gen) return;

		// Done — rest
		transitionState = 'resting';
		startCycle();
	} catch {
		// Tween cancelled (user started dragging or paused)
	}
}

function clearTimers() {
	if (cycleTimer) {
		clearTimeout(cycleTimer);
		cycleTimer = null;
	}
	if (resumeTimer) {
		clearTimeout(resumeTimer);
		resumeTimer = null;
	}
	if (swapTimer) {
		clearTimeout(swapTimer);
		swapTimer = null;
	}
}

// --- User interaction ---

function handleDrag(pos: number) {
	dividerPosition = pos;
}

function handleDragStart() {
	isUserDragging = true;
	generation++;
	clearTimers();
	cancelTween();
	transitionState = 'resting';
}

function handleDragEnd() {
	isUserDragging = false;
	if (isPaused) return;
	resumeTimer = setTimeout(() => {
		startCycle();
	}, RESUME_AFTER_DRAG);
}

function handleDotClick(index: number) {
	if (index === activePairIndex) return;
	generation++;
	clearTimers();
	cancelTween();
	transitionState = 'resting';
	leftIndex = index;
	rightIndex = index;
	activePairIndex = index;
	dividerPosition = 0.5;
	syncLeftFromIndex(index);
	syncRightFromIndex(index);
	startCycle();
}

function togglePause() {
	isPaused = !isPaused;
	if (isPaused) {
		generation++;
		clearTimers();
		cancelTween();
		transitionState = 'resting';
		dividerPosition = 0.5;
	} else {
		startCycle();
	}
}

// Preload the next image for whichever side will be swapped next (using 'hero' preset to match display)
$effect(() => {
	let img: HTMLImageElement | null = null;
	if (transitionState === 'resting' && hasMultiplePairs && browser) {
		if (swapSide === 'left') {
			const nextIdx = (leftIndex + 1) % leftImages.length;
			img = preloadImage(leftImages[nextIdx].url);
		} else {
			const nextIdx = (rightIndex + 1) % rightImages.length;
			img = preloadImage(rightImages[nextIdx].url);
		}
	}
	return () => {
		if (img) img.src = '';
	};
});

// Detect prefers-reduced-motion
onMount(() => {
	const mq = window.matchMedia('(prefers-reduced-motion: reduce)');
	reducedMotion = mq.matches;

	const handler = (e: MediaQueryListEvent) => {
		reducedMotion = e.matches;
		if (e.matches) {
			clearTimers();
			cancelTween();
			transitionState = 'resting';
			dividerPosition = 0.5;
		} else {
			startCycle();
		}
	};
	mq.addEventListener('change', handler);

	// Initialize display from first pair
	if (hasPairs) {
		syncLeftFromIndex(0);
		syncRightFromIndex(0);
	}

	if (hasMultiplePairs && !reducedMotion) {
		startCycle();
	}

	return () => {
		clearTimers();
		cancelTween();
		mq.removeEventListener('change', handler);
	};
});
</script>

{#if hasPairs}
	<div
		class="hero-slider relative"
		role="region"
		aria-label="Shader comparison showcase"
		aria-roledescription="carousel"
	>
		<ComparisonSlider
			leftImage={displayLeftImage}
			rightImage={displayRightImage}
			leftThumbhash={displayLeftThumbhash}
			rightThumbhash={displayRightThumbhash}
			leftLabel={displayLeftLabel}
			rightLabel={displayRightLabel}
			{dividerPosition}
			{orientation}
			disabled={isAnimating}
			{isAnimating}
			onDrag={handleDrag}
			onDragStart={handleDragStart}
			onDragEnd={handleDragEnd}
		/>

		<!-- Carousel controls -->
		{#if hasMultiplePairs}
			<div class="mt-4 flex items-center justify-center gap-3">
				<div class="flex gap-2" role="group" aria-label="Select comparison pair">
					{#each pairs as _, i (i)}
						<button
							class="h-2 rounded-full transition-all duration-300 {i === activePairIndex ? 'w-6 bg-white' : 'w-2 bg-white/40'}"
							aria-current={i === activePairIndex ? 'true' : undefined}
							aria-label="Pair {i + 1}"
							onclick={() => handleDotClick(i)}
						></button>
					{/each}
				</div>
				<button
					class="flex h-6 w-6 items-center justify-center rounded-full bg-white/20 text-white transition-colors hover:bg-white/30"
					aria-label={isPaused ? 'Play carousel' : 'Pause carousel'}
					onclick={togglePause}
				>
					{#if isPaused}
						<PlayIcon size={12} />
					{:else}
						<PauseIcon size={12} />
					{/if}
				</button>
			</div>
		{/if}
	</div>
{/if}
