<script lang="ts">
import { formatVersion } from '$lib/utils/display';
import { onMount } from 'svelte';
import { fade } from 'svelte/transition';
import { LUMINANCE } from './constants';
import type { ElementBounds, ElementLuminances, ShaderDisplayInfo } from './types';

interface Props {
	shader: ShaderDisplayInfo;
	position?: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
	visible?: boolean;
	/** Per-element target background luminances (0-1). Each element adapts independently. */
	targetLuminances?: ElementLuminances;
	/** Callback fired when element bounds change (e.g., after mount or resize). */
	onBoundsChange?: (bounds: ElementBounds) => void;
	/** Element to compute bounds relative to (for accurate canvas sampling) */
	relativeTo?: HTMLElement;
}

let {
	shader,
	position = 'top-left',
	visible = true,
	targetLuminances,
	onBoundsChange,
	relativeTo
}: Props = $props();

// Element references for bound measurement
let containerRef: HTMLDivElement | undefined = $state();
let nameRef: HTMLDivElement | undefined = $state();
let metaRef: HTMLDivElement | undefined = $state();
let authorRef: HTMLDivElement | undefined = $state();

// Per-element color state: 0 = black text, 1 = white text
let nameBrightness = $state(1);
let metaBrightness = $state(1);
let authorBrightness = $state(1);

let nameTargetBrightness = $state(1);
let metaTargetBrightness = $state(1);
let authorTargetBrightness = $state(1);

let animationId: number | null = null;
let lastTime = 0;

// Determine target brightness for a single element with hysteresis
function computeTargetBrightness(luminance: number | undefined, currentTarget: number): number {
	if (luminance === undefined) return currentTarget;

	// Apply bias: positive bias makes luminance appear darker, favoring white text
	const adjustedLuminance = luminance - LUMINANCE.BIAS;

	if (currentTarget === 1 && adjustedLuminance > LUMINANCE.HIGH_THRESHOLD) {
		return 0; // Switch to black text
	} else if (currentTarget === 0 && adjustedLuminance < LUMINANCE.LOW_THRESHOLD) {
		return 1; // Switch to white text
	}
	return currentTarget;
}

// Update targets when luminances change
$effect(() => {
	if (!targetLuminances) return;

	nameTargetBrightness = computeTargetBrightness(targetLuminances.name, nameTargetBrightness);
	metaTargetBrightness = computeTargetBrightness(targetLuminances.meta, metaTargetBrightness);
	authorTargetBrightness = computeTargetBrightness(targetLuminances.author, authorTargetBrightness);
});

// Generate color from brightness value
function brightnessToColor(brightness: number): string {
	const value = Math.round(brightness * 255);
	return `rgb(${value}, ${value}, ${value})`;
}

// Generate shadow from brightness value
function brightnessToShadow(brightness: number): string {
	return brightness > 0.5 ? '0 1px 2px rgba(0, 0, 0, 0.7), 0 0 4px rgba(0, 0, 0, 0.7)' : 'none';
}

// Derived colors and shadows for each element
const nameColor = $derived(brightnessToColor(nameBrightness));
const metaColor = $derived(brightnessToColor(metaBrightness));
const authorColor = $derived(brightnessToColor(authorBrightness));

const nameShadow = $derived(brightnessToShadow(nameBrightness));
const metaShadow = $derived(brightnessToShadow(metaBrightness));
const authorShadow = $derived(brightnessToShadow(authorBrightness));

const displayAuthor = $derived(shader.author ?? 'Unknown Author');
const displayMeta = $derived(
	[formatVersion(shader.version), shader.profile_name, shader.preset_name]
		.filter(Boolean)
		.join(' \u2022 ')
);

// Chase a single value toward its target
function chaseValue(current: number, target: number, dt: number): number {
	const distance = Math.abs(target - current);
	if (distance < LUMINANCE.EPSILON) return target;

	const speed = Math.max(LUMINANCE.MIN_SPEED, distance * LUMINANCE.CHASE_FACTOR);
	const step = speed * dt;
	const direction = target > current ? 1 : -1;

	if (step >= distance) return target;
	return current + direction * step;
}

function animate(time: number) {
	const dt = lastTime ? (time - lastTime) / 1000 : 0.016;
	lastTime = time;

	// Chase all three values
	nameBrightness = chaseValue(nameBrightness, nameTargetBrightness, dt);
	metaBrightness = chaseValue(metaBrightness, metaTargetBrightness, dt);
	authorBrightness = chaseValue(authorBrightness, authorTargetBrightness, dt);

	// Check if all animations are complete
	const allDone =
		nameBrightness === nameTargetBrightness &&
		metaBrightness === metaTargetBrightness &&
		authorBrightness === authorTargetBrightness;

	if (allDone) {
		animationId = null;
		return;
	}

	animationId = requestAnimationFrame(animate);
}

