import type { RequestHandler } from './$types';
import { fetchOgData, type OgType } from '$lib/og/data';
import { renderOgImage, renderTextOnlyOgImage } from '$lib/og/render';
import { ogCache, ogSingleflight } from '$lib/og/cache';
import { getLogger } from '@logtape/logtape';

const logger = getLogger(['ssr', 'routes', 'og']);

const VALID_TYPES: ReadonlySet<string> = new Set<OgType>([
	'shader',
	'scene',
	'home',
	'shaders',
	'scenes',
	'compare'
]);

const CACHE_CONTROL = 'public, max-age=86400, stale-while-revalidate=3600';
const CACHE_CONTROL_SHORT = 'public, max-age=300, stale-while-revalidate=60';

function jpegResponse(data: Uint8Array, fallback = false): Response {
	return new Response(new Uint8Array(data) as BlobPart, {
		headers: {
			'Content-Type': 'image/jpeg',
			'Cache-Control': fallback ? CACHE_CONTROL_SHORT : CACHE_CONTROL
		}
	});
}

export const GET: RequestHandler = async ({ params: routeParams, fetch }) => {
	const { type } = routeParams;
	if (!VALID_TYPES.has(type)) {
		return new Response('Invalid OG image type', { status: 400 });
	}

	const ogType = type as OgType;
	const slug = routeParams.params ?? '';
	const cacheKey = `${ogType}:${slug}`;

	// Check LRU cache
	const cached = ogCache.get(cacheKey);
	if (cached) return jpegResponse(cached);

	// Singleflight: deduplicate concurrent requests for the same image
	try {
		const { buffer, fallback } = await ogSingleflight.do(cacheKey, async () => {
			const data = await fetchOgData(ogType, slug, fetch);

			let imageBuffer: Uint8Array;

			if (data.imageUrl) {
				// Fetch the screenshot from CDN
				const imageResponse = await fetch(data.imageUrl);
				if (!imageResponse.ok) {
					return {
						buffer: await renderTextOnlyOgImage(data.text),
						fallback: true
					};
				}

				const rawImage = Buffer.from(await imageResponse.arrayBuffer());
				imageBuffer = await renderOgImage({ imageBuffer: rawImage, text: data.text });
			} else {
				imageBuffer = await renderTextOnlyOgImage(data.text);
			}

			// Don't cache fallback images (missing resources) — they should be re-checked
			if (!data.fallback) {
				ogCache.set(cacheKey, imageBuffer);
			}

			return { buffer: imageBuffer, fallback: !!data.fallback };
		});

		return jpegResponse(buffer, fallback);
	} catch (error) {
		logger.error('OG image generation failed: {cacheKey}', {
			cacheKey,
			error: error instanceof Error ? error.message : String(error),
			stack: error instanceof Error ? error.stack : undefined
		});
		return new Response('Failed to generate OG image', { status: 500 });
	}
};
