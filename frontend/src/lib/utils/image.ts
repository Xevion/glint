export interface ImageTransformOptions {
	width?: number;
	height?: number;
	quality?: number;
	fit?: 'scale-down' | 'contain' | 'cover' | 'crop';
	format?: 'auto' | 'avif' | 'webp' | 'jpeg';
}

export const IMAGE_PRESETS = {
	thumbnail: { width: 160, quality: 75, fit: 'cover', format: 'auto' },
	card: { width: 640, quality: 80, fit: 'cover', format: 'auto' },
	hero: { width: 1280, quality: 85, fit: 'cover', format: 'auto' },
	full: { format: 'auto', quality: 85 }
} as const satisfies Record<string, ImageTransformOptions>;

export type ImagePreset = keyof typeof IMAGE_PRESETS;

/**
 * Build a Cloudflare Image Transformation URL from a raw image URL.
 * Returns null if src is nullish so callers can use existing fallback chains.
 */
export function cfImageUrl(
	src: string | null | undefined,
	options: ImageTransformOptions | ImagePreset
): string | null {
	if (!src) return null;

	const opts: ImageTransformOptions = typeof options === 'string' ? IMAGE_PRESETS[options] : options;

	const parts: string[] = [];
	if (opts.width) parts.push(`width=${opts.width}`);
	if (opts.height) parts.push(`height=${opts.height}`);
	if (opts.quality) parts.push(`quality=${opts.quality}`);
	if (opts.fit) parts.push(`fit=${opts.fit}`);
	parts.push(`format=${opts.format ?? 'auto'}`);

	const url = new URL(src);
	return `${url.origin}/cdn-cgi/image/${parts.join(',')}${url.pathname}`;
}
