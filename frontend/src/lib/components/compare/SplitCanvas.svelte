<script lang="ts">
import { onMount } from 'svelte';
import ShaderInfoOverlay from './ShaderInfoOverlay.svelte';
import { LUMINANCE, TOUCH, ZOOM } from './constants';
import type { ElementBounds, ElementLuminances, ShaderDisplayInfo } from './types';

interface Props {
	leftImage: string;
	rightImage: string;
	leftShader?: ShaderDisplayInfo;
	rightShader?: ShaderDisplayInfo;
}

let { leftImage, rightImage, leftShader, rightShader }: Props = $props();

let canvas: HTMLCanvasElement | undefined = $state();
let container: HTMLDivElement | undefined = $state();
let ctx: CanvasRenderingContext2D | null = null;

// Loaded images
let leftImg: HTMLImageElement | null = $state(null);
let rightImg: HTMLImageElement | null = $state(null);

// Viewport state
let viewport = $state({ x: 0, y: 0, zoom: 1 });

// Canvas dimensions
let canvasWidth = $state(800);
let canvasHeight = $state(450);

// Interaction state
let interaction = $state({
	isDragging: false,
	lastMouseX: 0,
	lastMouseY: 0
});

// Touch state
let touch = $state({
	lastDistance: 0,
	lastCenterX: 0,
	lastCenterY: 0,
	lastTapTime: 0
});

// Per-element luminance sampling state
let leftLuminances = $state<ElementLuminances>({});
let rightLuminances = $state<ElementLuminances>({});
let leftBounds = $state<ElementBounds | null>(null);
let rightBounds = $state<ElementBounds | null>(null);
let sampleThrottleId: ReturnType<typeof setTimeout> | null = null;

// Image load error state
let loadError = $state<string | null>(null);

// Zoom constraints
const getMinZoom = () => calculateCoverZoom();

// Load image helper
function loadImage(src: string): Promise<HTMLImageElement> {
	return new Promise((resolve, reject) => {
		const img = new Image();
		img.crossOrigin = 'anonymous';
		img.onload = () => resolve(img);
		img.onerror = () => reject(new Error(`Failed to load image: ${src}`));
		img.src = src;
	});
}

// Calculate zoom to cover canvas (no empty space)
function calculateCoverZoom(): number {
	if (!leftImg) return 1;
	const scaleX = canvasWidth / leftImg.width;
	const scaleY = canvasHeight / leftImg.height;
	return Math.max(scaleX, scaleY);
}

/**
 * Sample luminance from the canvas at a given region.
 * Returns 0-1 where 0 is black and 1 is white.
 */
function sampleLuminanceFromCanvas(
	x: number,
	y: number,
	width: number,
	height: number
): number | undefined {
	if (!ctx || width <= 0 || height <= 0) return undefined;

	const dpr = window.devicePixelRatio || 1;

	// Clamp coordinates to canvas bounds
	const clampedX = Math.max(0, Math.min(x, canvasWidth - 1));
	const clampedY = Math.max(0, Math.min(y, canvasHeight - 1));
	const clampedWidth = Math.min(width, canvasWidth - clampedX);
	const clampedHeight = Math.min(height, canvasHeight - clampedY);

	if (clampedWidth <= 0 || clampedHeight <= 0) return undefined;

	try {
		const imageData = ctx.getImageData(
			clampedX * dpr,
			clampedY * dpr,
			clampedWidth * dpr,
			clampedHeight * dpr
		);
		const data = imageData.data;

		let totalLuminance = 0;
		const pixelCount = data.length / 4;

		for (let i = 0; i < data.length; i += 4) {
			const r = data[i] / 255;
			const g = data[i + 1] / 255;
			const b = data[i + 2] / 255;
			// ITU-R BT.709 luminance formula
			totalLuminance += 0.2126 * r + 0.7152 * g + 0.0722 * b;
		}

		return totalLuminance / pixelCount;
	} catch (e) {
		console.warn('[SplitCanvas] Luminance sampling failed:', e);
		return undefined;
	}
}

/**
 * Sample luminance for each element in bounds.
 * Bounds are already relative to canvas container - direct use.
 */