// Start animation when any target changes
$effect(() => {
	// Track all targets to restart animation when any changes
	const _ = [nameTargetBrightness, metaTargetBrightness, authorTargetBrightness];

	const needsAnimation =
		targetLuminances !== undefined &&
		!animationId &&
		(nameBrightness !== nameTargetBrightness ||
			metaBrightness !== metaTargetBrightness ||
			authorBrightness !== authorTargetBrightness);

	if (needsAnimation) {
		lastTime = 0;
		animationId = requestAnimationFrame(animate);
	}
});

// Report element bounds to parent
function reportBounds() {
	if (!onBoundsChange) return;

	const reference = relativeTo ?? containerRef;
	if (!reference) return;
	const referenceRect = reference.getBoundingClientRect();

	function getRelativeBounds(el: HTMLElement | undefined): DOMRect | null {
		if (!el) return null;
		const rect = el.getBoundingClientRect();
		return new DOMRect(
			rect.left - referenceRect.left,
			rect.top - referenceRect.top,
			rect.width,
			rect.height
		);
	}

	onBoundsChange({
		name: getRelativeBounds(nameRef),
		meta: displayMeta ? getRelativeBounds(metaRef) : null,
		author: getRelativeBounds(authorRef)
	});
}

onMount(() => {
	// Report initial bounds after render
	requestAnimationFrame(() => {
		reportBounds();
	});

	// Observe resize to update bounds
	const observer = new ResizeObserver(() => {
		reportBounds();
	});
	if (containerRef) {
		observer.observe(containerRef);
	}

	return () => {
		observer.disconnect();
		if (animationId) {
			cancelAnimationFrame(animationId);
		}
	};
});

// Re-report bounds when shader changes
$effect(() => {
	const _ = shader;
	requestAnimationFrame(() => {
		reportBounds();
	});
});
</script>

{#if visible}
	<div
		class="shader-overlay {position}"
		transition:fade={{ duration: 150 }}
		bind:this={containerRef}
	>
		<div
			class="shader-name"
			bind:this={nameRef}
			style:color={targetLuminances ? nameColor : undefined}
			style:text-shadow={targetLuminances ? nameShadow : undefined}
		>
			{shader.name}
		</div>
		{#if displayMeta}
			<div
				class="shader-meta"
				bind:this={metaRef}
				style:color={targetLuminances ? metaColor : undefined}
				style:text-shadow={targetLuminances ? metaShadow : undefined}
			>
				{displayMeta}
			</div>
		{/if}
		<div
			class="shader-author"
			bind:this={authorRef}
			style:color={targetLuminances ? authorColor : undefined}
			style:text-shadow={targetLuminances ? authorShadow : undefined}
		>
			{displayAuthor}
		</div>
	</div>
{/if}

<style>
	.shader-overlay {
		position: absolute;
		padding: 0.5rem 0.75rem;
		color: white;
		text-shadow:
			0 1px 2px rgba(0, 0, 0, 0.7),
			0 0 4px rgba(0, 0, 0, 0.7);
		pointer-events: none;
		z-index: 10;
		max-width: 45%;
	}

	.shader-overlay.top-left {
		top: 0.75rem;
		left: 0.75rem;
	}

	.shader-overlay.top-right {
		top: 0.75rem;
		right: 0.75rem;
		text-align: right;
	}

	.shader-overlay.bottom-left {
		bottom: 0.75rem;
		left: 0.75rem;
	}

	.shader-overlay.bottom-right {
		bottom: 0.75rem;
		right: 0.75rem;
		text-align: right;
	}

	.shader-name {
		font-weight: 600;
		font-size: 1rem;
		line-height: 1.2;
	}

	.shader-meta {
		font-size: 0.75rem;
		opacity: 0.9;
		margin-top: 0.125rem;
	}

	.shader-author {
		font-size: 0.7rem;
		opacity: 0.7;
		margin-top: 0.125rem;
	}

	@media (max-width: 640px) {
		.shader-overlay {
			padding: 0.375rem 0.5rem;
		}

		.shader-name {
			font-size: 0.875rem;
		}

		.shader-meta {
			font-size: 0.675rem;
		}

		.shader-author {
			font-size: 0.625rem;
		}
	}
</style>
