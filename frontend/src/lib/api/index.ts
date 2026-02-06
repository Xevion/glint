import { API_BASE_URL } from './config';
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
	return {
		shaders: new ShaderEndpoints(baseUrl ?? API_BASE_URL, fetchFn),
		scenes: new SceneEndpoints(baseUrl ?? API_BASE_URL, fetchFn),
		captures: new CaptureEndpoints(baseUrl ?? API_BASE_URL, fetchFn),
		admin: new AdminEndpoints(baseUrl ?? API_BASE_URL, fetchFn),
		adopt: new AdoptEndpoints(baseUrl ?? API_BASE_URL, fetchFn),
		worlds: new WorldsEndpoints(baseUrl ?? API_BASE_URL, fetchFn),
		device: new DeviceEndpoints(baseUrl ?? API_BASE_URL, fetchFn),
		runs: new RunEndpoints(baseUrl ?? API_BASE_URL, fetchFn)
	};
}

/**
 * Default API client instance for client-side usage
 */
export const api = createApiClient();

// Re-export types for convenience
export type * from '$lib/bindings';
export { ApiError, ApiErrorType } from './errors';
