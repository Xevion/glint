import { browser } from '$app/environment';
import { env } from '$env/dynamic/public';

/**
 * Get the API base URL.
 *
 * In development: Uses Vite proxy (relative /api URLs)
 * In production: Uses PUBLIC_API_URL env var or falls back to same origin
 */
export function getApiUrl(): string {
	// In browser, check for public env var
	if (browser && env.PUBLIC_API_URL) {
		return env.PUBLIC_API_URL;
	}

	// Default to relative URLs (works with Vite proxy in dev, same-origin in prod)
	return '';
}

export const API_BASE_URL = getApiUrl();
