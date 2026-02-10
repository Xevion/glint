<script lang="ts">
import { SKEW_DEG, type Orientation } from './types';

interface Props {
	/** Divider position from 0 to 1 */
	position: number;
	orientation: Orientation;
	/** Container aspect ratio (width/height), used to compute diagonal angle */
	aspectRatio?: number;
	/** Tailwind cursor class for the handle zone (e.g. 'cursor-ew-resize'). Omit to hide the zone. */
	cursor?: string;
}

let { position, orientation, aspectRatio = 16 / 9, cursor }: Props = $props();

/**
 * Compute the angle (in degrees) the divider line should be drawn at.
 * For diagonal, the clip-path uses a polygon with `SKEW_DEG` degrees of offset,
 * so the line connects (pos+skew, 0) to (pos-skew, 100) — we derive the angle from that.
 */
const lineAngle = $derived.by(() => {
	if (orientation === 'diagonal') {
		return Math.atan(((2 * SKEW_DEG) / 100) * aspectRatio) * (180 / Math.PI);
	}
	return 0;
});

/** Height needed to cover the container after rotation: 1/cos(angle) scaled with margin */
const diagonalHeight = $derived.by(() => {
	const rad = lineAngle * (Math.PI / 180);
	return `${(100 / Math.cos(rad)).toFixed(1)}%`;
});
</script>

{#if orientation === 'vertical'}
	<div
		class="divider-line absolute top-0 bottom-0 z-10 pointer-events-none"
		style="left: {position * 100}%; width: 1px; transform: translateX(-50%);"
	>
		<div class="h-full w-full bg-white/40"></div>
	</div>
	{#if cursor}
		<div
			class="absolute top-0 bottom-0 z-10 {cursor}"
			style="left: {position * 100}%; width: 32px; transform: translateX(-50%);"
		></div>
	{/if}
{:else if orientation === 'horizontal'}
	<div
		class="divider-line absolute left-0 right-0 z-10 pointer-events-none"
		style="top: {position * 100}%; height: 1px; transform: translateY(-50%);"
	>
		<div class="h-full w-full bg-white/40"></div>
	</div>
	{#if cursor}
		<div
			class="absolute left-0 right-0 z-10 {cursor}"
			style="top: {position * 100}%; height: 32px; transform: translateY(-50%);"
		></div>
	{/if}
{:else}
	<!-- Diagonal: a tall vertical line rotated to match the clip-path skew -->
	<div
		class="divider-line absolute z-10 pointer-events-none"
		style="
			left: {position * 100}%;
			top: 50%;
			width: 1px;
			height: {diagonalHeight};
			transform: translate(-50%, -50%) rotate({lineAngle}deg);
			transform-origin: center center;
		"
	>
		<div class="h-full w-full bg-white/40"></div>
	</div>
	{#if cursor}
		<div
			class="absolute z-10 {cursor}"
			style="
				left: {position * 100}%;
				top: 50%;
				width: 32px;
				height: {diagonalHeight};
				transform: translate(-50%, -50%) rotate({lineAngle}deg);
				transform-origin: center center;
			"
		></div>
	{/if}
{/if}
