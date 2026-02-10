<script lang="ts">
import { onMount } from 'svelte';
import 'img-comparison-slider';
import ShaderInfoOverlay from './ShaderInfoOverlay.svelte';
import SplitCanvas from './SplitCanvas.svelte';
import { SAMPLING, SLIDER } from './constants';
import type { CompareMode, ElementLuminances, ShaderDisplayInfo } from './types';

interface Props {
	leftImage: string;
	rightImage: string;
	mode?: CompareMode;
	leftShader?: ShaderDisplayInfo;
	rightShader?: ShaderDisplayInfo;
}

let { leftImage, rightImage, mode = 'slider', leftShader, rightShader }: Props = $props();

let showLeft = $state(true);
let sliderPosition = $state(50);

// Luminance sampling state
let leftLuminance = $state<number | undefined>(undefined);
let rightLuminance = $state<number | undefined>(undefined);
let leftImgEl = $state<HTMLImageElement | null>(null);
let rightImgEl = $state<HTMLImageElement | null>(null);

const showLeftOverlay = $derived(mode !== 'slider' || sliderPosition > SLIDER.HIDE_LEFT_THRESHOLD);
const showRightOverlay = $derived(
	mode !== 'slider' || sliderPosition < SLIDER.HIDE_RIGHT_THRESHOLD
);

// Convert single luminance to per-element format (all elements get same value)
function toElementLuminances(luminance: number | undefined): ElementLuminances | undefined {
	if (luminance === undefined) return undefined;
	return { name: luminance, meta: luminance, author: luminance };
}

/**
 * Calculate average luminance from an image region.
 * Returns 0-1 where 0 is black and 1 is white.
 */
function sampleLuminance(
	img: HTMLImageElement,
	corner: 'top-left' | 'top-right'
): number | undefined {
	if (!img.complete || img.naturalWidth === 0) return undefined;

	const canvas = document.createElement('canvas');
	const ctx = canvas.getContext('2d', { willReadFrequently: true });
	if (!ctx) return undefined;

	// Scale sample region to image's natural size
	const scaleX = img.naturalWidth / img.clientWidth;
	const scaleY = img.naturalHeight / img.clientHeight;

	const sampleW = Math.min(SAMPLING.WIDTH * scaleX, img.naturalWidth * 0.4);
	const sampleH = Math.min(SAMPLING.HEIGHT * scaleY, img.naturalHeight * 0.2);

	// Position based on corner (with padding offset)
	const padding = 12 * scaleX; // ~0.75rem padding
	const x = corner === 'top-left' ? padding : img.naturalWidth - sampleW - padding;
	const y = padding;

	canvas.width = sampleW;
	canvas.height = sampleH;

	try {
		ctx.drawImage(img, x, y, sampleW, sampleH, 0, 0, sampleW, sampleH);
		const imageData = ctx.getImageData(0, 0, sampleW, sampleH);
		const data = imageData.data;

		let totalLuminance = 0;
		const pixelCount = data.length / 4;

		for (let i = 0; i < data.length; i += 4) {
			// Relative luminance formula (ITU-R BT.709)
			const r = data[i] / 255;
			const g = data[i + 1] / 255;
			const b = data[i + 2] / 255;
			totalLuminance += 0.2126 * r + 0.7152 * g + 0.0722 * b;
		}

		return totalLuminance / pixelCount;
	} catch {
		// CORS or other error
		return undefined;
	}
}

function sampleImages() {
	if (leftImgEl) {
		leftLuminance = sampleLuminance(leftImgEl, 'top-left');
	}
	if (rightImgEl) {
		rightLuminance = sampleLuminance(rightImgEl, 'top-right');
	}
}

function handleImageLoad(side: 'left' | 'right') {
	// Sample after image loads
	requestAnimationFrame(() => {
		if (side === 'left' && leftImgEl) {
			leftLuminance = sampleLuminance(leftImgEl, 'top-left');
		} else if (side === 'right' && rightImgEl) {
			rightLuminance = sampleLuminance(rightImgEl, 'top-right');
		}
	});
}

onMount(() => {
	// Sample on mount in case images are already cached/loaded
	requestAnimationFrame(sampleImages);
});

function toggleImage() {
	showLeft = !showLeft;
}

function handleKeydown(event: KeyboardEvent) {
	if (mode === 'toggle' && (event.key === ' ' || event.key === 'Enter')) {
		event.preventDefault();
		toggleImage();
	}
}

function handleSliderChange(event: Event) {
	const slider = event.currentTarget as HTMLElement & { value: number };
	sliderPosition = slider.value;
}
</script>