function sampleElementLuminances(bounds: ElementBounds | null): ElementLuminances {
	if (!bounds) return {};
	const result: ElementLuminances = {};

	function sampleElement(rect: DOMRect | null, key: keyof ElementLuminances): void {
		if (!rect) return;
		// Bounds are already relative to canvas container - direct use
		const luminance = sampleLuminanceFromCanvas(rect.x, rect.y, rect.width, rect.height);
		if (luminance !== undefined) {
			result[key] = luminance;
		}
	}

	sampleElement(bounds.name, 'name');
	sampleElement(bounds.meta, 'meta');
	sampleElement(bounds.author, 'author');
	return result;
}

/**
 * Schedule luminance sampling (throttled).
 */
function scheduleSampling() {
	if (sampleThrottleId) return;

	sampleThrottleId = setTimeout(() => {
		sampleThrottleId = null;

		// Sample luminance for each text element
		leftLuminances = sampleElementLuminances(leftBounds);
		rightLuminances = sampleElementLuminances(rightBounds);
	}, LUMINANCE.SAMPLE_THROTTLE_MS);
}

// Handle bounds updates from overlays
function handleLeftBoundsChange(bounds: ElementBounds) {
	leftBounds = bounds;
	scheduleSampling();
}

function handleRightBoundsChange(bounds: ElementBounds) {
	rightBounds = bounds;
	scheduleSampling();
}

// Clamp viewport to image bounds
function clampViewport() {
	if (!leftImg) return;

	const imgWidth = leftImg.width * viewport.zoom;
	const imgHeight = leftImg.height * viewport.zoom;

	const maxX = Math.max(0, (imgWidth - canvasWidth) / 2);
	const maxY = Math.max(0, (imgHeight - canvasHeight) / 2);

	viewport.x = Math.max(-maxX, Math.min(maxX, viewport.x));
	viewport.y = Math.max(-maxY, Math.min(maxY, viewport.y));
}

// Draw the canvas
function draw() {
	if (!ctx || !leftImg || !rightImg) return;

	const dpr = window.devicePixelRatio || 1;
	ctx.clearRect(0, 0, canvasWidth * dpr, canvasHeight * dpr);

	const imgWidth = leftImg.width * viewport.zoom;
	const imgHeight = leftImg.height * viewport.zoom;

	const imgX = (canvasWidth - imgWidth) / 2 + viewport.x;
	const imgY = (canvasHeight - imgHeight) / 2 + viewport.y;

	ctx.save();
	ctx.scale(dpr, dpr);

	// Draw left half
	ctx.save();
	ctx.beginPath();
	ctx.rect(0, 0, canvasWidth / 2, canvasHeight);
	ctx.clip();
	ctx.drawImage(leftImg, imgX, imgY, imgWidth, imgHeight);
	ctx.restore();

	// Draw right half
	ctx.save();
	ctx.beginPath();
	ctx.rect(canvasWidth / 2, 0, canvasWidth / 2, canvasHeight);
	ctx.clip();
	ctx.drawImage(rightImg, imgX, imgY, imgWidth, imgHeight);
	ctx.restore();

	// Draw center divider
	ctx.strokeStyle = 'hsl(var(--primary))';
	ctx.lineWidth = 2;
	ctx.beginPath();
	ctx.moveTo(canvasWidth / 2, 0);
	ctx.lineTo(canvasWidth / 2, canvasHeight);
	ctx.stroke();

	ctx.restore();

	// Schedule luminance sampling after draw
	scheduleSampling();
}

// Handle mouse down
function handleMouseDown(e: MouseEvent) {
	interaction.isDragging = true;
	interaction.lastMouseX = e.clientX;
	interaction.lastMouseY = e.clientY;
	if (canvas) canvas.style.cursor = 'grabbing';
}

// Handle mouse move
function handleMouseMove(e: MouseEvent) {
	if (!interaction.isDragging) return;

	const deltaX = e.clientX - interaction.lastMouseX;
	const deltaY = e.clientY - interaction.lastMouseY;

	viewport.x += deltaX;
	viewport.y += deltaY;

	interaction.lastMouseX = e.clientX;
	interaction.lastMouseY = e.clientY;

	clampViewport();
	draw();
}

