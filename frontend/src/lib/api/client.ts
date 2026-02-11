import { browser } from '$app/environment';
import { type Logger, getLogger } from '@logtape/logtape';
import { Result } from 'true-myth';
import { getApiUrl } from './config';
import { ApiError, ApiErrorType } from './errors';

// Lazy-initialized server-side logger.
// logtape's getLogger() is safe to call on the client (returns a no-op logger
// when unconfigured), but we skip it entirely in the browser to avoid bundling.
let _logger: Logger | null = null;

function getApiLogger(): Logger | null {
	if (browser) return null;
	_logger ??= getLogger(['ssr', 'api']);
	return _logger;
}

/**
 * Base API client with Result pattern for type-safe error handling
 */
export class ApiClient {
	private baseUrl: string;
	private fetchFn: typeof fetch;

	constructor(baseUrl: string = getApiUrl(), fetchFn: typeof fetch = fetch) {
		this.baseUrl = baseUrl;
		this.fetchFn = fetchFn;
	}

	/**
	 * Generic fetch wrapper that returns Result<T, ApiError>
	 */
	protected async fetchJson<T>(path: string, options?: RequestInit): Promise<Result<T, ApiError>> {
		const url = `${this.baseUrl}${path}`;
		const method = options?.method ?? 'GET';
		const logger = getApiLogger();

		try {
			const headers = new Headers(options?.headers);
			if (options?.body) {
				headers.set('Content-Type', 'application/json');
			}

			logger?.debug('API request: {method} {path}', { method, url, path });

			const response = await this.fetchFn(url, {
				...options,
				headers
			});

			// Handle non-OK responses
			if (!response.ok) {
				let body: unknown;
				try {
					body = await response.json();
				} catch {
					// Response body isn't JSON, ignore
				}
				logger?.error('API request failed: {method} {path} → {status}', {
					method,
					url,
					path,
					status: response.status,
					statusText: response.statusText
				});
				return Result.err(ApiError.fromResponse(response, body));
			}

			// Handle empty responses (204 No Content, etc.)
			if (response.status === 204 || response.headers.get('content-length') === '0') {
				logger?.debug('API response: {method} {path} → {status}', {
					method,
					path,
					status: response.status
				});
				return Result.ok(null as T);
			}

			// Parse successful response
			try {
				const data = (await response.json()) as T;
				logger?.debug('API response: {method} {path} → {status}', {
					method,
					path,
					status: response.status
				});
				return Result.ok(data);
			} catch (error) {
				logger?.error('API response parse failed: {method} {path}', {
					method,
					url,
					path,
					error: error instanceof Error ? error.message : String(error)
				});
				return Result.err(
					new ApiError(ApiErrorType.Unknown, 'Failed to parse response JSON', response.status, error)
				);
			}
		} catch (error) {
			// Network error or other fetch failure
			const message = error instanceof Error ? error.message : 'Network request failed';
			logger?.error('API request exception: {method} {path}', {
				method,
				url,
				path,
				error: message
			});
			return Result.err(ApiError.network(message));
		}
	}

	/**
	 * GET request
	 */
	protected get<T>(path: string): Promise<Result<T, ApiError>> {
		return this.fetchJson<T>(path, { method: 'GET' });
	}

	/**
	 * GET request for plain text response
	 */
	protected async getText(path: string): Promise<Result<string, ApiError>> {
		const url = `${this.baseUrl}${path}`;

		try {
			const response = await this.fetchFn(url, { method: 'GET' });

			if (!response.ok) {
				let body: unknown;
				try {
					body = await response.json();
				} catch {
					// Response body isn't JSON, ignore
				}
				return Result.err(ApiError.fromResponse(response, body));
			}

			const text = await response.text();
			return Result.ok(text);
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Network request failed';
			return Result.err(ApiError.network(message));
		}
	}

	/**
	 * POST request
	 */
	protected post<T>(path: string, body?: unknown): Promise<Result<T, ApiError>> {
		return this.fetchJson<T>(path, {
			method: 'POST',
			body: body ? JSON.stringify(body) : undefined
		});
	}

	/**
	 * PUT request
	 */
	protected put<T>(path: string, body?: unknown): Promise<Result<T, ApiError>> {
		return this.fetchJson<T>(path, {
			method: 'PUT',
			body: body ? JSON.stringify(body) : undefined
		});
	}

	/**
	 * PATCH request
	 */
	protected patch<T>(path: string, body?: unknown): Promise<Result<T, ApiError>> {
		return this.fetchJson<T>(path, {
			method: 'PATCH',
			body: body ? JSON.stringify(body) : undefined
		});
	}

	/**
	 * DELETE request
	 */
	protected delete<T>(path: string): Promise<Result<T, ApiError>> {
		return this.fetchJson<T>(path, { method: 'DELETE' });
	}
}
