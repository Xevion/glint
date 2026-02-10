<script lang="ts">
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { onMount } from 'svelte';
import { browser } from '$app/environment';
import type { FeaturedPair } from '$lib/bindings';
import ComparisonSlider from './ComparisonSlider.svelte';
import type { Orientation } from './DividerHandle.svelte';
import { preloadImage } from '$lib/utils/image';

interface Props {
	pairs: FeaturedPair[];
}

let { pairs }: Props = $props();

// --- Constants ---
const REST_DURATION = 5000;
const SWEEP_DURATION = 1000;
const SWAP_DELAY = 50;
const RESUME_AFTER_DRAG = 3000;
const EASING = 'cubic-bezier(0.4, 0, 0.15, 1)';

// --- Orientation cycle ---
const ORIENTATIONS: Orientation[] = ['vertical', 'horizontal', 'diagonal'];

// --- State ---
type TransitionState = 'resting' | 'sweep-out' | 'swapping' | 'sweep-in';

let currentIndex = $state(0);
let transitionState = $state<TransitionState>('resting');
let dividerPosition = $state(0.5);
let orientation = $state<Orientation>('vertical');
let orientationIndex = $state(0);
let isUserDragging = $state(false);
let reducedMotion = $state(false);

// Timer IDs
let cycleTimer: ReturnType<typeof setTimeout> | null = null;
let resumeTimer: ReturnType<typeof setTimeout> | null = null;

// The pair currently being displayed (left/right images may be from different pairs during swap)
let displayLeftImage = $state('');
let displayLeftThumbhash = $state<string | null>(null);
let displayLeftLabel = $state('');
let displayRightImage = $state('');
let displayRightThumbhash = $state<string | null>(null);
let displayRightLabel = $state('');

// Whether we sweep left-to-right or right-to-left (alternates for visual variety)
let sweepDirection = $state<'left' | 'right'>('right');

// Transition CSS for the divider position
let dividerTransition = $state('');

const hasPairs = $derived(pairs.length > 0);
const hasMultiplePairs = $derived(pairs.length > 1);

// Initialize display from current pair
function syncDisplay(pair: FeaturedPair) {
	displayLeftImage = pair.left_image_url;
	displayLeftThumbhash = pair.left_thumbhash;
	displayLeftLabel = `${pair.left_shader_name} — ${pair.left_scene_name}`;
	displayRightImage = pair.right_image_url;
	displayRightThumbhash = pair.right_thumbhash;
	displayRightLabel = `${pair.right_shader_name} — ${pair.right_scene_name}`;
}

// --- Transition state machine ---

function startCycle() {
	clearTimers();
	if (!hasMultiplePairs || reducedMotion || isUserDragging) return;

	cycleTimer = setTimeout(() => {
		beginTransition();
	}, REST_DURATION);
}

function beginTransition() {
	if (!hasMultiplePairs || reducedMotion) return;

	transitionState = 'sweep-out';

	// Pick next orientation
	orientationIndex = (orientationIndex + 1) % ORIENTATIONS.length;
	// We keep the current orientation during sweep-out, change on sweep-in
	// Actually — change orientation between pairs for visual variety
	const nextOrientation = ORIENTATIONS[orientationIndex];

	// Alternate sweep direction
	sweepDirection = sweepDirection === 'right' ? 'left' : 'right';

	// Sweep the divider to one edge (hiding one image completely)
	const targetPos = sweepDirection === 'right' ? 1 : 0;

	dividerTransition = `${SWEEP_DURATION}ms ${EASING}`;

	// Use requestAnimationFrame to ensure the transition property is applied before changing position
	requestAnimationFrame(() => {
		dividerPosition = targetPos;
	});

	// After sweep completes, swap the hidden image
	setTimeout(() => {
		transitionState = 'swapping';
		dividerTransition = '';

		// Advance to next pair
		const nextIndex = (currentIndex + 1) % pairs.length;
		const nextPair = pairs[nextIndex];

		// Swap the image that's now hidden behind the divider
		if (sweepDirection === 'right') {
			// Divider swept right — right image is fully visible, left is hidden
			// Swap the left image to next pair's left
			displayLeftImage = nextPair.left_image_url;
			displayLeftThumbhash = nextPair.left_thumbhash;
			displayLeftLabel = `${nextPair.left_shader_name} — ${nextPair.left_scene_name}`;
			// Also update right to next pair's right (it's showing but will be swapped visually)
			displayRightImage = nextPair.right_image_url;
			displayRightThumbhash = nextPair.right_thumbhash;
			displayRightLabel = `${nextPair.right_shader_name} — ${nextPair.right_scene_name}`;
		} else {
			// Divider swept left — left image is fully visible, right is hidden
			displayRightImage = nextPair.right_image_url;
			displayRightThumbhash = nextPair.right_thumbhash;
			displayRightLabel = `${nextPair.right_shader_name} — ${nextPair.right_scene_name}`;
			displayLeftImage = nextPair.left_image_url;
			displayLeftThumbhash = nextPair.left_thumbhash;
			displayLeftLabel = `${nextPair.left_shader_name} — ${nextPair.left_scene_name}`;
		}

		currentIndex = nextIndex;

		// Brief pause, then sweep back
		setTimeout(() => {
			transitionState = 'sweep-in';
			orientation = nextOrientation;

			dividerTransition = `${SWEEP_DURATION}ms ${EASING}`;
			requestAnimationFrame(() => {
				dividerPosition = 0.5;
			});

			// After sweep-in completes, rest
			setTimeout(() => {
				transitionState = 'resting';
				dividerTransition = '';
				startCycle();
			}, SWEEP_DURATION);
		}, SWAP_DELAY);
	}, SWEEP_DURATION);
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
}

