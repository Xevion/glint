import type { RequestHandler } from './$types';
import { env } from '$env/dynamic/public';

const BACKEND_URL = env.PUBLIC_BACKEND_URL ?? 'http://localhost:3001';

/**
 * Catch-all proxy for /api/* requests to the Axum backend.
 *
 * In development, Vite's proxy handles this. In production, SvelteKit needs
 * to forward client-side API calls since only the Axum backend is not publicly
 * exposed — only the SvelteKit server is.
 */
const handler: RequestHandler = async ({ request, params, url }) => {
	const target = `${BACKEND_URL}/api/${params.path}${url.search}`;

	const headers = new Headers(request.headers);
	// Remove host header so the backend doesn't reject it
	headers.delete('host');

	return fetch(target, {
		method: request.method,
		headers,
		body: request.body,
		// @ts-expect-error -- Bun supports duplex streaming
		duplex: 'half'
	});
};

export const GET = handler;
export const POST = handler;
export const PUT = handler;
export const PATCH = handler;
export const DELETE = handler;
export const HEAD = handler;
export const OPTIONS = handler;
