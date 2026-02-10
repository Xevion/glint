<script lang="ts">
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import DividerHandle, { type Orientation } from './DividerHandle.svelte';
import { cfImageSrcset, cfImageUrl } from '$lib/utils/image';
import { decodeThumbhash } from '$lib/utils/thumbhash';

interface Props {
	leftImage: string;
	rightImage: string;
	leftThumbhash?: string | null;
	rightThumbhash?: string | null;
	leftLabel?: string;
	rightLabel?: string;
	/** Divider position 0-1 (externally controlled) */
	dividerPosition?: number;
	orientation?: Orientation;
	disabled?: boolean;
	onDrag?: (position: number) => void;
	onDragStart?: () => void;
	onDragEnd?: () => void;
}

let {
	leftImage,
	rightImage,
	leftThumbhash = null,
	rightThumbhash = null,
	leftLabel,
	rightLabel,
	dividerPosition = 0.5,
	orientation = 'vertical',
	disabled = false,
	onDrag,
	onDragStart,
	onDragEnd
}: Props = $props();

let leftLoaded = $state(false);
let rightLoaded = $state(false);

const leftPlaceholder = $derived(decodeThumbhash(leftThumbhash));
const rightPlaceholder = $derived(decodeThumbhash(rightThumbhash));

const leftSrcset = $derived(cfImageSrcset(leftImage, 'hero'));
const rightSrcset = $derived(cfImageSrcset(rightImage, 'hero'));
const leftFallback = $derived(cfImageUrl(leftImage, { width: 1280, format: 'auto' }));
const rightFallback = $derived(cfImageUrl(rightImage, { width: 1280, format: 'auto' }));

// Reset loaded state when images change
let prevLeft: string | undefined;
let prevRight: string | undefined;
$effect(() => {
	if (leftImage !== prevLeft) {
		prevLeft = leftImage;
		leftLoaded = false;
	}
	if (rightImage !== prevRight) {
		prevRight = rightImage;
		rightLoaded = false;
	}
});

/**
 * Compute the CSS clip-path for the "right" (top) image.
 * The right image is clipped to only show the portion after the divider.
 */
const clipPath = $derived.by(() => {
	const pos = dividerPosition * 100;

	if (orientation === 'vertical') {
		// Right image shows from divider to right edge
		return `inset(0 0 0 ${pos}%)`;
	} else if (orientation === 'horizontal') {
		// Right image shows from divider to bottom edge
		return `inset(${pos}% 0 0 0)`;
	} else {
		// Diagonal: angled wipe
		const skew = 8; // degrees of angle
		const left = pos - skew;
		const right = pos + skew;
		return `polygon(${right}% 0%, 100% 0%, 100% 100%, ${left}% 100%)`;
	}
});
</script>

<div class="comparison-slider relative w-full overflow-hidden rounded-xl" style="aspect-ratio: 16/9;">
	<!-- Left (base) image -->
	<div class="absolute inset-0">
		{#if leftPlaceholder}
			<div
				class="absolute inset-0 bg-cover bg-center transition-opacity duration-300"
				class:opacity-0={leftLoaded}
				style:background-image="url({leftPlaceholder})"
			></div>
		{/if}
		{#if leftFallback}
			<img
				src={leftFallback}
				srcset={leftSrcset}
				sizes="(min-width: 1024px) 66vw, 100vw"
				alt={leftLabel ?? 'Left comparison'}
				class="absolute inset-0 h-full w-full object-cover transition-opacity duration-300"
				class:opacity-0={!leftLoaded}
				loading="eager"
				decoding="async"
				onload={() => (leftLoaded = true)}
			/>
		{/if}
		{#if leftLabel}
			<div class="absolute bottom-4 left-4 z-20 rounded-md bg-black/60 px-3 py-1.5 text-sm font-medium text-white backdrop-blur-sm">
				{leftLabel}
			</div>
		{/if}
	</div>

	<!-- Right (clipped) image -->
	<div class="absolute inset-0 will-change-[clip-path]" style:clip-path={clipPath}>
		{#if rightPlaceholder}
			<div
				class="absolute inset-0 bg-cover bg-center transition-opacity duration-300"
				class:opacity-0={rightLoaded}
				style:background-image="url({rightPlaceholder})"
			></div>
		{/if}
		{#if rightFallback}
			<img
				src={rightFallback}
				srcset={rightSrcset}
				sizes="(min-width: 1024px) 66vw, 100vw"
				alt={rightLabel ?? 'Right comparison'}
				class="absolute inset-0 h-full w-full object-cover transition-opacity duration-300"
				class:opacity-0={!rightLoaded}
				loading="eager"
				decoding="async"
				onload={() => (rightLoaded = true)}
			/>
		{/if}
		{#if rightLabel}
			<div class="absolute bottom-4 right-4 z-20 rounded-md bg-black/60 px-3 py-1.5 text-sm font-medium text-white backdrop-blur-sm">
				{rightLabel}
			</div>
		{/if}
	</div>

	<!-- Divider -->
	<DividerHandle
		position={dividerPosition}
		{orientation}
		{disabled}
		{onDrag}
		{onDragStart}
		{onDragEnd}
	/>
</div>