// --- User interaction ---

function handleDrag(pos: number) {
	dividerPosition = pos;
}

function handleDragStart() {
	isUserDragging = true;
	clearTimers();
	dividerTransition = '';
	// If mid-transition, snap to wherever we are
	transitionState = 'resting';
}

function handleDragEnd() {
	isUserDragging = false;
	// Resume auto-cycle after timeout
	resumeTimer = setTimeout(() => {
		startCycle();
	}, RESUME_AFTER_DRAG);
}

function handleDotClick(index: number) {
	if (index === currentIndex) return;
	clearTimers();
	transitionState = 'resting';
	dividerTransition = '';
	currentIndex = index;
	dividerPosition = 0.5;
	syncDisplay(pairs[index]);
	startCycle();
}

// Preload next pair's images during rest
$effect(() => {
	if (transitionState === 'resting' && hasMultiplePairs && browser) {
		const nextIndex = (currentIndex + 1) % pairs.length;
		const nextPair = pairs[nextIndex];
		preloadImage(nextPair.left_image_url);
		preloadImage(nextPair.right_image_url);
	}
});

// Detect prefers-reduced-motion
onMount(() => {
	const mq = window.matchMedia('(prefers-reduced-motion: reduce)');
	reducedMotion = mq.matches;

	const handler = (e: MediaQueryListEvent) => {
		reducedMotion = e.matches;
		if (e.matches) {
			clearTimers();
			transitionState = 'resting';
			dividerTransition = '';
			dividerPosition = 0.5;
		} else {
			startCycle();
		}
	};
	mq.addEventListener('change', handler);

	// Initialize display
	if (hasPairs) {
		syncDisplay(pairs[0]);
	}

	// Start the cycle
	if (hasMultiplePairs && !reducedMotion) {
		startCycle();
	}

	return () => {
		clearTimers();
		mq.removeEventListener('change', handler);
	};
});

// Divider wrapper style: transition + custom property in one string
const wrapperStyle = $derived.by(() => {
	const parts: string[] = [];
	if (dividerTransition) {
		parts.push(`transition: all ${dividerTransition}`);
	}
	parts.push(`--divider-pos: ${dividerPosition}`);
	return parts.join('; ');
});
</script>

{#if hasPairs}
	<div
		class="hero-slider relative"
		role="region"
		aria-label="Shader comparison showcase"
		aria-roledescription="carousel"
	>
		<!-- Comparison slider wrapper — applies transition to divider position via CSS custom property -->
		<div class="relative" style={wrapperStyle}>
			<ComparisonSlider
				leftImage={displayLeftImage}
				rightImage={displayRightImage}
				leftThumbhash={displayLeftThumbhash}
				rightThumbhash={displayRightThumbhash}
				leftLabel={displayLeftLabel}
				rightLabel={displayRightLabel}
				{dividerPosition}
				{orientation}
				disabled={transitionState !== 'resting'}
				onDrag={handleDrag}
				onDragStart={handleDragStart}
				onDragEnd={handleDragEnd}
			/>
		</div>

		<!-- Dot indicators -->
		{#if hasMultiplePairs}
			<div class="mt-4 flex justify-center gap-2" role="tablist" aria-label="Select comparison pair">
				{#each pairs as _, i (i)}
					<button
						class="h-2 rounded-full transition-all duration-300 {i === currentIndex ? 'w-6 bg-white' : 'w-2 bg-white/40'}"
						role="tab"
						aria-selected={i === currentIndex}
						aria-label="Pair {i + 1}"
						onclick={() => handleDotClick(i)}
					></button>
				{/each}
			</div>
		{/if}
	</div>
{/if}
