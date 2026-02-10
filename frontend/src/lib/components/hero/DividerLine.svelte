<script lang="ts">
import { type Orientation, SKEW_DEG } from './types';

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

const isHorizontal = $derived(orientation === 'horizontal');

/** Computed positioning styles for both the visible line and the cursor hitzone. */
const lineStyle = $derived.by(() => {
	const pos = `${position * 100}%`;

	if (orientation === 'diagonal') {
		return {
			base: `left: ${pos}; top: 50%; width: 1px; height: ${diagonalHeight}; transform: translate(-50%, -50%) rotate(${lineAngle}deg); transform-origin: center center;`,
			hitzone: `left: ${pos}; top: 50%; width: 32px; height: ${diagonalHeight}; transform: translate(-50%, -50%) rotate(${lineAngle}deg); transform-origin: center center;`
		};
	}

	if (isHorizontal) {
		return {
			base: `top: ${pos}; left: 0; right: 0; height: 1px; transform: translateY(-50%);`,
			hitzone: `top: ${pos}; left: 0; right: 0; height: 32px; transform: translateY(-50%);`
		};
	}

	// vertical (default)
	return {
		base: `left: ${pos}; top: 0; bottom: 0; width: 1px; transform: translateX(-50%);`,
		hitzone: `left: ${pos}; top: 0; bottom: 0; width: 32px; transform: translateX(-50%);`
	};
});
</script>

<div
	class="divider-line pointer-events-none absolute z-10"
	style={lineStyle.base}
>
	<div class="h-full w-full bg-white/40"></div>
</div>
{#if cursor}
	<div
		class="absolute z-10 {cursor}"
		style={lineStyle.hitzone}
	></div>
{/if}