// Handle mouse up
function handleMouseUp() {
	interaction.isDragging = false;
	if (canvas) canvas.style.cursor = 'grab';
}

// Handle wheel (zoom)
function handleWheel(e: WheelEvent) {
	e.preventDefault();
	if (!canvas) return;

	const rect = canvas.getBoundingClientRect();
	const mouseX = e.clientX - rect.left;
	const mouseY = e.clientY - rect.top;

	const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
	const newZoom = Math.max(getMinZoom(), Math.min(ZOOM.MAX, viewport.zoom * zoomFactor));

	if (newZoom !== viewport.zoom) {
		const zoomRatio = newZoom / viewport.zoom;
		const centerX = canvasWidth / 2;
		const centerY = canvasHeight / 2;

		viewport.x = (viewport.x - (mouseX - centerX)) * zoomRatio + (mouseX - centerX);
		viewport.y = (viewport.y - (mouseY - centerY)) * zoomRatio + (mouseY - centerY);

		viewport.zoom = newZoom;
		clampViewport();
		draw();
	}
}

// Touch helpers
function getTouchDistance(touches: TouchList): number {
	if (touches.length < 2) return 0;
	const dx = touches[0].clientX - touches[1].clientX;
	const dy = touches[0].clientY - touches[1].clientY;
	return Math.sqrt(dx * dx + dy * dy);
}

function getTouchCenter(touches: TouchList): { x: number; y: number } {
	if (touches.length < 2) {
		return { x: touches[0].clientX, y: touches[0].clientY };
	}
	return {
		x: (touches[0].clientX + touches[1].clientX) / 2,
		y: (touches[0].clientY + touches[1].clientY) / 2
	};
}

// Handle touch start
function handleTouchStart(e: TouchEvent) {
	e.preventDefault();

	if (e.touches.length === 1) {
		const now = Date.now();
		if (now - touch.lastTapTime < TOUCH.DOUBLE_TAP_WINDOW) {
			const coverZoom = calculateCoverZoom();
			const zoomedIn = viewport.zoom > coverZoom * ZOOM.ZOOMED_IN_THRESHOLD;
			if (zoomedIn) {
				viewport.zoom = coverZoom;
				viewport.x = 0;
				viewport.y = 0;
			} else {
				viewport.zoom = Math.min(coverZoom * ZOOM.DOUBLE_TAP_MULTIPLIER, ZOOM.MAX);
			}
			clampViewport();
			draw();
			touch.lastTapTime = 0;
			return;
		}
		touch.lastTapTime = now;

		interaction.isDragging = true;
		interaction.lastMouseX = e.touches[0].clientX;
		interaction.lastMouseY = e.touches[0].clientY;
	} else if (e.touches.length === 2) {
		interaction.isDragging = false;
		touch.lastDistance = getTouchDistance(e.touches);
		const center = getTouchCenter(e.touches);
		touch.lastCenterX = center.x;
		touch.lastCenterY = center.y;
	}
}

// Handle touch move
function handleTouchMove(e: TouchEvent) {
	e.preventDefault();

	if (e.touches.length === 1 && interaction.isDragging) {
		const deltaX = e.touches[0].clientX - interaction.lastMouseX;
		const deltaY = e.touches[0].clientY - interaction.lastMouseY;

		viewport.x += deltaX;
		viewport.y += deltaY;

		interaction.lastMouseX = e.touches[0].clientX;
		interaction.lastMouseY = e.touches[0].clientY;

		clampViewport();
		draw();
	} else if (e.touches.length === 2) {
		const newDistance = getTouchDistance(e.touches);
		const center = getTouchCenter(e.touches);

		if (touch.lastDistance > 0) {
			const scale = newDistance / touch.lastDistance;
			const newZoom = Math.max(getMinZoom(), Math.min(ZOOM.MAX, viewport.zoom * scale));

			if (newZoom !== viewport.zoom) {
				const rect = canvas!.getBoundingClientRect();
				const touchX = center.x - rect.left;
				const touchY = center.y - rect.top;

				const zoomRatio = newZoom / viewport.zoom;
				const centerX = canvasWidth / 2;
				const centerY = canvasHeight / 2;

				viewport.x = (viewport.x - (touchX - centerX)) * zoomRatio + (touchX - centerX);
				viewport.y = (viewport.y - (touchY - centerY)) * zoomRatio + (touchY - centerY);

				viewport.zoom = newZoom;
			}
		}

		const deltaX = center.x - touch.lastCenterX;
		const deltaY = center.y - touch.lastCenterY;
		viewport.x += deltaX;
		viewport.y += deltaY;

		touch.lastDistance = newDistance;
		touch.lastCenterX = center.x;
		touch.lastCenterY = center.y;

		clampViewport();
		draw();
	}
}

