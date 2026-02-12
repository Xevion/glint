import type { RequestHandler } from './$types';

/**
 * Proxy /sitemap.xml to the backend's /api/sitemap.xml endpoint.
 *
 * Uses SvelteKit's event.fetch with a relative URL so the request goes
 * through the same routing pipeline as all other API calls (Vite proxy
 * in dev, SvelteKit API proxy in production).
 *
 * The backend generates relative paths (<loc>/shaders/foo</loc>).
 * This proxy prepends the request origin so crawlers get absolute URLs
 * as required by the sitemap protocol.
 */
export const GET: RequestHandler = async ({ url, fetch }) => {
	const response = await fetch('/api/sitemap.xml');

	if (!response.ok) {
		return new Response('Failed to generate sitemap', { status: 502 });
	}

	let xml = await response.text();

	// Prepend origin to all <loc> paths to produce absolute URLs
	const origin = url.origin;
	xml = xml.replaceAll('<loc>/', `<loc>${origin}/`);

	return new Response(xml, {
		headers: {
			'content-type': 'application/xml; charset=utf-8',
			'cache-control': 'public, max-age=900'
		}
	});
};
