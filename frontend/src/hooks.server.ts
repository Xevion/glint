import { dev } from '$app/environment';
import { env } from '$env/dynamic/private';
import { initLogger } from '$lib/logger';
import { requestContext } from '$lib/server/context';
import { getLogger } from '@logtape/logtape';
import type { Handle, HandleServerError } from '@sveltejs/kit';
import { PostHog } from 'posthog-node';

await initLogger();

const backendUrl = env.BACKEND_URL ?? 'http://localhost:8080';

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

			// Follow backend redirects (e.g. slug canonicalization) internally
			// so SSR load functions receive the final response, not the 308.
			if (response.status >= 300 && response.status < 400) {
				const location = response.headers.get('location');
				if (location) {
					const redirectUrl = location.startsWith('/') ? `${backendUrl}${location}` : location;
					response = await fetch(redirectUrl, {
						method: 'GET',
						headers,
						redirect: 'manual'
					});
				}
			}
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

		// Security headers applied to all SvelteKit-served responses (HTML, static assets).
		// CSP is handled separately via kit.csp in svelte.config.js.
		response.headers.set('X-Frame-Options', 'DENY');
		response.headers.set('X-Content-Type-Options', 'nosniff');
		response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
		response.headers.set('Permissions-Policy', 'camera=(), microphone=(), geolocation=()');
		if (!dev) {
			response.headers.set('Strict-Transport-Security', 'max-age=31536000; includeSubDomains');
		}

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

export const handleError: HandleServerError = ({ error, event, status }) => {
	if (status !== 404) {
		errorLogger.error('{method} {path} {status} (unhandled): {error}', {
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
