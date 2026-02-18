import { env } from '$env/dynamic/public';

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

/** S3 bucket name — matches docker-compose minio-init and R2 config. */
const S3_BUCKET = 'glint';

/**
 * Build a transformed image URL from an image path (S3 key).
 *
 * - When `PUBLIC_IMGPROXY_URL` is set, builds an imgproxy URL that fetches
 *   from S3 (local MinIO).
 * - Otherwise, builds a Cloudflare cdn-cgi/image URL using `PUBLIC_CDN_URL`.
 *
 * Returns null when `path` is nullish so callers can use existing fallback chains.
 */
export function imageUrl(
	path: string | null | undefined,
	options: ImageTransformOptions | ImagePreset
): string | null {
	if (!path) return null;

	const opts: ImageTransformOptions =
		typeof options === 'string' ? presetToTransformOptions(options) : options;

	if (env.PUBLIC_IMGPROXY_URL) {
		return buildImgproxyUrl(path, opts);
	}
	return buildCloudflareUrl(path, opts);
}

/**
 * Build a srcset string with transformed URLs for each width tier.
 */
export function imageSrcset(
	path: string | null | undefined,
	options: ImagePresetConfig | ImagePreset
): string | null {
	if (!path) return null;

	const opts: ImagePresetConfig = typeof options === 'string' ? IMAGE_PRESETS[options] : options;

	return IMAGE_WIDTHS.map((w) => {
		const transformed = imageUrl(path, {
			width: w,
			quality: opts.quality,
			fit: opts.fit,
			format: opts.format
		});
		return `${transformed} ${w}w`;
	}).join(', ');
}

/**
 * Construct a raw (non-transformed) URL for an image path.
 * Uses PUBLIC_CDN_URL as the base.
 */
export function rawImageUrl(path: string | null | undefined): string | null {
	if (!path) return null;
	const cdnUrl = env.PUBLIC_CDN_URL;
	if (!cdnUrl) return null;
	return `${cdnUrl}/${path}`;
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

function buildCloudflareUrl(path: string, opts: ImageTransformOptions): string | null {
	const cdnUrl = env.PUBLIC_CDN_URL;
	if (!cdnUrl) return null;

	const parts: string[] = [];
	if (opts.width) parts.push(`width=${opts.width}`);
	if (opts.height) parts.push(`height=${opts.height}`);
	if (opts.quality) parts.push(`quality=${opts.quality}`);
	if (opts.fit) parts.push(`fit=${opts.fit}`);
	parts.push(`format=${opts.format ?? 'auto'}`);

	return `${cdnUrl}/cdn-cgi/image/${parts.join(',')}/${path}`;
}

function buildImgproxyUrl(path: string, opts: ImageTransformOptions): string {
	const parts: string[] = [];

	// Resize type + dimensions
	const rt = mapFit(opts.fit);
	if (opts.width || opts.height) {
		parts.push(`rs:${rt}:${opts.width ?? 0}:${opts.height ?? 0}`);
	}

	if (opts.quality) parts.push(`q:${opts.quality}`);

	// format=auto → omit (imgproxy auto-negotiates via Accept header)
	// explicit format → f:webp, f:jpg, etc.
	if (opts.format && opts.format !== 'auto') {
		parts.push(`f:${opts.format === 'jpeg' ? 'jpg' : opts.format}`);
	}

	const processing = parts.length > 0 ? parts.join('/') : '';
	return `${env.PUBLIC_IMGPROXY_URL}/insecure/${processing}/plain/s3://${S3_BUCKET}/${path}`;
}

function mapFit(fit?: string): string {
	switch (fit) {
		case 'cover':
		case 'crop':
			return 'fill';
		case 'contain':
		case 'scale-down':
		default:
			return 'fit';
	}
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
	image_path?: string | null;
	thumbhash?: string | null;
}

export function resolveImageSource(source: string | ImageSourceLike): {
	src: string | null;
	thumbhash: string | null;
} {
	if (typeof source === 'string') return { src: source, thumbhash: null };
	return {
		src: source.image_path ?? null,
		thumbhash: source.thumbhash ?? null
	};
}

/**
 * Preload an image in the background using responsive srcset/sizes so the
 * browser fetches the same variant it would pick for the actual `<img>`.
 * Returns the Image element for optional cleanup.
 */
export function preloadImage(
	path: string | null | undefined,
	preset: ImagePreset = 'hero'
): HTMLImageElement | null {
	if (!path) return null;
	const srcset = imageSrcset(path, preset);
	if (!srcset) return null;
	const img = new Image();
	img.srcset = srcset;
	img.sizes = IMAGE_PRESETS[preset].sizes;
	const url = imageUrl(path, preset);
	if (url) img.src = url;
	return img;
}
