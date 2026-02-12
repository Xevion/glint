/**
 * Get the API base URL for the current context.
 *
 * Always returns '' (relative URLs) so that both browser and SSR requests
 * use paths like `/api/...`. In the browser, these are handled by the Vite
 * dev proxy or the production reverse proxy. During SSR, SvelteKit's handle
 * hook intercepts `/api/*` and proxies to the backend.
 *
 * Using relative URLs prevents internal backend addresses (e.g. localhost:3001)
 * from leaking into the serialized HTML via data-sveltekit-fetched data-url attributes.
 */
export function getApiUrl(): string {
	return '';
}
