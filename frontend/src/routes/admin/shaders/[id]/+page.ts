import { ApiErrorType, pageError } from '$lib/api/errors';
import { createGraphQLClient, query } from '$lib/graphql';
import type { PageLoad } from './$types';
import { ShaderDetailQuery } from './queries';

export const load: PageLoad = async ({ params, fetch, depends }) => {
	depends(`glint:admin:shader:${params.id}`);
	const client = createGraphQLClient(fetch);
	const result = await query(client, ShaderDetailQuery, { id: params.id });

	return result.match({
		Ok: (data) => {
			if (!data.shader) {
				pageError(404, 'Shader not found');
			}
			return { shader: data.shader };
		},
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) pageError(404, 'Shader not found');
			return err.throw();
		}
	});
};
