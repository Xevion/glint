<script lang="ts">
import { browser } from '$app/environment';
import type { FeaturedPair } from '$lib/bindings';
import { preloadImage } from '$lib/utils/image';
import { createTimerGroup } from '$lib/utils/timer';
import { createTween } from '$lib/utils/tween';
import PauseIcon from '@lucide/svelte/icons/pause';
import PlayIcon from '@lucide/svelte/icons/play';
import { onMount } from 'svelte';
import ComparisonSlider from './ComparisonSlider.svelte';
import { EMPTY_SIDE, type Orientation, SKEW_DEG, type SliderSide, pairToSides } from './types';

interface Props {
	pairs: FeaturedPair[];
	overlayVisible?: boolean;
}

let { pairs, overlayVisible = $bindable(true) }: Props = $props();

// --- Constants ---
const SWEEP_MS = 600;
const SWAP_DELAY = 50;
const RESUME_AFTER_DRAG = 3000;

// --- Showcase movement patterns ---
// Each pattern is a series of divider positions the slider visits during the
// matched-scene showcase phase. Every pattern starts and ends at 0.5 so the
// sweep-out always begins from center.
interface Waypoint {
	target: number;
	duration: number;
}

interface ShowcasePattern {
	name: string;
	waypoints: Waypoint[];
}

const SHOWCASE_PATTERNS: ShowcasePattern[] = [
	{
		name: 'classic-sweep',
		waypoints: [
			{ target: 0.25, duration: 1200 },
			{ target: 0.75, duration: 1600 },
			{ target: 0.5, duration: 1200 }
		]
	},
	{
		name: 'gentle-drift',
		waypoints: [
			{ target: 0.38, duration: 1500 },
			{ target: 0.62, duration: 1500 },
			{ target: 0.5, duration: 1000 }
		]
	},
	{
		name: 'quick-peek',
		waypoints: [
			{ target: 0.2, duration: 800 },
			{ target: 0.5, duration: 800 },
			{ target: 0.8, duration: 800 },
			{ target: 0.5, duration: 600 }
		]
	},
	{
		name: 'slow-reveal',
		waypoints: [
			{ target: 0.15, duration: 2000 },
			{ target: 0.85, duration: 2500 },
			{ target: 0.5, duration: 1500 }
		]
	}
];

// --- Orientation cycle ---
const ORIENTATIONS: Orientation[] = ['vertical', 'horizontal', 'diagonal'];

// --- Image pools (SliderSide arrays derived from pairs) ---
const leftPool = $derived<SliderSide[]>(pairs.map((p) => pairToSides(p).left));
const rightPool = $derived<SliderSide[]>(pairs.map((p) => pairToSides(p).right));

// --- Transition state machine ---
type TransitionState = 'resting' | 'showcase' | 'sweep-out' | 'swapping' | 'sweep-in';

const TRANSITION_GRAPH: Record<TransitionState, TransitionState | null> = {
	resting: 'showcase',
	showcase: 'sweep-out',
	'sweep-out': 'swapping',
	swapping: 'sweep-in',
	'sweep-in': null // loops back to showcase via startCycle()
};

function nextState(current: TransitionState): TransitionState {
	return TRANSITION_GRAPH[current] ?? 'resting';
}

function advanceTo(target: TransitionState) {
	const expected = nextState(transitionState);
	if (expected !== target) return; // invalid transition, ignore
	transitionState = target;
}

// --- Pattern selection (avoid repeating the same pattern consecutively) ---
let lastPatternIndex = -1;

function pickPattern(): ShowcasePattern {
	if (SHOWCASE_PATTERNS.length === 1) return SHOWCASE_PATTERNS[0];
	let idx: number;
	do {
		idx = Math.floor(Math.random() * SHOWCASE_PATTERNS.length);
	} while (idx === lastPatternIndex);
	lastPatternIndex = idx;
	return SHOWCASE_PATTERNS[idx];
}

// --- State ---
let transitionState = $state<TransitionState>('resting');
let dividerPosition = $state(0.5);
let orientation = $state<Orientation>('vertical');
let orientationIndex = $state(0);
let isUserDragging = $state(false);
let isPaused = $state(false);
let isHovered = $state(false);
let reducedMotion = $state(false);

// Monotonically increasing counter — bumped on any interruption (drag, pause, dot click).
// In-flight transitions compare their captured generation to reject stale continuations.
let generation = 0;

// Independent indices into each image pool
let leftIndex = $state(0);
let rightIndex = $state(0);

// Which side gets swapped next (alternates each cycle)
let swapSide = $state<'left' | 'right'>('left');

// Display state: two SliderSide objects instead of 12 individual fields
let displayLeft = $state<SliderSide>({ ...EMPTY_SIDE });
let displayRight = $state<SliderSide>({ ...EMPTY_SIDE });

