import sharp from 'sharp';
import { OG_HEIGHT, OG_WIDTH } from './constants';
import { type TextOverlayProps, renderTextOverlay } from './text';

/**
 * SVG gradient overlay: transparent at top, dark at bottom.
 * Covers the bottom ~50% with a smooth falloff.
 */
const GRADIENT_SVG = `<svg width="${OG_WIDTH}" height="${OG_HEIGHT}" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="g" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="black" stop-opacity="0"/>
      <stop offset="45%" stop-color="black" stop-opacity="0"/>
      <stop offset="75%" stop-color="black" stop-opacity="0.6"/>
      <stop offset="100%" stop-color="black" stop-opacity="0.85"/>
    </linearGradient>
  </defs>
  <rect width="${OG_WIDTH}" height="${OG_HEIGHT}" fill="url(#g)"/>
</svg>`;

const gradientBuffer = Buffer.from(GRADIENT_SVG);

export interface OgRenderOptions {
	/** Raw image bytes for the background screenshot */
	imageBuffer: Buffer;
	/** Text overlay props */
	text: TextOverlayProps;
}

/**
 * Render a complete OG image:
 * 1. Resize/crop screenshot to 1200x630
 * 2. Composite gradient overlay
 * 3. Composite text overlay (rendered via satori)
 * 4. Encode as JPEG
 */
export async function renderOgImage(options: OgRenderOptions): Promise<Uint8Array> {
	const { imageBuffer, text } = options;

	// Render the text overlay as a transparent PNG (async, can run while sharp processes)
	const textOverlayPromise = renderTextOverlay(text);

	// Resize and crop the screenshot to OG dimensions
	const base = sharp(imageBuffer).resize(OG_WIDTH, OG_HEIGHT, {
		fit: 'cover',
		position: 'centre'
	});

	const textOverlay = await textOverlayPromise;

	// Composite gradient + text overlay
	const result = await base
		.composite([
			{ input: gradientBuffer, top: 0, left: 0 },
			{ input: textOverlay, top: 0, left: 0 }
		])
		.jpeg({ quality: 85 })
		.toBuffer();

	return new Uint8Array(result);
}

/**
 * Render a text-only OG image (no screenshot background).
 * Used when no capture image is available.
 */
export async function renderTextOnlyOgImage(text: TextOverlayProps): Promise<Uint8Array> {
	const textOverlay = await renderTextOverlay(text);

	// Dark background with text overlay
	const result = await sharp({
		create: {
			width: OG_WIDTH,
			height: OG_HEIGHT,
			channels: 4,
			background: { r: 15, g: 15, b: 15, alpha: 1 }
		}
	})
		.composite([{ input: textOverlay, top: 0, left: 0 }])
		.jpeg({ quality: 85 })
		.toBuffer();

	return new Uint8Array(result);
}
