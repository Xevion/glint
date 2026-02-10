import type { Handle, HandleFetch, HandleServerError } from '@sveltejs/kit';
import { PostHog } from 'posthog-node';
import { dev } from '$app/environment';
import { env } from '$env/dynamic/private';
import { env as publicEnv } from '$env/dynamic/public';

const backendUrl = publicEnv.PUBLIC_BACKEND_URL ?? 'http://localhost:8080';

export const handle: Handle = ({ event, resolve }) => {
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