const hasPairs = $derived(pairs.length > 0);
const hasMultiplePairs = $derived(pairs.length > 1);
const isAnimating = $derived(transitionState !== 'resting');

// Dot indicators: tracks the last pair explicitly synced (via init or dot click).
let activePairIndex = $state(0);

// --- Timers & Tween ---
const timers = createTimerGroup('cycle', 'resume', 'swap', 'overlay');
const tween = createTween();

// --- Drag smoothing (rAF chase loop) ---
// Instead of snapping to cursor, the divider chases the target with exponential decay.
// Higher DRAG_SPEED = snappier response. 15 feels responsive with a slight trail.
const DRAG_SPEED = 15;
let dragTarget = 0.5;
let dragRafId: number | null = null;
let dragLastTime = 0;

function startDragLoop() {
	if (dragRafId !== null) return;
	dragLastTime = performance.now();

	function tick(now: number) {
		const dt = (now - dragLastTime) / 1000;
		dragLastTime = now;

		const diff = dragTarget - dividerPosition;
		if (Math.abs(diff) < 0.0005) {
			dividerPosition = dragTarget;
			dragRafId = null;
			return;
		}

		// Time-based exponential interpolation (frame-rate independent)
		dividerPosition += diff * (1 - Math.exp(-DRAG_SPEED * dt));
		dragRafId = requestAnimationFrame(tick);
	}

	dragRafId = requestAnimationFrame(tick);
}

function stopDragLoop() {
	if (dragRafId !== null) {
		cancelAnimationFrame(dragRafId);
		dragRafId = null;
	}
}

function tweenTo(target: number, duration: number): Promise<void> {
	return tween.tweenTo(
		target,
		duration,
		(v) => {
			dividerPosition = v;
		},
		() => dividerPosition
	);
}

function scheduleOverlayRestore() {
	timers.overlay.set(() => {
		overlayVisible = true;
	}, RESUME_AFTER_DRAG);
}

// --- Display sync ---

function syncLeftFromIndex(idx: number) {
	displayLeft = leftPool[idx];
}

function syncRightFromIndex(idx: number) {
	displayRight = rightPool[idx];
}

// --- Transition state machine ---

function startCycle() {
	clearCycleTimers();
	if (!hasMultiplePairs || reducedMotion || isUserDragging || isPaused || isHovered) return;

	// No rest phase — begin the next transition immediately
	void beginTransition();
}

/** Wait for a duration, rejecting if the generation has been bumped. */
function generationDelay(ms: number, gen: number): Promise<void> {
	return new Promise((resolve, reject) => {
		timers.swap.set(() => {
			if (generation !== gen) {
				reject(new Error('cancelled'));
			} else {
				resolve();
			}
		}, ms);
	});
}

async function beginTransition() {
	if (!hasMultiplePairs || reducedMotion) return;

	const gen = generation;

	// Determine sweep direction based on which side we're swapping
	const hidingLeft = swapSide === 'left';

	// Diagonal clip-paths need to overshoot the edge
	const overshoot = orientation === 'diagonal' ? SKEW_DEG / 100 : 0;
	const edgeTarget = hidingLeft ? 0 - overshoot : 1 + overshoot;

	// Pick next orientation (applied on sweep-in)
	orientationIndex = (orientationIndex + 1) % ORIENTATIONS.length;
	const nextOrientation: Orientation = ORIENTATIONS[orientationIndex] ?? 'vertical';

	// Pick a showcase movement pattern
	const pattern = pickPattern();

	try {
		// Phase 1: Showcase — run through pattern waypoints (matched scene visible)
		advanceTo('showcase');
		for (const waypoint of pattern.waypoints) {
			await tweenTo(waypoint.target, waypoint.duration);
			if (generation !== gen) return;
		}

		// Phase 2: Sweep to edge (hides the target side)
		advanceTo('sweep-out');
		await tweenTo(edgeTarget, SWEEP_MS);
		if (generation !== gen) return;

		// Phase 3: Swap the hidden image only
		advanceTo('swapping');

		if (hidingLeft) {
			leftIndex = (leftIndex + 1) % leftPool.length;
			syncLeftFromIndex(leftIndex);
		} else {
			rightIndex = (rightIndex + 1) % rightPool.length;
			syncRightFromIndex(rightIndex);
		}

		// If the two visible sides now show different scenes, do a double-swap:
		// sweep through to the opposite edge to hide the stale side, swap it too.
		let finalEdgeIsLeft = hidingLeft;
		const leftScene = pairs[leftIndex].left_scene_name;
		const rightScene = pairs[rightIndex].right_scene_name;

		if (leftScene !== rightScene) {
			await generationDelay(SWAP_DELAY, gen);
			if (generation !== gen) return;

			// Preload the other side's next image during the cross-sweep
			if (hidingLeft) {
				const nextIdx = (rightIndex + 1) % rightPool.length;
				preloadImage(rightPool[nextIdx].image);
			} else {
				const nextIdx = (leftIndex + 1) % leftPool.length;
				preloadImage(leftPool[nextIdx].image);
			}

			// Sweep from current edge all the way to the opposite edge
			const crossOvershoot = orientation === 'diagonal' ? SKEW_DEG / 100 : 0;
			const oppositeEdge = hidingLeft ? 1 + crossOvershoot : 0 - crossOvershoot;
			await tweenTo(oppositeEdge, SWEEP_MS * 2);
			if (generation !== gen) return;

			// Swap the now-hidden other side
			if (hidingLeft) {
				rightIndex = (rightIndex + 1) % rightPool.length;
				syncRightFromIndex(rightIndex);
			} else {
				leftIndex = (leftIndex + 1) % leftPool.length;
				syncLeftFromIndex(leftIndex);
			}

			finalEdgeIsLeft = !hidingLeft;
		}

		swapSide = swapSide === 'left' ? 'right' : 'left';

		// Brief pause for image swap to render
		await generationDelay(SWAP_DELAY, gen);
		if (generation !== gen) return;

		// Phase 4: Sweep back to center with new orientation
		advanceTo('sweep-in');
		orientation = nextOrientation;

		const inOvershoot = nextOrientation === 'diagonal' ? SKEW_DEG / 100 : 0;
		dividerPosition = finalEdgeIsLeft ? 0 - inOvershoot : 1 + inOvershoot;

		await tweenTo(0.5, SWEEP_MS);
		if (generation !== gen) return;

		// Done — immediately start next cycle (no rest phase)
		transitionState = 'resting';
		startCycle();
	} catch {
		// Tween cancelled (user started dragging or paused)
	}
}

