import { createApiClient } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.shaders.list();

	return result.match({
		Ok: (shaders) => ({
			// Filter out vanilla shader - it's a baseline, not a shader pack
			shaders: shaders.filter((s) => s.slug !== 'vanilla')
		}),
		Err: (error) => ({
			shaders: [],
			error: error.message
		})
	});
};
