import { thumbHashToDataURL } from 'thumbhash';

export function decodeThumbhash(base64: string | null | undefined): string | null {
	if (!base64) return null;
	try {
		const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
		return thumbHashToDataURL(bytes);
	} catch {
		return null;
	}
}
