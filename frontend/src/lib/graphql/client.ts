import { Client, cacheExchange, fetchExchange, type AnyVariables } from '@urql/core';
import type { TypedDocumentNode } from '@graphql-typed-document-node/core';
import { Result } from 'true-myth';
import { ApiError, ApiErrorType } from '$lib/api/errors';
import type { CombinedError } from '@urql/core';

export function createGraphQLClient(fetchFn: typeof fetch = fetch): Client {
	return new Client({
		url: '/api/graphql',
		fetch: fetchFn,
		exchanges: [cacheExchange, fetchExchange]
	});
}

export async function query<Data, Variables extends AnyVariables>(
	client: Client,
	document: TypedDocumentNode<Data, Variables>,
	variables: Variables
): Promise<Result<Data, ApiError>> {
	const result = await client.query(document, variables).toPromise();

	if (result.error) {
		return Result.err(mapGraphQLError(result.error));
	}
	if (!result.data) {
		return Result.err(new ApiError(ApiErrorType.ServerError, 'No data returned from GraphQL', 500));
	}
	return Result.ok(result.data);
}

function mapGraphQLError(error: CombinedError): ApiError {
	const gqlError = error.graphQLErrors[0];
	if (gqlError?.extensions?.code) {
		const code = gqlError.extensions.code as string;
		switch (code) {
			case 'NOT_FOUND':
				return new ApiError(ApiErrorType.NotFound, gqlError.message, 404, undefined, code);
			case 'BAD_REQUEST':
				return new ApiError(ApiErrorType.BadRequest, gqlError.message, 400, undefined, code);
			case 'FORBIDDEN':
				return new ApiError(ApiErrorType.Forbidden, gqlError.message, 403, undefined, code);
			case 'UNAUTHORIZED':
				return new ApiError(ApiErrorType.Unauthorized, gqlError.message, 401, undefined, code);
			default:
				return new ApiError(ApiErrorType.ServerError, gqlError.message, 500, undefined, code);
		}
	}

	if (error.networkError) {
		return ApiError.network(error.networkError.message);
	}

	return new ApiError(ApiErrorType.ServerError, error.message, 500);
}
