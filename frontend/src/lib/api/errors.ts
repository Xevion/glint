import { getLogger } from '@logtape/logtape';
import { error } from '@sveltejs/kit';

const logger = getLogger(['ssr', 'api', 'error']);

/**
 * Throw a SvelteKit error with enriched App.Error fields.
 *
 * For API errors, prefer `ApiError.throw()` which automatically
 * propagates the status code, error code, and detail from the backend.
 * Use `pageError` only for non-API errors (e.g. invalid route params).
 */
export function pageError(status: number, message: string): never {
	const errorId = crypto.randomUUID();
	const timestamp = new Date().toISOString();

	if (status !== 404) {
		logger.error('{errorId} [{status}] {message}', { errorId, status, message });
	}

	error(status, { message, errorId, source: 'server', timestamp });
}

export enum ApiErrorType {
	Network = 'NETWORK_ERROR',
	NotFound = 'NOT_FOUND',
	BadRequest = 'BAD_REQUEST',
	ServerError = 'SERVER_ERROR',
	Unauthorized = 'UNAUTHORIZED',
	Forbidden = 'FORBIDDEN',
	Unknown = 'UNKNOWN_ERROR'
}

export class ApiError extends Error {
	public readonly statusCode: number;

	constructor(
		public readonly type: ApiErrorType,
		message: string,
		statusCode?: number,
		public readonly details?: unknown,
		public readonly code?: string,
		public readonly detail?: string
	) {
		super(message);
		this.name = 'ApiError';
		this.statusCode = statusCode ?? ApiError.statusCodeForType(type);
	}

	/**
	 * Log the error and throw a SvelteKit HttpError with enriched App.Error fields.
	 * Use in load functions instead of raw `error()` so SSR failures always produce a log line.
	 */
	throw(): never {
		const errorId = crypto.randomUUID();
		const timestamp = new Date().toISOString();

		logger.error('{errorId} {type} {statusCode}: {message}', {
			errorId,
			type: this.type,
			statusCode: this.statusCode,
			message: this.message,
			...(this.code && { code: this.code }),
			...(this.detail && { detail: this.detail })
		});

		error(this.statusCode, {
			message: this.message,
			errorId,
			source: 'server',
			timestamp,
			code: this.code,
			detail: this.detail
		});
	}

	private static statusCodeForType(type: ApiErrorType): number {
		switch (type) {
			case ApiErrorType.BadRequest:
				return 400;
			case ApiErrorType.Unauthorized:
				return 401;
			case ApiErrorType.Forbidden:
				return 403;
			case ApiErrorType.NotFound:
				return 404;
			case ApiErrorType.Network:
				return 502;
			case ApiErrorType.ServerError:
				return 500;
			case ApiErrorType.Unknown:
				return 500;
		}
	}

	static fromResponse(response: Response, body?: unknown): ApiError {
		let message: string | undefined;
		let code: string | undefined;
		let detail: string | undefined;

		if (typeof body === 'object' && body) {
			if ('error' in body && typeof body.error === 'string') message = body.error;
			if ('code' in body && typeof body.code === 'string') code = body.code;
			if ('detail' in body && typeof body.detail === 'string') detail = body.detail;
		}

		if (!message) {
			if (response.statusText) {
				message = `HTTP ${response.status}: ${response.statusText}`;
			} else {
				const statusMessages: Record<number, string> = {
					400: 'Bad Request',
					401: 'Unauthorized',
					403: 'Forbidden',
					404: 'Not Found',
					500: 'Internal Server Error',
					502: 'Bad Gateway',
					503: 'Service Unavailable',
					504: 'Gateway Timeout'
				};
				const statusName = statusMessages[response.status] || 'Unknown Error';
				message = `HTTP ${response.status}: ${statusName}`;
			}
		}

		switch (response.status) {
			case 400:
				return new ApiError(ApiErrorType.BadRequest, message, 400, body, code, detail);
			case 401:
				return new ApiError(ApiErrorType.Unauthorized, message, 401, body, code, detail);
			case 403:
				return new ApiError(ApiErrorType.Forbidden, message, 403, body, code, detail);
			case 404:
				return new ApiError(ApiErrorType.NotFound, message, 404, body, code, detail);
			case 500:
			case 502:
			case 503:
			case 504:
				return new ApiError(ApiErrorType.ServerError, message, response.status, body, code, detail);
			default:
				return new ApiError(ApiErrorType.Unknown, message, response.status, body, code, detail);
		}
	}

	static network(message: string): ApiError {
		return new ApiError(ApiErrorType.Network, message);
	}
}
