import { thumbHashToDataURL } from 'thumbhash';

const cache = new Map<string, string>();

export function decodeThumbhash(base64: string | null | undefined): string | null {
	if (!base64) return null;

	const cached = cache.get(base64);
	if (cached) return cached;

	try {
		const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
		const dataUrl = thumbHashToDataURL(bytes);
		cache.set(base64, dataUrl);
		return dataUrl;
	} catch {
		return null;
	}
}
