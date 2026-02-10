<script lang="ts">
import ComparisonSide from './ComparisonSide.svelte';
import DividerLine from './DividerLine.svelte';
import { type Orientation, type SliderSide, computeClipPaths } from './types';

interface Props {
	left: SliderSide;
	right: SliderSide;
	/** Divider position 0-1 (externally controlled) */
	dividerPosition?: number;
	orientation?: Orientation;
	disabled?: boolean;
	/** Whether the parent is currently animating (enables will-change hint on clip-path) */
	isAnimating?: boolean;
	onDrag?: (position: number) => void;
	onDragStart?: () => void;
	onDragEnd?: () => void;
}

let {
	left,
	right,
	dividerPosition = 0.5,
	orientation = 'vertical',
	disabled = false,
	isAnimating = false,
	onDrag,
	onDragStart,
	onDragEnd
}: Props = $props();

let containerEl: HTMLDivElement | undefined = $state();
let isDragging = $state(false);
let isKeyboardDragging = $state(false);
let keyboardEndTimer: ReturnType<typeof setTimeout> | null = null;
const KEYBOARD_IDLE_MS = 500;

// Clean up keyboard debounce timer on destroy
$effect(() => {
	return () => {
		if (keyboardEndTimer) clearTimeout(keyboardEndTimer);
	};
});

const clipPaths = $derived(computeClipPaths(dividerPosition, orientation));

// --- Pointer interaction (on the whole container) ---

function getPositionFromEvent(clientX: number, clientY: number): number {
	if (!containerEl) return dividerPosition;
	const rect = containerEl.getBoundingClientRect();

	if (orientation === 'vertical' || orientation === 'diagonal') {
		return Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
	} else {
		return Math.max(0, Math.min(1, (clientY - rect.top) / rect.height));
	}
}

function handlePointerDown(e: PointerEvent) {
	if (disabled) return;

	// Let clicks on interactive elements (links, buttons) pass through
	const target = e.target as HTMLElement;
	if (target.closest('a, button')) return;

	e.preventDefault();

	// If keyboard drag was in progress, cancel it (pointer takes over)
	if (isKeyboardDragging) {
		if (keyboardEndTimer) clearTimeout(keyboardEndTimer);
		keyboardEndTimer = null;
		isKeyboardDragging = false;
	}

	isDragging = true;
	containerEl?.setPointerCapture(e.pointerId);
	onDragStart?.();

	// Jump divider to click position immediately
	const pos = getPositionFromEvent(e.clientX, e.clientY);
	onDrag?.(pos);
}

function handlePointerMove(e: PointerEvent) {
	if (!isDragging || disabled) return;
	e.preventDefault();
	const pos = getPositionFromEvent(e.clientX, e.clientY);
	onDrag?.(pos);
}

function handlePointerUp(e: PointerEvent) {
	if (!isDragging) return;
	isDragging = false;
	containerEl?.releasePointerCapture(e.pointerId);
	onDragEnd?.();
}

// --- Keyboard interaction (WAI-ARIA slider pattern) ---

const STEP_SMALL = 0.01;
const STEP_LARGE = 0.1;

function handleKeydown(e: KeyboardEvent) {
	if (disabled) return;

	let delta: number | undefined;

	switch (e.key) {
		case 'ArrowRight':
		case 'ArrowUp':
			delta = STEP_SMALL;
			break;
		case 'ArrowLeft':
		case 'ArrowDown':
			delta = -STEP_SMALL;
			break;
		case 'PageUp':
			delta = STEP_LARGE;
			break;
		case 'PageDown':
			delta = -STEP_LARGE;
			break;
		case 'Home':
			delta = -dividerPosition;
			break;
		case 'End':
			delta = 1 - dividerPosition;
			break;
		default:
			return;
	}

	e.preventDefault();

	// Start keyboard drag session (same path as pointer drag) unless pointer is active
	if (!isDragging && !isKeyboardDragging) {
		isKeyboardDragging = true;
		onDragStart?.();
	}

	const newPos = Math.max(0, Math.min(1, dividerPosition + delta));
	onDrag?.(newPos);

	// Debounce keyboard drag end (only when no pointer drag active)
	if (!isDragging) {
		if (keyboardEndTimer) clearTimeout(keyboardEndTimer);
		keyboardEndTimer = setTimeout(() => {
			isKeyboardDragging = false;
			keyboardEndTimer = null;
			onDragEnd?.();
		}, KEYBOARD_IDLE_MS);
	}
}

/** Cursor for the divider handle zone */
const handleCursor = $derived(
	orientation === 'horizontal' ? 'cursor-ns-resize' : 'cursor-ew-resize'
);

/** Allow scrolling on the axis perpendicular to the drag direction */
const touchAction = $derived(orientation === 'horizontal' ? 'pan-x' : 'pan-y');

const willChangeClip = $derived(isAnimating || isDragging);
</script>

<div
	bind:this={containerEl}
	class="comparison-slider relative w-full overflow-hidden rounded-xl select-none aspect-video max-sm:aspect-[4/3] max-sm:min-h-[280px]"
	style:touch-action={touchAction}
	role="slider"
	aria-orientation={orientation === 'horizontal' ? 'vertical' : 'horizontal'}
	aria-valuenow={Math.round(dividerPosition * 100)}
	aria-valuemin={0}
	aria-valuemax={100}
	aria-label="Comparison divider"
	tabindex={disabled ? -1 : 0}
	onpointerdown={handlePointerDown}
	onpointermove={handlePointerMove}
	onpointerup={handlePointerUp}
	onpointercancel={handlePointerUp}
	onkeydown={handleKeydown}
>
	<ComparisonSide
		side={left}
		position="left"
		clipPath={clipPaths.left}
		{orientation}
		willChange={willChangeClip}
	/>

	<ComparisonSide
		side={right}
		position="right"
		clipPath={clipPaths.right}
		{orientation}
		willChange={willChangeClip}
	/>

	<!-- Divider line with wider cursor zone -->
	<DividerLine
		position={dividerPosition}
		{orientation}
		cursor={disabled ? undefined : handleCursor}
	/>
</div>