// Handle touch end
function handleTouchEnd(e: TouchEvent) {
	if (e.touches.length === 0) {
		interaction.isDragging = false;
		touch.lastDistance = 0;
	} else if (e.touches.length === 1) {
		interaction.isDragging = true;
		interaction.lastMouseX = e.touches[0].clientX;
		interaction.lastMouseY = e.touches[0].clientY;
		touch.lastDistance = 0;
	}
}

// Resize observer
function setupResizeObserver() {
	// eslint-disable-next-line @typescript-eslint/no-empty-function
	if (!container || !canvas) return () => {};

	const observer = new ResizeObserver((entries) => {
		for (const entry of entries) {
			const { width, height } = entry.contentRect;
			if (width > 0 && height > 0 && canvas) {
				canvasWidth = width;
				canvasHeight = height;

				const dpr = window.devicePixelRatio || 1;
				canvas.width = width * dpr;
				canvas.height = height * dpr;

				clampViewport();
				draw();
			}
		}
	});
	observer.observe(container);
	return () => observer.disconnect();
}

// Load images when URLs change
$effect(() => {
	const leftSrc = leftImage;
	const rightSrc = rightImage;

	loadError = null;
	Promise.all([loadImage(leftSrc), loadImage(rightSrc)])
		.then(([left, right]) => {
			leftImg = left;
			rightImg = right;

			viewport.zoom = calculateCoverZoom();
			viewport.x = 0;
			viewport.y = 0;

			draw();
		})
		.catch((err: Error) => {
			loadError = err.message;
			console.error('[SplitCanvas] Image load failed:', err);
		});
});

onMount(() => {
	if (!canvas) return;
	ctx = canvas.getContext('2d', { willReadFrequently: true });
	const cleanup = setupResizeObserver();

	canvas.style.cursor = 'grab';

	return () => {
		cleanup();
		if (sampleThrottleId) {
			clearTimeout(sampleThrottleId);
		}
	};
});
</script>

<div class="split-canvas-container" bind:this={container}>
	<canvas
		bind:this={canvas}
		onmousedown={handleMouseDown}
		onmousemove={handleMouseMove}
		onmouseup={handleMouseUp}
		onmouseleave={handleMouseUp}
		onwheel={handleWheel}
		ontouchstart={handleTouchStart}
		ontouchmove={handleTouchMove}
		ontouchend={handleTouchEnd}
	></canvas>

	{#if loadError}
		<div class="load-error">{loadError}</div>
	{/if}

	{#if leftShader}
		<ShaderInfoOverlay
			shader={leftShader}
			position="top-left"
			targetLuminances={leftLuminances}
			onBoundsChange={handleLeftBoundsChange}
			relativeTo={container}
		/>
	{/if}

	{#if rightShader}
		<ShaderInfoOverlay
			shader={rightShader}
			position="top-right"
			targetLuminances={rightLuminances}
			onBoundsChange={handleRightBoundsChange}
			relativeTo={container}
		/>
	{/if}
</div>

<style>
	.split-canvas-container {
		position: relative;
		width: 100%;
		aspect-ratio: 16 / 9;
		overflow: hidden;
		background: var(--muted);
		border-radius: var(--radius);
	}

	canvas {
		width: 100%;
		height: 100%;
		display: block;
		touch-action: none;
	}

	.load-error {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		color: hsl(var(--destructive));
		background: hsl(var(--background) / 0.9);
		padding: 0.5rem 1rem;
		border-radius: var(--radius);
		font-size: 0.875rem;
	}
</style>