<div class="shader-compare">
	<!-- Slider mode -->
	<div class="mode-wrapper" class:active={mode === 'slider'}>
		<div class="slider-wrapper">
			<img-comparison-slider class="slider-container" onslide={handleSliderChange}>
				<div slot="first" class="slider-slot">
					<img
						bind:this={leftImgEl}
						src={leftImage}
						alt={leftShader?.name ?? 'Left'}
						onload={() => handleImageLoad('left')}
					/>
					{#if leftShader}
						<ShaderInfoOverlay
							shader={leftShader}
							position="top-left"
							visible={showLeftOverlay}
							targetLuminances={toElementLuminances(leftLuminance)}
						/>
					{/if}
				</div>
				<div slot="second" class="slider-slot">
					<img
						bind:this={rightImgEl}
						src={rightImage}
						alt={rightShader?.name ?? 'Right'}
						onload={() => handleImageLoad('right')}
					/>
					{#if rightShader}
						<ShaderInfoOverlay
							shader={rightShader}
							position="top-right"
							visible={showRightOverlay}
							targetLuminances={toElementLuminances(rightLuminance)}
						/>
					{/if}
				</div>
			</img-comparison-slider>
		</div>
	</div>

	<!-- Split mode -->
	<div class="mode-wrapper" class:active={mode === 'split'}>
		<SplitCanvas {leftImage} {rightImage} {leftShader} {rightShader} />
	</div>

	<!-- Toggle mode -->
	<div class="mode-wrapper" class:active={mode === 'toggle'}>
		<button
			type="button"
			class="toggle-container"
			onclick={toggleImage}
			onkeydown={handleKeydown}
			aria-label="Toggle between images"
		>
			<div class="toggle-images">
				<img
					bind:this={leftImgEl}
					src={leftImage}
					alt={leftShader?.name ?? 'Left'}
					class="toggle-image"
					class:visible={showLeft}
					class:hidden={!showLeft}
					onload={() => handleImageLoad('left')}
				/>
				<img
					bind:this={rightImgEl}
					src={rightImage}
					alt={rightShader?.name ?? 'Right'}
					class="toggle-image"
					class:visible={!showLeft}
					class:hidden={showLeft}
					onload={() => handleImageLoad('right')}
				/>
			</div>

			{#if leftShader && rightShader}
				<ShaderInfoOverlay
					shader={showLeft ? leftShader : rightShader}
					position="top-right"
					visible={true}
					targetLuminances={toElementLuminances(showLeft ? leftLuminance : rightLuminance)}
				/>
			{/if}

			<span class="toggle-hint">Click to toggle</span>
		</button>
	</div>
</div>

<style>
	.shader-compare {
		display: grid;
		width: 100%;
		max-width: 100%;
		border-radius: var(--radius);
		overflow: hidden;
		background: var(--muted);
	}

	/* Mode wrapper for CSS fade transitions - all modes overlap in same grid cell */
	.mode-wrapper {
		grid-area: 1 / 1;
		opacity: 0;
		pointer-events: none;
		transition: opacity 200ms ease;
	}

	.mode-wrapper.active {
		opacity: 1;
		pointer-events: auto;
	}

	/* Slider mode */
	.slider-wrapper {
		position: relative;
	}

	.slider-container {
		--divider-width: 3px;
		--divider-color: hsl(var(--primary));
		--default-handle-width: 44px;
		--default-handle-color: hsl(var(--primary));
		--default-handle-opacity: 1;
		--default-handle-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
		width: 100%;
		display: block;
	}

	.slider-slot {
		position: relative;
		width: 100%;
		height: 100%;
	}

	.slider-slot :global(.shader-overlay) {
		line-height: normal;
	}

	.slider-slot img,
	.slider-container img {
		width: 100%;
		height: auto;
		display: block;
		object-fit: cover;
	}

	/* Toggle mode */
	.toggle-container {
		position: relative;
		width: 100%;
		cursor: pointer;
		background: none;
		border: none;
		padding: 0;
		display: block;
	}

	.toggle-container:focus-visible {
		outline: 2px solid hsl(var(--ring));
		outline-offset: 2px;
	}

	.toggle-images {
		position: relative;
		width: 100%;
	}

	.toggle-image {
		width: 100%;
		height: auto;
		display: block;
		object-fit: cover;
		transition:
			opacity 150ms ease-in-out,
			transform 150ms ease-in-out;
	}

	.toggle-image.visible {
		opacity: 1;
		position: relative;
	}

	.toggle-image.hidden {
		opacity: 0;
		position: absolute;
		top: 0;
		left: 0;
	}

	.toggle-hint {
		position: absolute;
		bottom: 0.75rem;
		left: 0.75rem;
		color: white;
		padding: 0.25rem 0.5rem;
		font-size: 0.75rem;
		text-shadow:
			0 1px 2px rgba(0, 0, 0, 0.8),
			0 0 8px rgba(0, 0, 0, 0.5);
		pointer-events: none;
	}
</style>
