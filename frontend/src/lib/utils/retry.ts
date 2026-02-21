import type { ApiError } from '$lib/api/errors';
import type { Result } from 'true-myth';

interface RetryOptions {
	maxAttempts?: number;
	baseDelayMs?: number;
	maxDelayMs?: number;
}

/**
 * Retry wrapper for client-side interactive API calls.
 * Only retries on Network and 5xx ServerError — never on 4xx.
 * Exponential backoff with jitter.
 */
export async function withRetry<T>(
	fn: () => Promise<Result<T, ApiError>>,
	options?: RetryOptions
): Promise<Result<T, ApiError>> {
	const maxAttempts = options?.maxAttempts ?? 3;
	const baseDelayMs = options?.baseDelayMs ?? 1000;
	const maxDelayMs = options?.maxDelayMs ?? 10000;

	let lastResult: Result<T, ApiError> | undefined;

	for (let attempt = 0; attempt < maxAttempts; attempt++) {
		lastResult = await fn();

		if (lastResult.isOk) return lastResult;

		if (!lastResult.error.isRetryable || attempt === maxAttempts - 1) {
			return lastResult;
		}

		const delay = Math.min(baseDelayMs * 2 ** attempt, maxDelayMs) * (0.5 + Math.random() * 0.5);
		await new Promise((r) => setTimeout(r, delay));
	}

	return lastResult!;
}
