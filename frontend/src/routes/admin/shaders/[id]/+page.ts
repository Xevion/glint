import { ApiErrorType, pageError } from '$lib/api/errors';
import { createGraphQLClient, query } from '$lib/graphql';
import type { PageLoad } from './$types';
import { AdminShaderQuery } from './queries';

export const load: PageLoad = async ({ params, fetch }) => {
	const client = createGraphQLClient(fetch);
	const result = await query(client, AdminShaderQuery, { id: params.id });

	return result.match({
		Ok: (data) => {
			if (!data.adminShader) {
				pageError(404, 'Shader not found');
			}
			return { shader: data.adminShader };
		},
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) pageError(404, 'Shader not found');
			return err.throw();
		}
	});
};
