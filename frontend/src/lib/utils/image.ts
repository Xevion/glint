export interface ImageTransformOptions {
	width?: number;
	height?: number;
	quality?: number;
	fit?: 'scale-down' | 'contain' | 'cover' | 'crop';
	format?: 'auto' | 'avif' | 'webp' | 'jpeg';
}

export const IMAGE_WIDTHS = [160, 320, 640, 960, 1280, 1920] as const;

export const IMAGE_PRESETS = {
	thumbnail: {
		sizes: '160px',
		quality: 75,
		fit: 'cover',
		format: 'auto'
	},
	card: {
		sizes: '(min-width: 1280px) 25vw, (min-width: 768px) 50vw, 100vw',
		quality: 80,
		fit: 'cover',
		format: 'auto'
	},
	hero: {
		sizes: '(min-width: 1024px) 66vw, 100vw',
		quality: 85,
		fit: 'cover',
		format: 'auto'
	},
	full: { sizes: '100vw', quality: 90, format: 'auto' }
} as const satisfies Record<string, ImagePresetConfig>;

export interface ImagePresetConfig {
	sizes: string;
	quality?: number;
	fit?: ImageTransformOptions['fit'];
	format?: ImageTransformOptions['format'];
}

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

	const opts: ImageTransformOptions =
		typeof options === 'string' ? presetToTransformOptions(options) : options;

	const parts: string[] = [];
	if (opts.width) parts.push(`width=${opts.width}`);
	if (opts.height) parts.push(`height=${opts.height}`);
	if (opts.quality) parts.push(`quality=${opts.quality}`);
	if (opts.fit) parts.push(`fit=${opts.fit}`);
	parts.push(`format=${opts.format ?? 'auto'}`);

	const url = new URL(src);
	return `${url.origin}/cdn-cgi/image/${parts.join(',')}${url.pathname}`;
}

/**
 * Build a srcset string with Cloudflare-transformed URLs for each width tier.
 */
export function cfImageSrcset(
	src: string | null | undefined,
	options: ImagePresetConfig | ImagePreset
): string | null {
	if (!src) return null;

	const opts: ImagePresetConfig = typeof options === 'string' ? IMAGE_PRESETS[options] : options;

	return IMAGE_WIDTHS.map((w) => {
		const transformed = cfImageUrl(src, {
			width: w,
			quality: opts.quality,
			fit: opts.fit,
			format: opts.format
		});
		return `${transformed} ${w}w`;
	}).join(', ');
}

function presetToTransformOptions(preset: ImagePreset): ImageTransformOptions {
	const config: ImagePresetConfig = IMAGE_PRESETS[preset];
	return {
		width:
			preset === 'thumbnail'
				? 160
				: preset === 'card'
					? 640
					: preset === 'hero'
						? 1280
						: preset === 'full'
							? 1920
							: undefined,
		quality: config.quality,
		fit: config.fit,
		format: config.format
	};
}

interface ImageSourceLike {
	image_url?: string | null;
	thumbhash?: string | null;
}

export function resolveImageSource(source: string | ImageSourceLike): {
	src: string | null;
	thumbhash: string | null;
} {
	if (typeof source === 'string') return { src: source, thumbhash: null };
	return {
		src: source.image_url ?? null,
		thumbhash: source.thumbhash ?? null
	};
}

/**
 * Preload an image in the background using responsive srcset/sizes so the
 * browser fetches the same variant it would pick for the actual `<img>`.
 * Returns the Image element for optional cleanup.
 */
export function preloadImage(
	src: string | null | undefined,
	preset: ImagePreset = 'hero'
): HTMLImageElement | null {
	if (!src) return null;
	const srcset = cfImageSrcset(src, preset);
	if (!srcset) return null;
	const img = new Image();
	img.srcset = srcset;
	img.sizes = IMAGE_PRESETS[preset].sizes;
	// Fallback src for browsers that ignore srcset on Image()
	const url = cfImageUrl(src, preset);
	if (url) img.src = url;
	return img;
}
