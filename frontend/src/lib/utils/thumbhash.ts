import { thumbHashToDataURL } from 'thumbhash';

const MAX_CACHE_SIZE = 500;
const cache = new Map<string, string>();

export function decodeThumbhash(base64: string | null | undefined): string | null {
	if (!base64) return null;

	const cached = cache.get(base64);
	if (cached) {
		// Move to end for LRU behavior
		cache.delete(base64);
		cache.set(base64, cached);
		return cached;
	}

	try {
		const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
		const dataUrl = thumbHashToDataURL(bytes);

		if (cache.size >= MAX_CACHE_SIZE) {
			// Evict oldest entry (first key in Map insertion order)
			const oldest = cache.keys().next().value!;
			cache.delete(oldest);
		}

		cache.set(base64, dataUrl);
		return dataUrl;
	} catch {
		return null;
	}
}
