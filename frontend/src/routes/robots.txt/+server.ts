import type { RequestHandler } from './$types';

/**
 * Dynamic robots.txt that includes the absolute sitemap URL
 * derived from the request origin.
 */
export const GET: RequestHandler = ({ url }) => {
	const origin = url.origin;

	const body = [
		'User-agent: *',
		'Allow: /',
		'',
		'Disallow: /admin/',
		'Disallow: /api/',
		'',
		`Sitemap: ${origin}/sitemap.xml`
	].join('\n');

	return new Response(body, {
		headers: {
			'content-type': 'text/plain; charset=utf-8',
			'cache-control': 'public, max-age=86400'
		}
	});
};
