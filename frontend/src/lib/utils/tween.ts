/**
 * Lightweight tween engine using requestAnimationFrame.
 * Returns controls for animating a numeric value over time.
 */

function easeInOutCubic(t: number): number {
	return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
}

interface TweenControls {
	/** Animate from current value to target over duration (ms). Rejects if cancelled. */
	tweenTo: (
		target: number,
		duration: number,
		onUpdate: (value: number) => void,
		getCurrentValue: () => number
	) => Promise<void>;
	/** Cancel any in-flight tween. */
	cancel: () => void;
}

/**
 * Create a tween controller. Only one tween runs at a time —
 * starting a new one implicitly cancels the previous.
 */
export function createTween(): TweenControls {
	let rafId: number | null = null;
	let cancelCurrent: (() => void) | null = null;

	function cancel() {
		if (rafId !== null) {
			cancelAnimationFrame(rafId);
			rafId = null;
		}
		cancelCurrent?.();
		cancelCurrent = null;
	}

	function tweenTo(
		target: number,
		duration: number,
		onUpdate: (value: number) => void,
		getCurrentValue: () => number
	): Promise<void> {
		return new Promise((resolve, reject) => {
			cancel();

			const start = getCurrentValue();
			const delta = target - start;

			if (Math.abs(delta) < 0.001) {
				onUpdate(target);
				resolve();
				return;
			}

			const startTime = performance.now();
			let cancelled = false;

			cancelCurrent = () => {
				cancelled = true;
			};

			function tick(now: number) {
				if (cancelled) {
					reject(new Error('cancelled'));
					return;
				}

				const elapsed = now - startTime;
				const progress = Math.min(1, elapsed / duration);
				const eased = easeInOutCubic(progress);
				const value = start + delta * eased;

				onUpdate(value);

				if (progress < 1) {
					rafId = requestAnimationFrame(tick);
				} else {
					onUpdate(target);
					rafId = null;
					cancelCurrent = null;
					resolve();
				}
			}

			rafId = requestAnimationFrame(tick);
		});
	}

	return { tweenTo, cancel };
}
