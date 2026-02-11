import type { RequestHandler } from './$types';
import { fetchOgData, type OgType } from '$lib/og/data';
import { renderOgImage, renderTextOnlyOgImage } from '$lib/og/render';
import { ogCache, ogSingleflight } from '$lib/og/cache';
import { getLogger } from '@logtape/logtape';

const logger = getLogger(['ssr', 'routes', 'og']);

const VALID_TYPES: ReadonlySet<string> = new Set<OgType>(['shader', 'scene', 'home', 'shaders']);

const CACHE_CONTROL = 'public, max-age=86400, stale-while-revalidate=3600';

function jpegResponse(data: Uint8Array): Response {
	// Copy into a fresh ArrayBuffer-backed Uint8Array to satisfy TS strict typing
	// (Uint8Array<ArrayBufferLike> includes SharedArrayBuffer which isn't a valid BlobPart)
	const body = new Uint8Array(data) as BlobPart;
	return new Response(new Blob([body], { type: 'image/jpeg' }), {
		headers: {
			'Content-Type': 'image/jpeg',
			'Cache-Control': CACHE_CONTROL
		}
	});
}

export const GET: RequestHandler = async ({ params: routeParams, fetch }) => {
	const { type } = routeParams;
	if (!VALID_TYPES.has(type)) {
		return new Response(`Invalid OG type: ${type}`, { status: 400 });
	}

	const ogType = type as OgType;
	const slug = routeParams.params ?? '';
	const cacheKey = `${ogType}:${slug}`;

	// Check LRU cache
	const cached = ogCache.get(cacheKey);
	if (cached) return jpegResponse(cached);

	// Singleflight: deduplicate concurrent requests for the same image
	try {
		const buffer = await ogSingleflight.do(cacheKey, async () => {
			const data = await fetchOgData(ogType, slug, fetch);

			let imageBuffer: Uint8Array;

			if (data.imageUrl) {
				// Fetch the screenshot from CDN
				const imageResponse = await fetch(data.imageUrl);
				if (!imageResponse.ok) {
					return renderTextOnlyOgImage(data.text);
				}

				const rawImage = Buffer.from(await imageResponse.arrayBuffer());
				imageBuffer = await renderOgImage({ imageBuffer: rawImage, text: data.text });
			} else {
				imageBuffer = await renderTextOnlyOgImage(data.text);
			}

			ogCache.set(cacheKey, imageBuffer);
			return imageBuffer;
		});

		return jpegResponse(buffer);
	} catch (error) {
		logger.error('OG image generation failed: {cacheKey}', {
			cacheKey,
			error: error instanceof Error ? error.message : String(error),
			stack: error instanceof Error ? error.stack : undefined
		});
		return new Response('Failed to generate OG image', { status: 500 });
	}
};
