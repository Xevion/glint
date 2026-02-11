import type { Handle, HandleFetch, HandleServerError } from '@sveltejs/kit';
import { PostHog } from 'posthog-node';
import { dev } from '$app/environment';
import { env } from '$env/dynamic/private';
import { env as publicEnv } from '$env/dynamic/public';

const backendUrl = publicEnv.PUBLIC_BACKEND_URL ?? 'http://localhost:8080';

export const handle: Handle = async ({ event, resolve }) => {
	// Proxy /api/* requests from the browser to the internal Axum backend.
	// In dev, Vite's proxy handles this. In production, SvelteKit is the
	// public-facing server and must forward API requests itself.
	if (event.url.pathname.startsWith('/api/')) {
		const targetUrl = `${backendUrl}${event.url.pathname}${event.url.search}`;
		const headers = new Headers(event.request.headers);
		// Remove host header so the backend sees its own host
		headers.delete('host');

		const response = await fetch(targetUrl, {
			method: event.request.method,
			headers,
			body: event.request.body,
			// Don't follow redirects — the browser must follow them (OAuth flow)
			redirect: 'manual',
			// @ts-expect-error Bun supports duplex streaming
			duplex: 'half'
		});

		return new Response(response.body, {
			status: response.status,
			statusText: response.statusText,
			headers: response.headers
		});
	}

	return resolve(event, {
		transformPageChunk: ({ html }) => html.replace('%paraglide.lang%', 'en'),
		filterSerializedResponseHeaders: (name) => name === 'content-length' || name === 'content-type'
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

const posthogKey = env.POSTHOG_KEY;
const posthogHost = env.POSTHOG_HOST;

const posthog = posthogKey && posthogHost ? new PostHog(posthogKey, { host: posthogHost }) : null;

export const handleError: HandleServerError = ({ error, event, status }) => {
	if (status !== 404) {
		console.error(`[SSR ${status}] ${event.request.method} ${event.url.pathname}`, error);
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
		// Expose stack trace in dev for the error page to display
		...(dev && error instanceof Error && error.stack ? { stack: error.stack } : {})
	};
};
