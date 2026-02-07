import { createApiClient } from '$lib/api';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.admin.getShader(params.id);

	return result.match({
		Ok: (data) => ({ shader: data }),
		Err: (e) => {
			error(e.statusCode ?? 500, e.message);
		}
	});
};
