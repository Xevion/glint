interface CacheEntry {
	buffer: Uint8Array;
	createdAt: number;
}

const DEFAULT_MAX_ENTRIES = 100;
const DEFAULT_MAX_AGE_MS = 60 * 60 * 1000; // 1 hour

/**
 * Simple LRU cache for generated OG images.
 * Bounded by entry count and TTL.
 */
class OgImageCache {
	private cache = new Map<string, CacheEntry>();
	private maxEntries: number;
	private maxAgeMs: number;

	constructor(maxEntries = DEFAULT_MAX_ENTRIES, maxAgeMs = DEFAULT_MAX_AGE_MS) {
		this.maxEntries = maxEntries;
		this.maxAgeMs = maxAgeMs;
	}

	get(key: string): Uint8Array | null {
		const entry = this.cache.get(key);
		if (!entry) return null;

		if (Date.now() - entry.createdAt > this.maxAgeMs) {
			this.cache.delete(key);
			return null;
		}

		// Move to end for LRU ordering (Map preserves insertion order)
		this.cache.delete(key);
		this.cache.set(key, entry);
		return entry.buffer;
	}

	set(key: string, buffer: Uint8Array): void {
		// Evict oldest entries if at capacity
		while (this.cache.size >= this.maxEntries) {
			const oldest = this.cache.keys().next();
			if (oldest.done) break;
			this.cache.delete(oldest.value);
		}

		this.cache.set(key, { buffer, createdAt: Date.now() });
	}
}

/**
 * Singleflight gate — deduplicates concurrent requests for the same key.
 * If a generation is already in-flight, subsequent requests await the same promise.
 */
class Singleflight<T = Uint8Array> {
	private inflight = new Map<string, Promise<T>>();

	/**
	 * Execute `fn` for `key`, deduplicating concurrent calls.
	 * If a call for the same key is already in-flight, returns its result.
	 */
	async do(key: string, fn: () => Promise<T>): Promise<T> {
		const existing = this.inflight.get(key);
		if (existing) return existing;

		const promise = fn().finally(() => {
			this.inflight.delete(key);
		});

		this.inflight.set(key, promise);
		return promise;
	}
}

/** Shared instances for the OG image endpoint */
export const ogCache = new OgImageCache();

interface OgGenerationResult {
	buffer: Uint8Array;
	fallback: boolean;
}

export const ogSingleflight = new Singleflight<OgGenerationResult>();
