import { getApiUrl } from './config';
import { AdminEndpoints } from './endpoints/admin';
import { AdoptEndpoints } from './endpoints/adopt';
import { CaptureEndpoints } from './endpoints/captures';
import { DeviceEndpoints } from './endpoints/device';
import { RunEndpoints } from './endpoints/runs';
import { SceneEndpoints } from './endpoints/scenes';
import { ShaderEndpoints } from './endpoints/shaders';
import { WorldsEndpoints } from './endpoints/worlds';

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
		shaders: new ShaderEndpoints(url, fetchFn),
		scenes: new SceneEndpoints(url, fetchFn),
		captures: new CaptureEndpoints(url, fetchFn),
		admin: new AdminEndpoints(url, fetchFn),
		adopt: new AdoptEndpoints(url, fetchFn),
		worlds: new WorldsEndpoints(url, fetchFn),
		device: new DeviceEndpoints(url, fetchFn),
		runs: new RunEndpoints(url, fetchFn)
	};
}

/**
 * Default API client instance for client-side usage
 */
export const api = createApiClient();

// Re-export types for convenience
export type * from '$lib/bindings';
export { ApiError, ApiErrorType } from './errors';
