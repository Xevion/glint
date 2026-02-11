import type { Handle, HandleFetch, HandleServerError } from '@sveltejs/kit';
import { dev } from '$app/environment';
import { env } from '$env/dynamic/private';
import { env as publicEnv } from '$env/dynamic/public';
import { initLogger } from '$lib/logger';
import { requestContext } from '$lib/server/context';
import { getLogger } from '@logtape/logtape';
import { PostHog } from 'posthog-node';

await initLogger();

const backendUrl = publicEnv.PUBLIC_BACKEND_URL ?? 'http://localhost:8080';

const posthog =
	env.POSTHOG_KEY && env.POSTHOG_HOST
		? new PostHog(env.POSTHOG_KEY, { host: env.POSTHOG_HOST })
		: null;

const proxyLogger = getLogger(['ssr', 'proxy']);
const errorLogger = getLogger(['ssr', 'error']);

export const handle: Handle = async ({ event, resolve }) => {
	const { method } = event.request;
	const { pathname } = event.url;

	// Proxy /api/* requests from the browser to the internal Axum backend.
	// In dev, Vite's proxy handles this. In production, SvelteKit is the
	// public-facing server and must forward API requests itself.
	if (pathname.startsWith('/api/')) {
		const targetUrl = `${backendUrl}${pathname}${event.url.search}`;
		const headers = new Headers(event.request.headers);
		headers.delete('host');

		let response: Response;
		try {
			response = await fetch(targetUrl, {
				method,
				headers,
				body: event.request.body,
				redirect: 'manual',
				// @ts-expect-error Bun supports duplex streaming
				duplex: 'half'
			});
		} catch (err) {
			proxyLogger.error('{method} {path} → backend unreachable', {
				method,
				path: pathname,
				error: err instanceof Error ? err.message : String(err)
			});
			return new Response(JSON.stringify({ error: 'Backend unavailable' }), {
				status: 502,
				headers: { 'content-type': 'application/json' }
			});
		}

		return new Response(response.body, {
			status: response.status,
			statusText: response.statusText,
			headers: response.headers
		});
	}

	// Extract or generate a request ID for tracing
	const requestId = event.request.headers.get('x-request-id') ?? crypto.randomUUID();

	return requestContext.run({ requestId }, async () => {
		const response = await resolve(event, {
			transformPageChunk: ({ html }) => html.replace('%paraglide.lang%', 'en'),
			filterSerializedResponseHeaders: (name) => name === 'content-length' || name === 'content-type'
		});

		if (response.status >= 400) {
			const reqLogger = getLogger(['ssr', 'request']);
			reqLogger.warn('{method} {path} {status}', {
				method,
				path: pathname,
				status: response.status
			});
		}

		return response;
	});
};

/**
 * Forward cookies to the backend during SSR.
 * SvelteKit's fetch only auto-forwards cookies for same-origin requests;
 * the backend runs on a different port so cookies are dropped without this.
 */
export const handleFetch: HandleFetch = async ({ event, request, fetch }) => {
	if (request.url.startsWith(backendUrl)) {
		const cookie = event.request.headers.get('cookie');
		if (cookie) {
			request.headers.set('cookie', cookie);
		}
	}
	return fetch(request);
};

export const handleError: HandleServerError = ({ error, event, status }) => {
	if (status !== 404) {
		errorLogger.error('{method} {path} {status} (unhandled)', {
			status,
			method: event.request.method,
			path: event.url.pathname,
			error: error instanceof Error ? error.message : String(error)
		});
	}

	if (posthog && status !== 404) {
		const requestId = event.request.headers.get('x-request-id') ?? 'unknown';
		posthog.captureException(error, requestId, {
			method: event.request.method,
			path: event.url.pathname,
			status
		});
	}

	const message =
		status === 404
			? 'Not Found'
			: dev && error instanceof Error
				? error.message
				: 'An error occurred';

	return {
		message,
		...(dev && error instanceof Error && error.stack ? { stack: error.stack } : {})
	};
};
