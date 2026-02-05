import { createApiClient } from '$lib/api';
import type { Shader } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.admin.listShaders();

	return result.match({
		Ok: (shaders) => ({ shaders, error: null as string | null }),
		Err: (err) => ({ shaders: [] as Shader[], error: err.message })
	});
};
