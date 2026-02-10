<script lang="ts">
export type Orientation = 'vertical' | 'horizontal' | 'diagonal';

interface Props {
	/** Divider position from 0 to 1 */
	position: number;
	orientation: Orientation;
	disabled?: boolean;
	onDrag?: (position: number) => void;
	onDragStart?: () => void;
	onDragEnd?: () => void;
}

let { position, orientation, disabled = false, onDrag, onDragStart, onDragEnd }: Props = $props();

let containerEl: HTMLDivElement | undefined = $state();
let isDragging = $state(false);

function getPositionFromEvent(clientX: number, clientY: number): number {
	if (!containerEl) return position;
	const parent = containerEl.parentElement;
	if (!parent) return position;

	const rect = parent.getBoundingClientRect();

	if (orientation === 'vertical' || orientation === 'diagonal') {
		return Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
	} else {
		return Math.max(0, Math.min(1, (clientY - rect.top) / rect.height));
	}
}

function handlePointerDown(e: PointerEvent) {
	if (disabled) return;
	e.preventDefault();
	isDragging = true;
	(e.target as HTMLElement).setPointerCapture(e.pointerId);
	onDragStart?.();
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
	(e.target as HTMLElement).releasePointerCapture(e.pointerId);
	onDragEnd?.();
}

const lineStyle = $derived.by(() => {
	const pct = `${position * 100}%`;

	if (orientation === 'vertical') {
		return `left: ${pct}; top: 0; bottom: 0; width: 2px; transform: translateX(-50%); cursor: ew-resize;`;
	} else if (orientation === 'horizontal') {
		return `top: ${pct}; left: 0; right: 0; height: 2px; transform: translateY(-50%); cursor: ns-resize;`;
	} else {
		// Diagonal: position the handle at the divider line center
		return `left: ${pct}; top: 0; bottom: 0; width: 2px; transform: translateX(-50%); cursor: ew-resize;`;
	}
});

const gripStyle = $derived.by(() => {
	if (orientation === 'horizontal') {
		return 'left: 50%; top: 50%; transform: translate(-50%, -50%);';
	}
	return 'left: 50%; top: 50%; transform: translate(-50%, -50%);';
});

const isVerticalLike = $derived(orientation === 'vertical' || orientation === 'diagonal');
</script>

<div
	bind:this={containerEl}
	class="divider-handle absolute z-10 select-none"
	class:active={isDragging}
	style={lineStyle}
	role="slider"
	aria-valuenow={Math.round(position * 100)}
	aria-valuemin={0}
	aria-valuemax={100}
	aria-label="Comparison divider"
	tabindex={disabled ? -1 : 0}
	onpointerdown={handlePointerDown}
	onpointermove={handlePointerMove}
	onpointerup={handlePointerUp}
	onpointercancel={handlePointerUp}
>
	<!-- Wider invisible hit area -->
	<div
		class="absolute"
		class:hit-area-vertical={isVerticalLike}
		class:hit-area-horizontal={!isVerticalLike}
	></div>

	<!-- Visible line -->
	<div
		class="absolute bg-white/90 shadow-sm"
		class:line-vertical={isVerticalLike}
		class:line-horizontal={!isVerticalLike}
	></div>

	<!-- Grip circle -->
	<div
		class="absolute z-20 flex items-center justify-center rounded-full bg-white shadow-md transition-transform"
		class:scale-110={isDragging}
		style={gripStyle}
		style:width="32px"
		style:height="32px"
	>
		{#if isVerticalLike}
			<svg width="12" height="16" viewBox="0 0 12 16" fill="none" class="text-gray-500">
				<path d="M4 0v16M8 0v16" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
			</svg>
		{:else}
			<svg width="16" height="12" viewBox="0 0 16 12" fill="none" class="text-gray-500">
				<path d="M0 4h16M0 8h16" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
			</svg>
		{/if}
	</div>
</div>

<style>
	.hit-area-vertical {
		top: 0;
		bottom: 0;
		left: 50%;
		width: 32px;
		transform: translateX(-50%);
	}

	.hit-area-horizontal {
		left: 0;
		right: 0;
		top: 50%;
		height: 32px;
		transform: translateY(-50%);
	}

	.line-vertical {
		top: 0;
		bottom: 0;
		left: 50%;
		width: 2px;
		transform: translateX(-50%);
	}

	.line-horizontal {
		left: 0;
		right: 0;
		top: 50%;
		height: 2px;
		transform: translateY(-50%);
	}

	.divider-handle {
		touch-action: none;
	}
</style>
