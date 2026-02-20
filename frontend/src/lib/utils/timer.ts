/**
 * Managed timeout wrapper that prevents leaks and simplifies cleanup.
 * Each ManagedTimer holds at most one active timeout at a time.
 */

interface ManagedTimer {
	/** Schedule a callback after `ms` milliseconds. Cancels any pending timeout. */
	set: (callback: () => void, ms: number) => void;
	/** Cancel the pending timeout if any. */
	clear: () => void;
	/** Whether a timeout is currently pending. */
	readonly pending: boolean;
}

/** Create a managed timer. Call `clear()` on component destroy to prevent leaks. */
function createTimer(): ManagedTimer {
	let id: ReturnType<typeof setTimeout> | null = null;

	function set(callback: () => void, ms: number) {
		clear();
		id = setTimeout(() => {
			id = null;
			callback();
		}, ms);
	}

	function clear() {
		if (id !== null) {
			clearTimeout(id);
			id = null;
		}
	}

	return {
		set,
		clear,
		get pending() {
			return id !== null;
		}
	};
}

/**
 * Create a group of named timers that can be cleared individually or all at once.
 * Useful when a component manages multiple independent timeouts.
 */
export function createTimerGroup<K extends string>(
	...keys: K[]
): Record<K, ManagedTimer> & { clearAll: () => void } {
	const timers = {} as Record<K, ManagedTimer>;
	for (const key of keys) {
		timers[key] = createTimer();
	}
	return Object.assign(timers, {
		clearAll() {
			for (const key of keys) {
				timers[key].clear();
			}
		}
	});
}
