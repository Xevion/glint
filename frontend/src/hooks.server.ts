import type { Handle, HandleServerError } from '@sveltejs/kit';
import { PostHog } from 'posthog-node';
import { env } from '$env/dynamic/private';

export const handle: Handle = ({ event, resolve }) => {
	return resolve(event, {
		transformPageChunk: ({ html }) => html.replace('%paraglide.lang%', 'en')
	});
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

	return {
		message: status === 404 ? 'Not Found' : 'An error occurred'
	};
};
