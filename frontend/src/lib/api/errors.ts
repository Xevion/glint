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
	constructor(
		public readonly type: ApiErrorType,
		message: string,
		public readonly statusCode?: number,
		public readonly details?: unknown,
		public readonly code?: string
	) {
		super(message);
		this.name = 'ApiError';
	}

	static fromResponse(response: Response, body?: unknown): ApiError {
		let message: string | undefined;
		let code: string | undefined;

		if (typeof body === 'object' && body) {
			if ('error' in body && typeof body.error === 'string') message = body.error;
			if ('code' in body && typeof body.code === 'string') code = body.code;
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
				return new ApiError(ApiErrorType.BadRequest, message, 400, body, code);
			case 401:
				return new ApiError(ApiErrorType.Unauthorized, message, 401, body, code);
			case 403:
				return new ApiError(ApiErrorType.Forbidden, message, 403, body, code);
			case 404:
				return new ApiError(ApiErrorType.NotFound, message, 404, body, code);
			case 500:
			case 502:
			case 503:
			case 504:
				return new ApiError(ApiErrorType.ServerError, message, response.status, body, code);
			default:
				return new ApiError(ApiErrorType.Unknown, message, response.status, body, code);
		}
	}

	static network(message: string): ApiError {
		return new ApiError(ApiErrorType.Network, message);
	}
}
