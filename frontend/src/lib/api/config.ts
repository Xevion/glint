import { browser } from '$app/environment';
import { env } from '$env/dynamic/public';

/**
 * Get the API base URL for the current context.
 *
 * - Browser: relative URLs (same-origin, handled by Vite proxy in dev or reverse proxy in prod)
 * - SSR: PUBLIC_BACKEND_URL env var for server-to-server calls (may be different host/port)
 */
export function getApiUrl(): string {
	if (browser) {
		return '';
	}

	// SSR: use dynamic public env for runtime resolution (import.meta.env is empty in built output)
	return env.PUBLIC_BACKEND_URL ?? 'http://localhost:8080';
}
