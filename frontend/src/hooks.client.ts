import type { HandleClientError } from '@sveltejs/kit';
import posthog from 'posthog-js';

export const handleError: HandleClientError = ({ error, status }) => {
	if (status !== 404) {
		posthog.captureException(error);
	}

	return {
		message: status === 404 ? 'Not Found' : 'An error occurred'
	};
};
