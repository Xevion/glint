import { getApiUrl } from './config';
import { AdminEndpoints } from './endpoints/admin';
import { AdoptEndpoints } from './endpoints/adopt';
import { BackgroundEndpoints } from './endpoints/backgrounds';
import { DeviceEndpoints } from './endpoints/device';
import { RunEndpoints } from './endpoints/runs';
import { UserEndpoints } from './endpoints/user';

/**
 * Create API client with optional custom fetch function
 *
 * For server-side usage (load functions), pass event.fetch:
 * ```ts
 * const api = createApiClient(event.fetch);
 * ```
 *
 * For client-side usage, use the default export:
 * ```ts
 * import { api } from '$lib/api';
 * ```
 */
export function createApiClient(fetchFn?: typeof fetch, baseUrl?: string) {
	const url = baseUrl ?? getApiUrl();
	return {
		backgrounds: new BackgroundEndpoints(url, fetchFn),
		admin: new AdminEndpoints(url, fetchFn),
		adopt: new AdoptEndpoints(url, fetchFn),
		device: new DeviceEndpoints(url, fetchFn),
		runs: new RunEndpoints(url, fetchFn),
		user: new UserEndpoints(url, fetchFn)
	};
}

/**
 * Default API client instance for client-side usage
 */
export const api = createApiClient();

// Re-export types for convenience
export type * from '$lib/bindings';
export { ApiErrorType } from './errors';
