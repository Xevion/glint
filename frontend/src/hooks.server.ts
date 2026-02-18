import { dev } from '$app/environment';
import { env } from '$env/dynamic/private';
import { initLogger } from '$lib/logger.server';
import { requestContext } from '$lib/server/context';
import { createThumbhashExpression } from '$lib/thumbhash-inline';
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

// Pre-compute once at startup — the expression is static
const thumbhashSnippet = createThumbhashExpression();

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

			// Follow backend-to-backend redirects (e.g. slug canonicalization)
			// so SSR load functions receive the final response, not the 308.
			// Only follow redirects that stay within /api/ — everything else
			// (frontend routes like /login, external URLs like Discord OAuth)
			// must pass through to the browser so Set-Cookie headers and
			// redirect destinations are preserved.
			if (response.status >= 300 && response.status < 400) {
				const location = response.headers.get('location');
				if (location?.startsWith('/api/')) {
					response = await fetch(`${backendUrl}${location}`, {
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
			transformPageChunk: ({ html }) =>
				html.replace('%paraglide.lang%', 'en').replace('%thumbhash.snippet%', thumbhashSnippet),
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
	const errorId = crypto.randomUUID();
	const timestamp = new Date().toISOString();
	const requestId = event.request.headers.get('x-request-id') ?? undefined;
	const errorMessage = error instanceof Error ? error.message : String(error);
	const stack = error instanceof Error ? error.stack : undefined;

	if (status !== 404) {
		errorLogger.error('{errorId} {method} {path} {status} (unhandled): {error}', {
			errorId,
			status,
			method: event.request.method,
			path: event.url.pathname,
			error: errorMessage,
			requestId
		});
	}

	if (posthog && status !== 404) {
		posthog.captureException(error, requestId ?? errorId, {
			method: event.request.method,
			path: event.url.pathname,
			status,
			errorId
		});
	}

	return {
		message: status === 404 ? 'Not Found' : errorMessage,
		errorId,
		source: 'server' as const,
		timestamp,
		stack,
		requestId
	};
};
