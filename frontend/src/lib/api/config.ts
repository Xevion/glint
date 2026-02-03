import { browser } from '$app/environment';

/**
 * Get the API base URL.
 *
 * In development: Uses Vite proxy (relative /api URLs)
 * In production: Uses PUBLIC_API_URL env var or falls back to same origin
 */
export function getApiUrl(): string {
	// In browser, check for public env var
	if (browser && typeof window !== 'undefined') {
		const publicApiUrl = import.meta.env?.PUBLIC_API_URL;
		if (publicApiUrl) {
			return publicApiUrl;
		}
	}

	// Default to relative URLs (works with Vite proxy in dev, same-origin in prod)
	return '';
}

export const API_BASE_URL = getApiUrl();
