import { browser } from '$app/environment';

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

	// SSR: use public env var for backend origin (accessible in universal load functions)
	return import.meta.env?.PUBLIC_BACKEND_URL ?? 'http://localhost:8080';
}
