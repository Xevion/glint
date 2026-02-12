import type { Result } from 'true-myth';
import type { ApiError } from './errors';

/**
 * Svelte 5 composable for client-side retry with reactive state.
 *
 * The fetcher already benefits from ApiClient's automatic retry (transparent
 * backoff on 502/503/504). This composable handles *manual* retry — the user
 * clicks a button after all automatic retries have been exhausted.
 */
export function useRetry<T>(
	fetcher: () => Promise<Result<T, ApiError>>,
	options?: { initial?: T }
) {
	let data = $state<T | undefined>(options?.initial);
	let error = $state<ApiError | null>(null);
	let loading = $state(false);
	let attempts = $state(0);

	async function execute() {
		loading = true;
		error = null;
		attempts++;
		const result = await fetcher();
		result.match({
			Ok: (value) => {
				data = value;
				error = null;
			},
			Err: (err) => {
				error = err;
			}
		});
		loading = false;
	}

	return {
		get data() {
			return data;
		},
		get error() {
			return error;
		},
		get loading() {
			return loading;
		},
		get attempts() {
			return attempts;
		},
		get isRetryable() {
			return error?.isRetryable ?? false;
		},
		execute,
		retry: execute
	};
}
