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
		public readonly details?: unknown
	) {
		super(message);
		this.name = 'ApiError';
	}

	static fromResponse(response: Response, body?: unknown): ApiError {
		const message =
			typeof body === 'object' && body && 'message' in body
				? String(body.message)
				: `HTTP ${response.status}: ${response.statusText}`;

		switch (response.status) {
			case 400:
				return new ApiError(ApiErrorType.BadRequest, message, 400, body);
			case 401:
				return new ApiError(ApiErrorType.Unauthorized, message, 401, body);
			case 403:
				return new ApiError(ApiErrorType.Forbidden, message, 403, body);
			case 404:
				return new ApiError(ApiErrorType.NotFound, message, 404, body);
			case 500:
			case 502:
			case 503:
			case 504:
				return new ApiError(ApiErrorType.ServerError, message, response.status, body);
			default:
				return new ApiError(ApiErrorType.Unknown, message, response.status, body);
		}
	}

	static network(message: string): ApiError {
		return new ApiError(ApiErrorType.Network, message);
	}
}