function clearCycleTimers() {
	timers.cycle.clear();
	timers.resume.clear();
	timers.swap.clear();
}

// --- User interaction ---

function handleDrag(pos: number) {
	dragTarget = pos;
	startDragLoop();
}

function handleDragStart() {
	isUserDragging = true;
	overlayVisible = false;
	timers.overlay.clear();
	generation++;
	clearCycleTimers();
	tween.cancel();
	transitionState = 'resting';
}

function handleDragEnd() {
	isUserDragging = false;
	stopDragLoop();
	dividerPosition = dragTarget;
	scheduleOverlayRestore();
	if (!isPaused) {
		timers.resume.set(() => {
			startCycle();
		}, RESUME_AFTER_DRAG);
	}
}

function handleDotClick(index: number) {
	if (index === activePairIndex) return;
	generation++;
	clearCycleTimers();
	timers.overlay.clear();
	tween.cancel();
	transitionState = 'resting';
	overlayVisible = true;
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
		clearCycleTimers();
		timers.overlay.clear();
		tween.cancel();
		transitionState = 'resting';
		overlayVisible = true;
		dividerPosition = 0.5;
	} else {
		startCycle();
	}
}

function handleHoverEnter() {
	isHovered = true;
	generation++;
	clearCycleTimers();
	tween.cancel();
	transitionState = 'resting';

	// Smoothly return to center instead of snapping
	if (Math.abs(dividerPosition - 0.5) > 0.01) {
		void tweenTo(0.5, 400);
	}
}

function handleHoverLeave() {
	isHovered = false;
	if (!isPaused && !isUserDragging && transitionState === 'resting') {
		startCycle();
	}
}

// Preload the next image during the showcase phase (when both matched images are visible)
$effect(() => {
	let img: HTMLImageElement | null = null;
	if (transitionState === 'showcase' && hasMultiplePairs && browser) {
		if (swapSide === 'left') {
			const nextIdx = (leftIndex + 1) % leftPool.length;
			img = preloadImage(leftPool[nextIdx].image);
		} else {
			const nextIdx = (rightIndex + 1) % rightPool.length;
			img = preloadImage(rightPool[nextIdx].image);
		}
	}
	return () => {
		if (img) {
			img.srcset = '';
			img.src = '';
		}
	};
});

// Detect prefers-reduced-motion
onMount(() => {
	const mq = window.matchMedia('(prefers-reduced-motion: reduce)');
	reducedMotion = mq.matches;

	const handler = (e: MediaQueryListEvent) => {
		reducedMotion = e.matches;
		if (e.matches) {
			clearCycleTimers();
			tween.cancel();
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
		clearCycleTimers();
		timers.overlay.clear();
		tween.cancel();
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
		onmouseenter={handleHoverEnter}
		onmouseleave={handleHoverLeave}
	>
		<ComparisonSlider
			left={displayLeft}
			right={displayRight}
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
