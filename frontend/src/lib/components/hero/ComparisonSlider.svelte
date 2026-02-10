<script lang="ts">
import { resolve } from '$app/paths';
import DividerLine from './DividerLine.svelte';
import { SKEW_DEG, type Orientation } from './types';
import { cfImageSrcset, cfImageUrl } from '$lib/utils/image';
import { formatVersion } from '$lib/utils/display';
import { decodeThumbhash } from '$lib/utils/thumbhash';

interface Props {
	leftImage: string;
	rightImage: string;
	leftThumbhash: string | null;
	rightThumbhash: string | null;
	leftLabel: string;
	rightLabel: string;
	leftSlug: string;
	rightSlug: string;
	leftAuthor: string | null;
	rightAuthor: string | null;
	leftVersion: string;
	rightVersion: string;
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
	leftImage,
	rightImage,
	leftThumbhash,
	rightThumbhash,
	leftLabel,
	rightLabel,
	leftSlug,
	rightSlug,
	leftAuthor,
	rightAuthor,
	leftVersion,
	rightVersion,
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
let leftLoaded = $state(false);
let rightLoaded = $state(false);

const leftPlaceholder = $derived(decodeThumbhash(leftThumbhash));
const rightPlaceholder = $derived(decodeThumbhash(rightThumbhash));

const leftSrc = $derived(cfImageUrl(leftImage, { width: 1280, format: 'auto' }));
const rightSrc = $derived(cfImageUrl(rightImage, { width: 1280, format: 'auto' }));
const leftSrcset = $derived(cfImageSrcset(leftImage, 'hero'));
const rightSrcset = $derived(cfImageSrcset(rightImage, 'hero'));

// Reset loaded state when images change — each effect tracks one prop
$effect(() => {
	void leftImage; // subscribe to trigger on change
	leftLoaded = false;
});
$effect(() => {
	void rightImage; // subscribe to trigger on change
	rightLoaded = false;
});

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
	const newPos = Math.max(0, Math.min(1, dividerPosition + delta));
	onDrag?.(newPos);
}

/**
 * Compute CSS clip-paths for left and right image layers.
 * The left layer shows the portion before the divider, the right layer after.
 */
const clipPaths = $derived.by(() => {
	const pos = dividerPosition * 100;

	if (orientation === 'vertical') {
		return {
			left: `inset(0 ${100 - pos}% 0 0)`,
			right: `inset(0 0 0 ${pos}%)`
		};
	} else if (orientation === 'horizontal') {
		return {
			left: `inset(0 0 ${100 - pos}% 0)`,
			right: `inset(${pos}% 0 0 0)`
		};
	} else {
		const l = pos - SKEW_DEG;
		const r = pos + SKEW_DEG;
		return {
			left: `polygon(0% 0%, ${r}% 0%, ${l}% 100%, 0% 100%)`,
			right: `polygon(${r}% 0%, 100% 0%, 100% 100%, ${l}% 100%)`
		};
	}
});

/** Cursor for the divider handle zone */
const handleCursor = $derived(
	orientation === 'horizontal' ? 'cursor-ns-resize' : 'cursor-ew-resize'
);

/** Allow scrolling on the axis perpendicular to the drag direction */
const touchAction = $derived(orientation === 'horizontal' ? 'pan-x' : 'pan-y');

const willChangeClip = $derived(isAnimating || isDragging);
</script>

{#snippet imageSlot(
	src: string | null,
	srcset: string | null,
	placeholder: string | null,
	loaded: boolean,
	onLoad: () => void,
	alt: string,
	label: string,
	slug: string,
	author: string | null,
	version: string,
	side: 'left' | 'right',
	clipPath: string
)}
	<div
		class="absolute inset-0"
		class:will-change-[clip-path]={willChangeClip}
		style:clip-path={clipPath}
	>
		{#if placeholder}
			<div
				class="absolute inset-0 bg-cover bg-center transition-opacity duration-300"
				class:opacity-0={loaded}
				style:background-image="url({placeholder})"
			></div>
		{/if}
		<img
			src={src}
			srcset={srcset}
			sizes="(min-width: 1024px) 66vw, 100vw"
			{alt}
			class="pointer-events-none absolute inset-0 h-full w-full object-cover transition-opacity duration-300"
			class:opacity-0={!loaded}
			loading="eager"
			decoding="async"
			fetchpriority="high"
			draggable="false"
			onload={onLoad}
		/>
		{#if label}
			{@const href = resolve('/shaders/[id]', { id: slug })}
			{@const detail = author && version ? `by ${author}, ${formatVersion(version)}` : version ? formatVersion(version) : author ? `by ${author}` : undefined}
			<a
				{href}
				class="pointer-events-auto absolute z-20 block rounded-md border border-white/15 bg-black/60 px-3 py-1.5 backdrop-blur-sm transition-all hover:border-white/30 hover:bg-black/70
					{side === 'left' ? 'left-4' : 'right-4'}
					{orientation === (side === 'left' ? 'horizontal' : 'diagonal') ? 'top-4' : 'bottom-4'}"
			>
				<span class="text-sm font-medium text-white">{label}</span>
				{#if detail}
					<div class="hidden text-xs text-white/70 md:block">{detail}</div>
				{/if}
			</a>
		{/if}
	</div>
{/snippet}

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
	{@render imageSlot(
		leftSrc,
		leftSrcset,
		leftPlaceholder,
		leftLoaded,
		() => (leftLoaded = true),
		leftLabel ?? 'Left comparison',
		leftLabel,
		leftSlug,
		leftAuthor,
		leftVersion,
		'left',
		clipPaths.left
	)}

	{@render imageSlot(
		rightSrc,
		rightSrcset,
		rightPlaceholder,
		rightLoaded,
		() => (rightLoaded = true),
		rightLabel ?? 'Right comparison',
		rightLabel,
		rightSlug,
		rightAuthor,
		rightVersion,
		'right',
		clipPaths.right
	)}

	<!-- Divider line with wider cursor zone -->
	<DividerLine
		position={dividerPosition}
		{orientation}
		cursor={disabled ? undefined : handleCursor}
	/>
</div>
