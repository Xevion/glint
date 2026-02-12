import { initClientLogger } from '$lib/logger.client';
import { getLogger } from '@logtape/logtape';
import type { HandleClientError } from '@sveltejs/kit';
import posthog from 'posthog-js';

initClientLogger();

const logger = getLogger(['client', 'error']);

export const handleError: HandleClientError = ({ error, status }) => {
	const errorId = crypto.randomUUID();
	const timestamp = new Date().toISOString();

	const errorMessage = error instanceof Error ? error.message : String(error);
	const stack = error instanceof Error ? error.stack : undefined;

	logger.error('{errorId} [{status}] {message}', {
		errorId,
		status,
		message: errorMessage,
		stack,
		url: window.location.href
	});

	if (status !== 404) {
		posthog.captureException(error, {
			$set: { errorId }
		});
	}

	return {
		message: status === 404 ? 'Not Found' : errorMessage,
		errorId,
		source: 'client' as const,
		timestamp,
		stack
	};
};
