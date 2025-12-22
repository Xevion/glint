import { Result } from 'true-myth';
import { API_BASE_URL } from './config';
import { ApiError, ApiErrorType } from './errors';

/**
 * Base API client with Result pattern for type-safe error handling
 */
export class ApiClient {
	private baseUrl: string;
	private fetchFn: typeof fetch;

	constructor(baseUrl: string = API_BASE_URL, fetchFn: typeof fetch = fetch) {
		this.baseUrl = baseUrl;
		this.fetchFn = fetchFn;
	}

	/**
	 * Generic fetch wrapper that returns Result<T, ApiError>
	 */
	protected async fetchJson<T>(path: string, options?: RequestInit): Promise<Result<T, ApiError>> {
		const url = `${this.baseUrl}${path}`;

		try {
			const headers = new Headers(options?.headers);
			headers.set('Content-Type', 'application/json');

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
				return Result.err(ApiError.fromResponse(response, body));
			}

			// Parse successful response
			try {
				const data = (await response.json()) as T;
				return Result.ok(data);
			} catch (error) {
				return Result.err(
					new ApiError(
						ApiErrorType.Unknown,
						'Failed to parse response JSON',
						response.status,
						error
					)
				);
			}
		} catch (error) {
			// Network error or other fetch failure
			const message = error instanceof Error ? error.message : 'Network request failed';
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
	 * DELETE request
	 */
	protected delete<T>(path: string): Promise<Result<T, ApiError>> {
		return this.fetchJson<T>(path, { method: 'DELETE' });
	}
}
